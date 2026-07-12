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
| `max_tokens` | u32 | no | 2048 | Maximum tokens in the LLM response. |
| `timeout_secs` | u64 | no | 60 | Timeout for a single prompt call in seconds. |

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

Agent parameters can also be set in `config.toml` under `[agents.*]`. When both a config file and an agent definition file exist, the config file values take precedence for model/provider/timeout, while the agent definition's system prompt is used as-is.

See `examples/muon.toml` for the full configuration schema.

## 6. Agent Loading

`InfrastructureContext::new_live()` builds real `ReActAgent` wrappers using agent_rs. The system prompt from the agent definition file is used as the rig agent's preamble. Providers are configured in `config.toml`; an empty provider list degrades to `ConfigRequiredAgent` stubs that return `MuonError::Config` on every call. Agent definition files are always loaded from `~/.config/muon/agents/` (user) and `examples/agents/` (repo fallback).
