# μon Backend Architecture

## 1. Overview

μon is a terminal-based deep research agent. It takes a user query, classifies intent, optionally clarifies scope interactively, then runs either shallow (single-pass) or deep (multi-loop orchestrator/planner/researcher) research. The output is a citation-backed structured report.

On first launch, μon creates `~/.config/muon/config.toml` (copy of `examples/muon.toml`) and `~/.config/muon/agents/` containing the six agent preamble `.md` files. Existing files are never overwritten — edit them in place or use the Settings view. There is no separate `muon init` subcommand.

The backend follows a **CLEAN layered architecture**:

```
Presentation (ratatui TUI)
    ↓ events / ↑ AgentEvent
Application (pipeline state machine, services)
    ↓ trait calls
Domain (pure models + port traits)
    ↓ impl via adapters
Infrastructure (agent_rs, Diesel, reqwest, TurboVec)
```

Domain and Infrastructure are separated by port traits defined in `src/domain/traits/`. Infrastructure adapters implement these ports, keeping domain logic free of framework dependencies.

## 2. Module Layout

### Presentation (`src/presentation/`)
ratatui rendering layer. Components organized into chrome, inputs, panels, cards, and graphs. Five views (Welcome, Dashboard, Progress, Results, Settings) with handlers, form system, and click-target registry. The TUI consumes `AgentEvent`s from an mpsc channel to update its display; it never calls domain/infrastructure directly.

### Application (`src/application/`)
Pipeline orchestration, bridge types, and services. The pipeline state machine (`PipelineStage`) drives stages from Idle through IntentClassification, Clarification, ShallowResearch, DeepResearch, to Complete/Cancelled. The pipeline runner (`pipeline_runner/`) sequences agents and handles escalation from shallow to deep. Services include report building and export (Markdown, Obsidian).

### Domain (`src/domain/`)
Pure models (Session, Source, Citation, ResearchReport, LogEntry, QueryIntent, ResearchPlan) and port traits (MuonAgent, SearchProvider, VectorStore, SessionStore). No framework deps — only `serde`, `chrono`, `uuid`, and `async-trait`.

### Infrastructure (`src/infrastructure/`)
Concrete adapters:
- **agent_rs** (`infrastructure/agent_rs/`): Type-erased ReAct wrappers converting `agent_rs::BuiltReAct` into `Box<dyn MuonAgent>`.
- **Storage** (`infrastructure/storage/`): Diesel schema, migrations, connection pool, and typed repos for sessions, sources, citations, reports, and log entries.
- **RAG** (`infrastructure/rag/`): TurboVec vector index + FastEmbed embeddings for retrieval-augmented generation.
- **Search** (`infrastructure/search/`): Web search (Brave, Tavily, Firecrawl, Serper) and paper search (arXiv) providers. The `CompositeSearchProvider` fan-out dispatches queries to all configured `[[search.providers]]` concurrently, merges results, and dedupes by URL.
- **Context** (`infrastructure/context.rs`): `InfrastructureContext` wiring all adapters together.

## 3. Pipeline State Machine

Defined in `src/application/pipeline.rs`:

```rust
pub enum PipelineStage {
    Idle,
    IntentClassification,
    Clarification,
    ShallowResearch,
    DeepResearch,
    CitationVerify,
    Report,
    Complete,
    Cancelled,
    Failed,
}
```

`PipelineState` tracks the current stage, timing, and step count. The pipeline runs on its own tokio task and communicates with the TUI via:
- **mpsc channel** (`AgentEvent`): Stage changes, log entries, clarifier questions, plan proposals, final results.
- **oneshot channel**: TUI sends clarifier answers and plan decisions back to the pipeline.

