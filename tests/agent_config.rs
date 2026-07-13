#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::config::AgentsConfig;
use muon::infrastructure::config::{load_by_name, parse_agent_md};
use std::path::PathBuf;

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agents")
        .join(format!("{name}.md"))
}

#[test]
fn parse_deep_orchestrator() {
    let def = parse_agent_md(&example("deep-orchestrator")).unwrap();
    assert_eq!(def.name, "deep-orchestrator");
    assert!(!def.preamble_markdown.is_empty());
    assert_eq!(def.temperature, 0.2);
    assert_eq!(def.max_tokens, 2048);
    assert_eq!(def.timeout_secs, 600);
}

#[test]
fn parse_planner() {
    let def = parse_agent_md(&example("planner")).unwrap();
    assert_eq!(def.name, "planner");
    assert!(!def.preamble_markdown.is_empty());
    assert_eq!(def.model, "glm-5.2-short");
    assert_eq!(def.provider, "NeuralWatt");
    assert_eq!(def.temperature, 0.3);
    assert_eq!(def.max_tokens, 1024);
    assert_eq!(def.timeout_secs, 180);
}

#[test]
fn parse_researcher() {
    let def = parse_agent_md(&example("researcher")).unwrap();
    assert_eq!(def.name, "researcher");
    assert!(!def.preamble_markdown.is_empty());
    assert_eq!(def.max_tokens, 4096);
    assert_eq!(def.timeout_secs, 90);
}

#[test]
fn parse_all_examples_have_preambles() {
    for name in ["intent-classifier", "clarifier", "shallow-researcher"] {
        let def = parse_agent_md(&example(name))
            .unwrap_or_else(|e| panic!("parsing {name} should succeed: {e}"));
        assert_eq!(def.name, name);
        assert!(!def.preamble_markdown.is_empty(), "{name} preamble empty");
    }
}

#[test]
fn parse_missing_frontmatter_delimiter() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "name: x\nmodel: y\nprovider: z\n").unwrap();
    let err = parse_agent_md(tmp.path()).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("missing opening frontmatter delimiter"),
        "unexpected error: {msg}"
    );
}

#[test]
fn parse_missing_required_name_field() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "---\nmodel: y\nprovider: z\n---\nbody text\n").unwrap();
    assert!(parse_agent_md(tmp.path()).is_err());
}

#[test]
fn parse_malformed_yaml() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "---\nname: : :\n---\nbody\n").unwrap();
    assert!(parse_agent_md(tmp.path()).is_err());
}

#[test]
fn parse_empty_body_ok() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "---\nname: x\nmodel: y\nprovider: z\n---\n").unwrap();
    let def = parse_agent_md(tmp.path()).unwrap_or_else(|e| panic!("empty body should parse: {e}"));
    assert_eq!(def.name, "x");
    assert!(def.preamble_markdown.is_empty());
}

#[test]
fn load_by_name_happy_path() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agents");
    let def = load_by_name(&dir, "intent-classifier")
        .unwrap()
        .expect("intent-classifier.md should exist");
    assert_eq!(def.name, "intent-classifier");
    assert!(!def.model.is_empty());
    assert!(!def.provider.is_empty());
}

#[test]
fn load_by_name_missing_returns_none() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agents");
    let got = load_by_name(&dir, "does-not-exist-agent").unwrap();
    assert!(got.is_none());
}

#[test]
fn load_by_name_parse_error_returns_err() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("broken.md"),
        "---\nname: : :\n---\nbody\n",
    )
    .unwrap();
    let err = load_by_name(dir.path(), "broken").unwrap_err();
    assert!(
        err.to_string().contains("invalid YAML") || err.to_string().contains("broken"),
        "unexpected error: {err}"
    );
}

#[test]
fn legacy_toml_agent_model_provider_keys_ignored() {
    let raw = r#"
[clarifier]
max_turns = 7
plan_approval = false
max_iterations = 4
model = "should-be-ignored"
provider = "also-ignored"

[shallow_researcher]
max_llm_turns = 11
max_tool_iters = 6
model = "ignored"
provider = "ignored"

[deep_researcher]
iterations = 3
"#;
    let agents: AgentsConfig =
        toml::from_str(raw).expect("legacy model/provider keys must not fail deserialize");
    assert_eq!(agents.clarifier.max_turns, 7);
    assert!(!agents.clarifier.plan_approval);
    assert_eq!(agents.clarifier.max_iterations, 4);
    assert_eq!(agents.shallow_researcher.max_llm_turns, 11);
    assert_eq!(agents.shallow_researcher.max_tool_iters, 6);
    assert_eq!(agents.deep_researcher.iterations, 3);
}
