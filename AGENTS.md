# AGENTS.md — μon

Terminal-based deep research agent TUI. Rust, ratatui, crossterm, tokio.

## Architecture

CLEAN layered architecture. Presentation → Application → Domain → Infrastructure (domain and infrastructure layers not yet created).

- **Presentation** (`src/presentation/`): ratatui rendering. Components in 5 categories: chrome, inputs, panels, cards, graphs. 5 views (Welcome, Dashboard, Progress, Results, Settings) with handlers, layouts, form system, click-target registry.
- **Application** (`src/application/`): pipeline state machine (`PipelineStage` idle → intent → clarify → shallow → deep → complete/cancelled).
- **Domain** (`src/domain/`): not yet created. SPEC describes pure models and traits (ports): SearchProvider, VectorStore, SessionStore, MuonAgent.
- **Infrastructure** (`src/infrastructure/`): not yet created. SPEC describes agent_rs wrappers, search providers, diesel storage, export.

**Bootstrap:** `src/main.rs` calls `app::run()` which sets up terminal (raw mode, alternate screen, mouse capture), spawns tokio event task (250ms poll on mpsc channel), runs main loop.

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

No tests exist yet. When added: `#[cfg(test)]` modules inline in `src/` files or `tests/` directory for integration tests. No test framework dependency currently in `Cargo.toml`.
