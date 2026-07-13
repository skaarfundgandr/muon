#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::config::MuonConfig;
use std::path::Path;

#[test]
fn scaffold_writes_config_and_agents_when_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir: &Path = tmp.path();
    MuonConfig::ensure_scaffolded_in(dir);
    let cfg_path = dir.join("config.toml");
    assert!(cfg_path.exists(), "config.toml not scaffolded");
    let agents_dir = dir.join("agents");
    for name in [
        "intent-classifier.md",
        "clarifier.md",
        "shallow-researcher.md",
        "deep-orchestrator.md",
        "planner.md",
        "researcher.md",
    ] {
        assert!(
            agents_dir.join(name).exists(),
            "agent {name} not scaffolded"
        );
    }
}

#[test]
fn scaffold_is_idempotent_and_does_not_overwrite_edits() {
    let tmp = tempfile::tempdir().unwrap();
    let dir: &Path = tmp.path();
    MuonConfig::ensure_scaffolded_in(dir);
    let cfg_path = dir.join("config.toml");
    let original = std::fs::read_to_string(&cfg_path).unwrap();
    let edited = "# user edit\n".to_string();
    std::fs::write(&cfg_path, &edited).unwrap();
    MuonConfig::ensure_scaffolded_in(dir);
    let after = std::fs::read_to_string(&cfg_path).unwrap();
    assert_eq!(after, edited, "second scaffold overwrote user edit");
    assert_eq!(
        std::fs::read_to_string(dir.join("config.toml")).unwrap(),
        edited
    );
    let _ = original; // anchor
}

#[test]
fn scaffold_skips_existing_agent_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir: &Path = tmp.path();
    let agents_dir = dir.join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    let custom = "# custom clarifier\n".to_string();
    std::fs::write(agents_dir.join("clarifier.md"), &custom).unwrap();
    MuonConfig::ensure_scaffolded_in(dir);
    let after = std::fs::read_to_string(agents_dir.join("clarifier.md")).unwrap();
    assert_eq!(after, custom, "existing agent file was overwritten");
    assert!(
        agents_dir.join("intent-classifier.md").exists(),
        "other files still scaffolded"
    );
}