The pipeline runner (`src/application/pipeline_runner/runner.rs`) sequences through stages:
1. **IntentClassification** → classify query as meta/research and shallow/deep.
2. **Clarification** (if needed) → interactive Q&A loop, then optional plan generation/approval.
3. **ShallowResearch** → single-pass synthesis with escalation check.
4. **DeepResearch** → orchestrator/planner/researcher multi-loop iteration.
5. **CitationVerify** → verify report citations against the source registry.
6. **Report** → build the final structured report.
7. **Complete**, **Cancelled**, or **Failed**.

## 4. Infrastructure Context

`InfrastructureContext` (`src/infrastructure/context.rs`) holds all concrete adapters behind trait objects:

```rust
pub struct InfrastructureContext {
    pub intent_classifier: Box<dyn MuonAgent>,
    pub shallow: Box<dyn MuonAgent>,
    pub clarifier: Box<dyn MuonAgent>,
    pub deep_orchestrator: Box<dyn MuonAgent>,
    pub planner: Box<dyn MuonAgent>,
    pub researcher: Box<dyn MuonAgent>,
    pub session_store: Box<dyn SessionStore>,
}
```

**Constructor:**
- `InfrastructureContext::new_live(cfg, bridge)` — builds real provider-backed ReAct agents, a Diesel session pool, and a `DieselSessionStore`. Works with zero or more `[[providers]]` entries; an empty list degrades to stub agents that return `MuonError::Config` on every call.

When `cfg.providers` is empty, `new_live` degrades gracefully: it initializes the session store (so existing sessions load), installs `ConfigRequiredAgent` stubs for all six agent roles that return `MuonError::Config` on every `prompt_raw`, and returns `Ok`. The TUI starts with an Info toast directing the user to Settings → Providers. No fake API keys are generated.

## 5. Storage Schema

**Never edit `src/infrastructure/storage/schema.rs` by hand** — it is Diesel CLI–generated. Schema changes go through SQL migrations + `diesel migration run` / `diesel print-schema`.

SQLite via Diesel with a single process-wide deadpool connection pool (`OnceLock<DbPool>`). The pool is initialized once via `init_pool(path)` and all subsequent calls (infra rebuild, CLI export) return a clone of the same singleton. A hot-swap of `session_db_path` after first init errors with a "restart muon" message.

Every pooled connection runs SQLITE_PRAGMAs via a `post_create` Hook:
- `foreign_keys = ON`
- `journal_mode = WAL`
- `synchronous = NORMAL`
- `busy_timeout = 5000`
- `mmap_size = 30000000000`

This eliminates `database is locked` errors under concurrent `append_log` writes. `create_pool(path)` is the test-friendly factory (no singleton, no migrations); `global_pool()` clones the singleton or errors. Two logical databases:
- `sessions.db` — sessions, sources, log entries, research reports, citations.
- `rag.db` — chunk vectors + embedding cache (TurboVec/FastEmbed).

### Tables and Relationships

```
sessions (id TEXT PK, query, status, pipeline_stage, plan_json, clarifier_result,
          telemetry_json, created_at, updated_at)
  │
  ├─1:N─ log_entries (id INTEGER PK, session_id FK, agent_tag, message, level, timestamp)
  │
  ├─1:N─ sources (id INTEGER PK, session_id FK, url, title, snippet, relevance,
  │               source_type, verification_status, embedding_id)
  │
  └─1:N─ research_reports (id INTEGER PK, session_id FK, title, content,
                           stats_json, created_at)
                │
                └─1:N─ citations (id INTEGER PK, report_id FK, reference_number,
                                 url, title, context_snippet, verification_level)
```

### Row Types

| Row | Domain Model | Key Fields |
|-----|-------------|------------|
| `SessionRow` | `Session` | `id`, `query`, `status`, `pipeline_stage`, `plan_json`, `clarifier_result` |
| `SourceRow` | `Source` | `url`, `title`, `snippet`, `relevance`, `source_type`, `verification_status` |
| `CitationRow` | `Citation` | `reference_number`, `url`, `title`, `context_snippet`, `verification_level` |
| `ReportRow` | `ResearchReport` | `title`, `content` (JSON), `stats_json` |
| `LogEntryRow` | `LogEntry` | `agent_tag`, `message`, `level`, `timestamp` |

