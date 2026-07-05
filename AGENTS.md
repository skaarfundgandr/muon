# AGENTS.md — μon

Terminal-based deep research agent TUI. Rust, ratatui, crossterm, tokio.

## Architecture

CLEAN layered architecture. Presentation → Application → Domain → Infrastructure.

- **Presentation** (`src/presentation/`): ratatui rendering. Components in 5 categories: chrome, inputs, panels, cards, graphs. 5 views (Welcome, Dashboard, Progress, Results, Settings) with handlers, layouts, form system, click-target registry.
- **Application** (`src/application/`): pipeline state machine (`PipelineStage` idle → intent → clarify → shallow → deep → complete/cancelled).
- **Domain** (`src/domain/`): pure models and port traits (MuonAgent, SearchProvider, VectorStore, SessionStore).
- **Infrastructure** (`src/infrastructure/`): agent_rs ReAct wrappers, Diesel storage (SQLite), RAG (TurboVec + FastEmbed), search providers (Brave/SearXNG/SemanticScholar/arXiv), export (Markdown/Obsidian).

**Bootstrap:** `src/main.rs` calls `app::run()` which sets up terminal (raw mode, alternate screen, mouse capture), spawns tokio event task (250ms poll on mpsc channel), runs main loop.

**Headless CLI:** `muon run --headless --mock "query"` prints report to stdout without TUI. `muon export <session> <format> -o path` exports completed sessions. `MUON_LIVE=1` env var gates real OpenAI/Diesel/RAG adapters; default is mock.

## Module Conventions

- `src/lib.rs` flatly re-exports top-level modules: `app`, `application`, `config`, `error`, `presentation`, `session`.
- Each directory has a `mod.rs` that re-exports its children — no implementation in `mod.rs`.
- `thiserror` for errors: `MuonError` enum in `src/error.rs`, type alias `Result<T>`.
- TOML config (serde): `MuonConfig` loads from `~/.config/muon/config.toml` with `Default` fallback. Sub-configs: Agents, Tools, DataSources, Display, Advanced.
- Edition 2024. Clippy denies `unwrap_used`, `expect_used`, `panic`; forbids `todo`, `unimplemented`.

## Key Patterns

### Form System (`src/presentation/form.rs`)
`FieldDef` (label + `FieldKind`: Text, Number, Dropdown, Checkbox, Button) + `FormState` (focus index, edit buffer, dropdown state, dirty flag). Bracket convention: `[value▼]` (dropdown), `[✓]`/`[ ]` (checkbox). Settings panels each expose `fields()`, `get_field()`, `set_field()`, `toggle_field()`, `render()`.

### Click-Target Registry (`src/presentation/click.rs`)
`ClickTarget` (id + `Rect` + `ClickAction`). Accumulated in `hit_registry` during render, cleared per frame. `handle_mouse_click()` reverse-iterates for hit testing. Actions: activate field, toggle checkbox, switch tab/view, activate query input.

### View Router (`src/presentation/views/`)
`View` enum (5 variants), `ViewRouter` (Tab/F-key navigation), `RenderParams` struct passed to each view's `render()`. Handlers dispatch by active view: `dashboard.rs` (query input), `settings.rs` (form nav/edit/save), `view_events.rs` (global keybinds).

### Pipeline State Machine (`src/application/pipeline.rs`)
`PipelineState` with `stage: PipelineStage`, timing, step counters. `advance()` sequences through stages. Runs on its own tokio task, communicates via mpsc channel to TUI event loop.

## SPEC Reference

Full specifications live in `SPEC/` (git submodule, private repo: `github.com/skaarfundgandr/muon-spec`):

| File | Content |
|------|---------|
| `SPEC/SPEC.md` | Full reference spec (Draft 2) |
| `SPEC/frontend/SPEC.md` | TUI views, components, handlers, forms, click registry, theme |
| `SPEC/backend/SPEC.md` | Pipeline orchestration, agent definitions, data model, storage (SQLite + turbovec), error handling |
| `SPEC/mockup/*.html` | Visual mockups for all 5 views |

Reference these by path only. The SPEC is the source of truth for planned features not yet implemented.

## Test Conventions

All tests go in `tests/` (integration tests). No `#[cfg(test)]` inline modules in `src/`.

### Clippy in Test Code

The Cargo.toml clippy lints (`unwrap_used = "deny"`, `expect_used = "deny"`, `panic = "deny"`) apply to `src/` production code only. Test code in `tests/` must opt out at the module level:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
```

## Documentation

- `docs/backend.md` — architecture overview, pipeline state machine, storage schema, citation verifier, export, LangSmith, headless CLI.
- `docs/agents.md` — how to author agent definition files (frontmatter schema, prompt body, template variables, pipeline routing).
- `examples/agents/` — six example agent definitions (intent-classifier, clarifier, shallow-researcher, deep-orchestrator, planner, researcher).
- `examples/muon.toml` — complete example configuration file.
