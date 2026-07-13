#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::config::{MuonConfig, SearchProviderType};
use muon::infrastructure::config;
use muon::infrastructure::observability::map_langsmith_config;
use std::path::Path;

#[test]
fn scaffold_writes_config_and_agents_when_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let dir: &Path = tmp.path();
    config::ensure_scaffolded_in(dir);
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
fn scaffolded_config_toml_deserializes_with_providers() {
    let tmp = tempfile::tempdir().unwrap();
    let dir: &Path = tmp.path();
    config::ensure_scaffolded_in(dir);
    let cfg = config::load_from_path(&dir.join("config.toml"));
    assert!(
        !cfg.providers.is_empty(),
        "scaffolded config should retain [[providers]] (got empty Default?)"
    );
    assert!(
        !cfg.search.providers.is_empty(),
        "scaffolded config should retain [[search.providers]]"
    );
    assert!(
        cfg.search
            .providers
            .iter()
            .any(|p| p.provider_type == SearchProviderType::Tavily),
        "expected tavily search provider from type = \"tavily\""
    );
    assert!(
        cfg.search
            .providers
            .iter()
            .any(|p| p.provider_type == SearchProviderType::Brave),
        "expected brave search provider from type = \"brave\""
    );
    assert_ne!(
        cfg.agents.intent_classifier.model,
        MuonConfig::default().agents.intent_classifier.model
    );
}

#[test]
fn examples_muon_toml_round_trips() {
    let raw = include_str!("../examples/muon.toml");
    let cfg: MuonConfig = toml::from_str(raw).expect("examples/muon.toml must parse as MuonConfig");
    assert_eq!(cfg.search.providers.len(), 2);
    assert_eq!(
        cfg.search.providers[0].provider_type,
        SearchProviderType::Tavily
    );
    assert_eq!(
        cfg.search.providers[1].provider_type,
        SearchProviderType::Brave
    );
    assert!(!cfg.providers.is_empty());
}

#[test]
fn scaffold_is_idempotent_and_does_not_overwrite_edits() {
    let tmp = tempfile::tempdir().unwrap();
    let dir: &Path = tmp.path();
    config::ensure_scaffolded_in(dir);
    let cfg_path = dir.join("config.toml");
    let edited = "# user edit\n".to_string();
    std::fs::write(&cfg_path, &edited).unwrap();
    config::ensure_scaffolded_in(dir);
    let after = std::fs::read_to_string(&cfg_path).unwrap();
    assert_eq!(after, edited, "second scaffold overwrote user edit");
}

#[test]
fn scaffold_skips_existing_agent_files() {
    let tmp = tempfile::tempdir().unwrap();
    let dir: &Path = tmp.path();
    let agents_dir = dir.join("agents");
    std::fs::create_dir_all(&agents_dir).unwrap();
    let custom = "# custom clarifier\n".to_string();
    std::fs::write(agents_dir.join("clarifier.md"), &custom).unwrap();
    config::ensure_scaffolded_in(dir);
    let after = std::fs::read_to_string(agents_dir.join("clarifier.md")).unwrap();
    assert_eq!(after, custom, "existing agent file was overwritten");
    assert!(
        agents_dir.join("intent-classifier.md").exists(),
        "other files still scaffolded"
    );
}

#[test]
fn map_langsmith_prefers_toml_service_name() {
    let mut cfg = muon::application::config::LangSmithConfig::default();
    cfg.service_name = "from-toml".into();
    let mapped = map_langsmith_config("muon", &cfg, "key".into());
    assert_eq!(mapped.service_name, "from-toml");
    assert_eq!(mapped.api_key, "key");
}

#[test]
fn map_langsmith_falls_back_service_when_empty() {
    let mut cfg = muon::application::config::LangSmithConfig::default();
    cfg.service_name.clear();
    let mapped = map_langsmith_config("muon", &cfg, "key".into());
    assert_eq!(mapped.service_name, "muon");
}
