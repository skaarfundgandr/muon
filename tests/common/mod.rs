#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

pub mod stub_agent;

use std::sync::Arc;

use muon::domain::models::log_entry::AgentTag;
use muon::domain::traits::session_store::SessionStore;
use muon::infrastructure::context::InfrastructureContext;
use muon::infrastructure::storage::{open_pool, DieselSessionStore};

pub async fn diesel_store() -> (tempfile::TempDir, DieselSessionStore) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let path_str = path.to_string_lossy().to_string();
    let pool = open_pool(&path_str).await.unwrap();
    (dir, DieselSessionStore::new(pool))
}

pub async fn stub_infra() -> (tempfile::TempDir, InfrastructureContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let path_str = path.to_string_lossy().to_string();
    let pool = open_pool(&path_str).await.unwrap();
    let store: Arc<dyn SessionStore> = Arc::new(DieselSessionStore::new(pool));
    let infra = InfrastructureContext::new(
        Arc::new(stub_agent::StubAgent::new(
            AgentTag::Intent,
            r#"{"intent":"research","depth":"shallow"}"#,
        )),
        Arc::new(stub_agent::StubAgent::new(AgentTag::Search, "Mock shallow answer.")),
        Arc::new(stub_agent::StubAgent::new(
            AgentTag::Clarify,
            r#"{"needs_clarification":false,"clarification_question":""}"#,
        )),
        Arc::new(stub_agent::StubAgent::new(AgentTag::Orchestrate, "Mock deep report.")),
        Arc::new(stub_agent::StubAgent::new(AgentTag::Plan, "Mock plan.")),
        Arc::new(stub_agent::StubAgent::new(
            AgentTag::Search,
            "Mock researcher answer.",
        )),
        store,
    );
    (dir, infra)
}