## 6. Citation Verifier 5-Level Flow

The citation verifier (`src/application/pipeline_runner/citation_verifier.rs`) checks each citation in a report against a registry of URLs retrieved during research. The verification pipeline:

### Level 1: Exact Match
Normalized report URL equals a normalized registry URL (case-insensitive, trailing slash stripped, `www.` stripped, fragment dropped).

### Level 2: Prefix Match (Truncation)
A registry URL exists under the report URL's "area" (the path with its last segment stripped). Only counts if exactly one registry URL matches — e.g., the report cites `https://example.com/blog/` and the only matching registry entry is `https://example.com/blog/2024-01-post-title`.

### Level 3: Prefix Match (Path)
The normalized report URL is a prefix of a registry URL — e.g., the report cites `https://example.com/docs/guide` and a registry entry `https://example.com/docs/guide/intro` exists.

### Level 4: Child Path Match
The report URL and a registry URL share the same host, and the report URL's path is a subpath of the registry URL's path (or vice versa). Requires 2+ path segments in both paths. This catches truncated URLs that point to a child of a known source — e.g., the report cites `https://example.com/docs/guide/intro` and the registry has `https://example.com/docs/guide`.

### Level 5: Query Subset Match
Same host+path, and the report URL's query parameters are a subset of the registry URL's query parameters.

**Before matching**, URLs are sanitized: filtered if they are `bit.ly`/`t.co` shorteners, contain `...`, are IP literals, or use `javascript:`/`data:`/`vbscript:`/`file:` schemes.

Knowledge-layer citations (e.g., `"report.pdf, p.15"`) are matched via substring against knowledge paths.

Unmatched citations are removed with a reason: `UrlNotInRegistry`, `CitationKeyNotInRegistry`, or `Unverifiable`. The report is then reflowed with renumbered `[N]` references.

## 7. Export

Four export components in `src/application/services/`:

- **`ExportService`** — routes export requests to the appropriate exporter based on `ExportFormat` (`markdown` / `obsidian` / `pdf`).
- **`MarkdownExporter`** — writes a report to `~/.local/share/muon/exports/<session_id>.md` with YAML frontmatter (title, query, created_at, source count).
- **`ObsidianExporter`** — writes the report into `<vault_path>/Muon/<slug>.md` and appends a link to the map of content at `Muon/MOC.md`. Slug is the title lowercased with dashes, max 60 chars.
- **`PdfExporter`** — renders the same markdown body as `MarkdownExporter` to `~/.local/share/muon/exports/<session_id>.pdf` via `pdf_oxide::api::Pdf::from_markdown` + `save`. Reaches the TUI via the **F2** action-bar button (`[F2] Export PDF`) and via `muon export <session> pdf -o path` on the CLI; F2 has full parity with F1 Export MD (hover, click hit-target, success/error toasts). `pdf_oxide` is also the PDF→Markdown ingest source for RAG directory indexing (multi-page, see §3.9).

## 8. LangSmith Tracing

μon initializes OpenTelemetry tracing at process start (`src/infrastructure/observability.rs`) and emits ReAct spans (thoughts, actions, observations) to LangSmith. Configuration is process-start only — restart the process to apply changes (config hot-reload does not re-init the tracer).

### Config (`[observability]`)

```toml
[observability]
debug = false   # true = never truncate OTEL span attributes; TUI live feed still truncated
```

### Config (`[observability.langsmith]`)

```toml
[observability.langsmith]
api_key = ""                # or "${LANGSMITH_API_KEY}"
project = "default"
endpoint = "https://api.smith.langchain.com/otel/v1/traces"
service_name = "muon"
console = false
batch = true
batch_delay_ms = 1000
```

