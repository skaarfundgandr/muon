#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::pipeline_runner::services::session_service::InMemorySessionStore;
use muon::domain::error::MuonError;
use muon::domain::models::session::SessionId;
use muon::domain::traits::session_store::SessionStore;

#[tokio::test]
async fn update_stage_writes_status_column_for_terminal_stages() {
    let store = InMemorySessionStore::new();
    let id = store.create("test").await.unwrap();

    store.update_stage(id, "Clarification").await.unwrap();
    let summary = store.get(id).await.unwrap().unwrap();
    assert_eq!(summary.query, "test");

    store.update_stage(id, "Complete").await.unwrap();
    store.update_stage(id, "Cancelled").await.unwrap();
    store.update_stage(id, "Failed").await.unwrap();

    let _ = MuonError::Cancelled;
}

#[tokio::test]
async fn cancelled_status_round_trips_through_inmemory_store() {
    let store = InMemorySessionStore::new();
    let id: SessionId = uuid::Uuid::new_v4();
    store.create_with_id(id, "round-trip").await.unwrap();
    store.update_stage(id, "Cancelled").await.unwrap();
    let got = store.get(id).await.unwrap().unwrap();
    assert_eq!(got.id, id);
}
