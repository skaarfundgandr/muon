#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::pipeline_runner::services::session_service::InMemorySessionStore;
use muon::domain::models::session::SessionId;
use muon::domain::traits::session_store::SessionStore;

#[tokio::test]
async fn in_memory_store_delete_removes_session_and_associated_data() {
    let store = InMemorySessionStore::new();
    let id = store.create("test query").await.unwrap();
    let log = muon::domain::models::log_entry::LogEntry {
        timestamp: chrono::Utc::now(),
        agent_tag: muon::domain::models::log_entry::AgentTag::Sys,
        message: "test log".into(),
        level: muon::domain::models::log_entry::LogLevel::Info,
    };
    store.append_log(id, &log).await.unwrap();
    store.save_sources(id, &[muon::domain::models::source::Source::default()]).await.unwrap();
    store
        .save_report(
            id,
            &muon::domain::models::report::ResearchReport {
                title: "T".into(),
                summary: "S".into(),
                sections: Vec::new(),
                citations: Vec::new(),
                stats: Default::default(),
            },
        )
        .await
        .unwrap();

    store.delete(id).await.unwrap();

    assert!(store.get(id).await.unwrap().is_none());
    assert!(store.get_report(id).await.unwrap().is_none());
    assert!(store.get_sources(id).await.unwrap().is_empty());
}

#[tokio::test]
async fn session_service_remove_reactivates_first_session() {
    use muon::application::session::SessionService;
    use muon::application::session::SessionSummary;

    let mut svc = SessionService::new();
    let now = chrono::Utc::now();
    svc.replace_all(vec![
        SessionSummary { id: SessionId::new_v4(), title: "a".into(), query: "q1".into(), created_at: now, is_active: false },
        SessionSummary { id: SessionId::new_v4(), title: "b".into(), query: "q2".into(), created_at: now, is_active: true },
    ]);
    assert_eq!(svc.list().len(), 2);

    let removed = svc.remove(1);
    assert!(removed.is_some());
    assert_eq!(svc.list().len(), 1);
    assert_eq!(svc.active().map(|s| s.title.as_str()), Some("a"));

    let last_two = svc.remove(0);
    assert!(last_two.is_some());
    assert_eq!(svc.list().len(), 0);
    assert!(svc.active().is_none());
}
