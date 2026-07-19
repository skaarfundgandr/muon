#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::bridge::BridgeChannels;
use muon::application::deps::PipelineDeps;
use muon::application::config::MuonConfig;
use muon::infrastructure::context::InfrastructureContext;

#[tokio::test]
async fn new_live_degrades_without_ready_providers() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let bridge = BridgeChannels::new(tx);

    // Process-global session pool: one path for the whole test binary scenario.
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(format!(
        "muon-empty-config-boot-{}.db",
        uuid::Uuid::new_v4()
    ));
    let _ = std::fs::remove_file(&temp_dir);

    // 1) Empty [[providers]] → stubs
    let mut cfg = MuonConfig::default();
    assert!(cfg.providers.is_empty());
    cfg.advanced.session_db_path = temp_dir.to_string_lossy().to_string();
    let infra = InfrastructureContext::new_live(&cfg, &bridge)
        .await
        .expect("new_live must succeed with empty providers");
    let deps = PipelineDeps::from_infra(&infra);
    match deps.intent_classifier.prompt_raw("test").await.unwrap_err() {
        muon::domain::error::MuonError::Config(_) => {}
        other => panic!("expected Config error, got {other:?}"),
    }

    // 2) Placeholder ${ENV} key unset → still boots via degrade (same pool path)
    cfg.providers.push(muon::application::config::ProviderConfig {
        name: "DeepSeek".into(),
        base_url: "https://api.deepseek.com/v1".into(),
        api_key: "${MUON_TEST_UNSET_API_KEY_XYZ}".into(),
        models: vec![],
        provider_type: muon::application::config::ProviderType::OpenAICompatible,
    });
    let infra = InfrastructureContext::new_live(&cfg, &bridge)
        .await
        .expect("new_live must degrade when provider env key is unset");
    let deps = PipelineDeps::from_infra(&infra);
    match deps.shallow.prompt_raw("test").await.unwrap_err() {
        muon::domain::error::MuonError::Config(_) => {}
        other => panic!("expected Config error from stub, got {other:?}"),
    }

    let _ = std::fs::remove_file(&temp_dir);
    let _ = std::fs::remove_file(temp_dir.with_extension("db-wal"));
    let _ = std::fs::remove_file(temp_dir.with_extension("db-shm"));
}