**API key precedence:** TOML `api_key` (with optional `${ENV}` expansion) → else `LANGSMITH_API_KEY` env. Empty / unset key disables export (no-op handle). Other fields overlay agent_rs defaults / remaining OTEL env vars (`LANGSMITH_PROJECT`, `OTEL_EXPORTER_OTLP_ENDPOINT`, etc.).

## 9. Headless CLI

μon supports three entry points:

```bash
# Interactive TUI (default)
muon
muon tui

# Headless mode — runs pipeline and prints report to stdout
muon run --headless "What is async Rust?"

# Export a completed session
muon export <session-id> markdown -o report.md
# Obsidian: set [obsidian] vault_path in config.toml or MUON_OBSIDIAN_VAULT
muon export <session-id> obsidian -o report.md
```

Headless mode always uses live infrastructure (`InfrastructureContext::new_live`). Zero `[[providers]]` configured degrades to `ConfigRequiredAgent` stubs (see §Constructor).

Reports are Markdown formatted with frontmatter, section headings, and renumbered citation references.

## 10. Dynamic Provider Model

The runtime configuration model is dynamic — providers are declared as a list rather than hardcoded. Each agent references a provider by `name`; each search provider declares its `type` and per-provider options inline.

### 10.1 LLM providers (`[[providers]]`)

```toml
[[providers]]
name = "DeepSeek"
base_url = "https://api.deepseek.com/v1"
api_key = "${DEEPSEEK_API_KEY}"

[[providers.models]]
name = "DeepSeek V4 Flash"
description = "DeepSeek V4 Flash, fast inference"
```

Agent configuration references a provider by `name`:

```yaml
---
name: intent-classifier
model: Gemma 4 E2B
provider: Ollama
---
```

Resolution: `ResolvedClient::for_named_provider(name, &cfg.providers)` looks up the `name` in `cfg.providers` and builds an `openai::Client` from `api_key` + `base_url`. If `name` is empty, `for_default_provider` returns the first entry.

### 10.2 `${ENV_VAR}` expansion

Provider `api_key` and search provider `api_key` fields support `${ENV_VAR}` placeholders. At resolution time, the literal `${VAR}` is replaced by `std::env::var("VAR")`. If the variable is unset or empty, a `MuonError::Config` is raised before any HTTP call.

The `expand_env(value: &str) -> Result<String, MuonError>` helper in `src/infrastructure/config/env.rs` does the substitution and is reused by both LLM and search providers.

### 10.3 Search providers (`[[search.providers]]` + fan-out)

```toml
[[search.providers]]
name = "tavily"
type = "tavily"
api_key = "${TAVILY_API_KEY}"
max_results = 10

[[search.providers]]
name = "brave"
type = "brave"
api_key = "${BRAVE_API_KEY}"
```

At context bootstrap (`InfrastructureContext::new_live`), `resolve_web_provider` builds an `Arc<dyn SearchProvider>` for each enabled entry, then wraps them in a `CompositeSearchProvider`. The composite dispatches every query to all providers concurrently, merges the result lists, dedupes by URL, and returns the union. A provider that fails logs a warning and contributes zero sources (failures are non-fatal).

Supported `type` values: `tavily`, `firecrawl`, `brave`, `serper`. Per-provider options (e.g. Tavily `search_depth`, Serper `gl`/`hl`) live inline on each `[[search.providers]]` entry as a sub-table.

### 10.4 Paper search (`[search.papers]`)

Currently arXiv only (`arxiv_enabled: bool`). Semantic Scholar was removed per the v0.1 spec; SearXNG was also removed.

### 10.5 Provider configuration model

LLM providers are declared via `[[providers]]` entries with `name`, `base_url`, and `api_key`; search providers via `[[search.providers]]` entries with `type` in {tavily, firecrawl, brave, serper}. There is no legacy `[tools]` table or backfill — the `ToolsConfig` struct and `backfill_legacy_providers()` were removed in the config audit.

