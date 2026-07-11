#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::pipeline_runner::services::session_service::InMemorySessionStore;
use muon::domain::models::session::SessionId;
use muon::domain::traits::session_store::SessionStore;

#[tokio::test]
async fn update_stage_writes_status_column_for_terminal_stages() {
    let store = InMemorySessionStore::new();
    let id = store.create("test").await.unwrap();

    store.update_stage(id, "Clarification").await.unwrap();
    assert_eq!(store.get_stage(id).as_deref(), Some("Clarification"));

    for terminal in ["Complete", "Cancelled", "Failed"] {
        store.update_stage(id, terminal).await.unwrap();
        assert_eq!(store.get_stage(id).as_deref(), Some(terminal));
    }
}

#[tokio::test]
async fn cancelled_status_round_trips_through_inmemory_store() {
    let store = InMemorySessionStore::new();
    let id: SessionId = uuid::Uuid::new_v4();
    store.create_with_id(id, "round-trip").await.unwrap();
    store.update_stage(id, "Cancelled").await.unwrap();
    let got = store.get(id).await.unwrap().unwrap();
    assert_eq!(got.id, id);
    assert_eq!(store.get_stage(id).as_deref(), Some("Cancelled"));
}
