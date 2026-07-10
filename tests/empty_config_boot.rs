#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::config::MuonConfig;
use muon::infrastructure::context::InfrastructureContext;
use muon::application::bridge::BridgeChannels;

#[tokio::test]
async fn new_live_with_empty_providers_returns_ok_and_stubs_agent_prompts() {
    let cfg = MuonConfig::default();
    assert!(cfg.providers.is_empty(), "default config must have no providers");

    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let bridge = BridgeChannels::new(tx);

    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(format!("muon-empty-config-boot-{}.db", uuid::Uuid::new_v4()));
    let _ = std::fs::remove_file(&temp_dir);

    let mut cfg = cfg;
    cfg.advanced.session_db_path = temp_dir.to_string_lossy().to_string();
    let infra = InfrastructureContext::new_live(&cfg, &bridge)
        .await
        .expect("new_live must succeed with empty providers");

    let err = infra.intent_classifier.prompt_raw("test").await.unwrap_err();
    match err {
        muon::domain::error::MuonError::Config(_) => {}
        other => panic!("expected Config error, got {other:?}"),
    }

    store::cleanup(&temp_dir);
}

mod store {
    use std::path::Path;
    pub fn cleanup(path: &Path) {
        let _ = std::fs::remove_file(path);
        let wal = path.with_extension("db-wal");
        let shm = path.with_extension("db-shm");
        let _ = std::fs::remove_file(wal);
        let _ = std::fs::remove_file(shm);
    }
}
