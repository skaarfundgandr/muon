# μon Backend Architecture

## 1. Overview

μon is a terminal-based deep research agent. It takes a user query, classifies intent, optionally clarifies scope interactively, then runs either shallow (single-pass) or deep (multi-loop orchestrator/planner/researcher) research. The output is a citation-backed structured report.

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
- **Search** (`infrastructure/search/`): Web search (Brave, SearXNG) and paper search (Semantic Scholar, arXiv) providers.
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
    Complete,
    Cancelled,
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
5. **Complete** or **Cancelled**.

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

**Two constructors:**
- `InfrastructureContext::mock()` — returns deterministic `MockAgent`s for testing and dev.
- `InfrastructureContext::new_live(cfg, bridge)` — builds real OpenAI-backed ReAct agents, a Diesel connection pool, and a `DieselSessionStore`. Requires `OPENAI_API_KEY` env var.

Set `MUON_LIVE=1` to gate real adapters. The default (mock) bypasses LLM calls and returns canned responses.

## 5. Storage Schema

SQLite via Diesel with a deadpool connection pool. Two logical databases:
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
The normalized report URL is an area prefix of exactly one registry URL — e.g., `https://example.com/blog/` matches `https://example.com/blog/2024-01-post-title`.

### Level 3: Prefix Match (Path)
The normalized report URL is a prefix of a registry URL — e.g., `https://example.com/docs/guide` is a prefix of `https://example.com/docs/guide/intro`.

### Level 4: Child Path Match
The report URL's final path segment appears as a segment in a registry URL. Requires 2+ path segments in the report URL.

### Level 5: Query Subset Match
Same host+path, and the report URL's query parameters are a subset of the registry URL's query parameters.

**Before matching**, URLs are sanitized: filtered if they are `bit.ly`/`t.co` shorteners, contain `...`, are IP literals, or use `javascript:`/`data:`/`vbscript:`/`file:` schemes.

Knowledge-layer citations (e.g., `"report.pdf, p.15"`) are matched via substring against knowledge paths.

Unmatched citations are removed with a reason: `UrlNotInRegistry`, `CitationKeyNotInRegistry`, or `Unverifiable`. The report is then reflowed with renumbered `[N]` references.

## 7. Export

Three export components in `src/application/services/`:

- **`ExportService`** — routes export requests to the appropriate exporter based on format.
- **`MarkdownExporter`** — writes a report to `~/.local/share/muon/exports/<session_id>.md` with YAML frontmatter (title, query, created_at, source count).
- **`ObsidianExporter`** — writes the report into `<vault_path>/Muon/<slug>.md` and appends a link to the map of content at `Muon/MOC.md`. Slug is the title lowercased with dashes, max 60 chars.

## 8. LangSmith Tracing

Enable distributed tracing via LangSmith by setting the `LANGSMITH_API_KEY` environment variable. When set, μon initializes OpenTelemetry tracing at bootstrap (`src/infrastructure/observability.rs`) and emits ReAct spans (thoughts, actions, observations) to LangSmith for monitoring and debugging.

The `LANGSMITH_API_KEY` is optional — when unset, tracing initializes as a no-op.

## 9. Headless CLI

μon supports three entry points:

```bash
# Interactive TUI (default)
muon
muon tui

# Headless mode — runs pipeline and prints report to stdout
muon run --headless --mock "What is async Rust?"
MUON_LIVE=1 muon run --headless "Compare diesel-async and sqlx"

# Export a completed session
muon export <session-id> markdown -o report.md
muon export <session-id> obsidian --vault ~/my-vault
```

`--mock` uses `InfrastructureContext::mock()` for deterministic output without API keys. Without `--mock`, `MUON_LIVE=1` must be set and `OPENAI_API_KEY` must be present.

Reports are Markdown formatted with frontmatter, section headings, and renumbered citation references.
