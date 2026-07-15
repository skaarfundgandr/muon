# μon Agent Definitions

## 1. Overview

Agent definitions are markdown files with YAML frontmatter and a system prompt body. They live under `examples/agents/` (committed examples) or `~/.config/muon/agents/` (user overrides). The agent_rs wrappers in `src/infrastructure/` load these files at startup and use them to configure each pipeline stage's LLM behavior.

Example agent files are provided for all six μon agents: intent-classifier, clarifier, shallow-researcher, deep-orchestrator, planner, and researcher.

## 2. Frontmatter Schema

Each agent file starts with YAML frontmatter delimited by `---`:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | yes | — | Agent identifier (e.g., `"intent-classifier"`) |
| `model` | string | yes | — | LLM model name (e.g., `"gpt-4o"`, `"gpt-4o-mini"`) |
| `provider` | string | yes | — | LLM provider (`"openai"` supported) |
| `temperature` | f64 | no | 0.0 | Sampling temperature (0.0–2.0). Lower = more deterministic. |
| `max_tokens` | u32 | no | 6144 | Maximum tokens in the LLM response. |
| `timeout_secs` | u64 | no | 60 | Timeout for a single prompt call in seconds. Applied to ReAct agents; ignored for the researcher (`ManagedAgent` has no timeout arg). |

### Minimal Example

```yaml
---
name: my-agent
model: gpt-4o
provider: openai
---
```

## 3. Prompt Body

Everything after the closing `---` is the system prompt, written in free-form markdown. The prompt body is the preamble set on the rig agent via `.preamble(text)`.

### Template Variables

μon supports `{{variable}}` template substitution in prompt bodies. These variables are resolved at runtime:

| Variable | Source | Description |
|----------|--------|-------------|
| `{{query}}` | User input | The current research query |
| `{{context}}` | Clarifier result | Clarification context (Q&A history + plan) |
| `{{previous_findings}}` | Deep researcher | Accumulated findings from prior iterations |
| `{{max_turns}}` | Clarifier config | Maximum clarification turns allowed |

Variables not present in the context are rendered as empty strings.

### Example with variables

```markdown
---
name: clarifier
model: gpt-4o-mini
provider: openai
---

You are the Clarifier for μon.
The user query is: {{query}}
Ask at most {{max_turns}} clarifying questions.
```

## 4. Pipeline Routing

Each agent maps to a specific pipeline stage:

| Agent | Pipeline Stage | Role |
|-------|---------------|------|
| `intent-classifier` | `IntentClassification` | Classify query intent (research vs meta) and depth (shallow vs deep) |
| `clarifier` | `Clarification` | Interactive Q&A to disambiguate scope, time horizon, format, depth |
| `shallow-researcher` | `ShallowResearch` | Single-pass synthesis of top-k search results into a brief |
| `deep-orchestrator` | `DeepResearch` | Multi-loop orchestrator coordinating planner + researcher sub-agents |
| `planner` | `DeepResearch` (sub-agent) | Decompose research question into concrete search sub-queries |
| `researcher` | `DeepResearch` (sub-agent) | Execute sub-queries via web search and synthesize findings |

## 5. Configuration Override

TOML `[agents.*]` sections hold **pipeline orchestration knobs only** (e.g., `max_turns`, `plan_approval`, `max_iterations` for clarifier; `max_llm_turns`, `max_tool_iters` for shallow researcher; deep researcher iteration/tool budgets). Per-agent **model / provider / temperature / max_tokens / timeout** live exclusively in `agents/*.md` YAML frontmatter.

**Settings → Agents** edits both: knobs write to `config.toml`; model/provider/timeout write to the matching agent markdown frontmatter under `agents_dir` (Ctrl+S saves both, then reloads agents).

See `examples/muon.toml` for the full configuration schema.

## 6. Agent Loading

`InfrastructureContext::new_live()` builds real `ReActAgent` wrappers using agent_rs. The system prompt from the agent definition file is used as the rig agent's preamble. The load path uses `infrastructure::config::load_by_name(dir, name)`, searching first the resolved `agents_dir` (default `~/.config/muon/agents/`) then `examples/agents/` (repo fallback). The **filename** is the load key (not the YAML `name` field). Missing files fall through to the next directory; a file that exists but fails to parse returns a hard `MuonError::Config` (no silent fallthrough to the bundled example). If no matching agent definition is found in either directory, `new_live` also returns `MuonError::Config` with both search paths in the message. Providers are configured in `config.toml`; an empty provider list degrades to `ConfigRequiredAgent` stubs that return `MuonError::Config` on every call.

Existing installs: scaffold never overwrites `config.toml` or existing `agents/*.md`. After the YAML SSoT change, model/provider no longer come from TOML — edit `~/.config/muon/agents/*.md` (or delete individual agent files so scaffold can recopy examples on next launch).
