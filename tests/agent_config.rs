#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::config::agent_config::parse_agent_md;
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
    assert_eq!(def.timeout_secs, 90);
}

#[test]
fn parse_planner() {
    let def = parse_agent_md(&example("planner")).unwrap();
    assert_eq!(def.name, "planner");
    assert!(!def.preamble_markdown.is_empty());
    assert_eq!(def.temperature, 0.3);
    assert_eq!(def.max_tokens, 1024);
    assert_eq!(def.timeout_secs, 30);
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
    let def = parse_agent_md(tmp.path())
        .unwrap_or_else(|e| panic!("empty body should parse: {e}"));
    assert_eq!(def.name, "x");
    assert!(def.preamble_markdown.is_empty());
}