**CLEAN layout:** Application owns pure settings types (`application::config::{types, agent_def}`). Infrastructure owns load/save/watch/scaffold, agent `.md` parse, and `${ENV}` expansion (`infrastructure::config::{load, env, agent_md}`). Consumers call `infrastructure::config::resolve_api_key(&provider)` directly; `ProviderConfig::resolved_api_key()` is removed.

### 10.6 TUI

The Settings view has 6 tabs (in order): `Providers`, `Agents`, `Tools`, `Data Sources`, `Display`, `Advanced`. The `Providers` tab is a dynamic row editor over `cfg.providers`; the `Tools` tab renders `cfg.search.providers` rows + the arXiv toggle. The `Agents` tab edits pipeline orchestration knobs only (clarifier / shallow / deep budgets and flags). Per-agent model, provider, temperature, max_tokens, and timeout_secs live in `agents/*.md` YAML frontmatter — edit those files outside the TUI.

### 10.7 Removed (per SPEC) / per feature-wiring audit

- `SearXNGProvider` — removed; use Brave/Tavily/Firecrawl/Serper instead.
- `SemanticScholarProvider` — removed; arXiv is the only paper source.
- The old hardcoded `match provider { "openai" | "anthropic" | ... }` dispatch in `providers.rs` was replaced by `for_named_provider` / `for_default_provider`.
- `FetchPageTool` is reqwest-only; the SPEC-mandated Firecrawl/Tavily fallback chain is a follow-up.
- **Feature-wiring cleanup (branch `feat/feature-wiring-spec-alignment`):**
  - `enterprise_systems` config field, `SourceType::Enterprise`, `SourceType::Code`, their DB mapping arms, and the Enterprise source-registry UI row — stripped (no producer existed; `try_from_str` maps unknown DB strings to `Web` with `tracing::warn!`).
  - Hand-rolled Tavily / Firecrawl / Serper HTTP replaced with the `tavily` / `firecrawl` / `serper-sdk` crates; Brave stays hand-rolled (no adequate library SDK).

### 10.8 Knowledge Layer RAG

The RAG layer is **off by default** (`knowledge_layer_rag = false` in `DataSourcesConfig::default()` and `examples/muon.toml`). When enabled:

- `RagContext::open` runs at startup, ahead of agent construction, and logs `Sys` info/warn accordingly (fail-open preserved on init error).
- **Research agents only** receive RAG context — **Shallow Researcher** and the deep **Researcher** sub-agent get `.dynamic_context(rag_top_k, index)` on the rig builder plus the `KnowledgeSearchTool`. Planner, Orchestrator, Intent Classifier, and Clarifier do **not** receive either (Planner plan decomposition relies on template context, not local corpus retrieval).
- The thin `RagToRigIndex` adapter (in `src/infrastructure/rag/rag_to_rig_index.rs`) bridges `Arc<RagContext>` to rig's `VectorStoreIndexDyn` via the existing `TurboVectorIndex` query path.
- Directory indexing is wired through `RagIndexerService` (application layer): walks Directory / File / Glob paths, indexes `.txt` / `.md` directly, and pre-extracts **all pages** of `.pdf` files to markdown via `pdf_oxide` so the agent_rs indexer stays txt/md. The TUI `ClickAction::ReindexRagIndex` action spawns the index job asynchronously and updates status (`○ indexing… → ● indexed / ⚠ error`) and chunk count without blocking the event loop. When RAG is disabled, reindex skips embedding entirely and marks the row `○ disabled`.
- The post-research embed loop in `deep_researcher.rs` (~142–174) is intentionally retained for corpus growth; it will be revisited if directory indexing fully supersedes it.
- `SourceType::Knowledge` gets a distinct `📚 knowledge` card badge (cyan) — no longer mapped to Web.

## 11. Intentional SPEC Drift

This section documents design decisions that intentionally diverge from SPEC Draft 2, so future agents don't "fix" them.

### 11.1 Clarifier config single source of truth

`[agents.clarifier]` is the sole source for `plan_approval`, `max_turns`, and `max_iterations`. The `migrate_clarifier_config()` and `mirror_clarifier_to_advanced()` dual-write functions were removed in the config audit — there is no migration or mirror code. Default `plan_approval = true` (fresh configs get the plan-approval gate). The Advanced settings UI no longer shows these three controls; they live exclusively in the Agents tab.

This is intentional drift from SPEC Draft 2, which described a dual-write scheme between `[advanced]` and `[agents.clarifier]`.

### 11.2 Pipeline stages

The full stage list is: `Idle, IntentClassification, Clarification, ShallowResearch, DeepResearch, CitationVerify, Report, Complete, Cancelled, Failed`. The `CitationVerify` and `Report` stages are emitted by both shallow and deep paths before terminal `Complete`. On pipeline `Err`, `mark_session_terminal` persists `Cancelled` (for `MuonError::Cancelled`) or `Failed` to the DB best-effort.

`SessionStatus` gains `Cancelled` alongside `Pending, Clarifying, Researching, Complete, Failed`. `DieselSessionStore::update_stage` derives the status column from the stage string so they stay in sync.

### 11.3 Deep researcher design

- Deep orchestrator uses `max_retries` (not `iterations`) as the retry loop count. Each iteration produces a full draft; the quality gate (`is_report_complete`) checks length, section count, source coverage, and "gave up" signal patterns.
- The orchestrator uses `SubagentTool` (`PlannerKind`, `ResearcherKind`) to delegate planning and research. The planner uses the `think` tool; the researcher is a `ManagedAgent` (no `think` tool — it runs search/fetch then answers).
- Researcher soft-fails: on `MaxCycles` or partial-draft errors, it produces a stub report from the source registry rather than aborting.

### 11.4 Process-wide SQLite pool

A single `OnceLock<DbPool>` per process. `init_pool` is idempotent — multiple calls (infra rebuild, CLI export) return clones of the same pool. No second pool is ever created for the same path. PRAGMAs are applied per-connection via `post_create` Hook, not URI strings.

### 11.5 Escalation text scope

`should_escalate` checks the **full report text** (summary + all section body markdown joined by `\n`), not just the summary. It takes the last **800 Unicode scalars** and checks against the keyword list: `unable to find`, `need more research`, `i don't have enough information`.

### 11.6 Empty config behavior

Missing or empty `config.toml` (no `[[providers]]`) does NOT crash the TUI. `new_live` degrades: it initializes the session store, installs `ConfigRequiredAgent` stubs, and returns `Ok`. The TUI shows an Info toast directing to Settings → Providers. Research attempts fail with `MuonError::Config` until providers are configured.

### 11.7 Session plan persistence

`SessionStore::save_clarifier_outcome(id, plan_json, clarifier_result_json)` writes the `plan_json` and `clarifier_result` columns. Called best-effort after a successful deep clarifier run. `ClarifierResult::to_plan()` converts to `ResearchPlan` for serialization. The domain `Session` model has `intent: Option<String>`, `plan: Option<ResearchPlan>`, `clarifier_result: Option<String>` fields. `Session.intent` is a reserved field for future intent-classifier wiring; it is currently left unset. (Tracked as review finding #35.) No schema migration was needed — the columns pre-existed.

### 11.8 Non-goals (explicitly out of scope)

- Enterprise search implementation — removed (see §10.7 Removed; `enterprise_systems` config field stripped).
- Researcher BuiltReAct + `think` tool.
- Truncation `VerificationLevel` variant.
- Full CLEAN DI rewrite / removing `InfrastructureContext`.
- Top-level TOML rename to `[pipeline]`/`[storage]` (documented mapping only).
- Auto-provision API keys.
- App-level SQLite retries as replacement for WAL + busy_timeout + single pool.
