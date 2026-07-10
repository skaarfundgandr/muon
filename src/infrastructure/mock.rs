use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(any(test, feature = "mock"))]
use crate::application::pipeline_runner::services::session_service::InMemorySessionStore;
use crate::domain::models::log_entry::AgentTag;
use crate::domain::traits::agent::MuonAgent;
use crate::domain::error::MuonError;
use crate::infrastructure::context::InfrastructureContext;

/// Trivial in-process mock agent. Returns a configured answer for every
/// prompt. Available only for tests and for downstream builds that opt in
/// via the `mock` Cargo feature.
pub struct MockAgent {
    tag: AgentTag,
    answer: String,
    calls: Arc<AtomicUsize>,
}

impl MockAgent {
    pub fn new(tag: AgentTag, answer: impl Into<String>) -> Self {
        Self {
            tag,
            answer: answer.into(),
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl MuonAgent for MockAgent {
    fn tag(&self) -> AgentTag {
        self.tag
    }

    async fn prompt_raw(&self, _prompt: &str) -> Result<String, MuonError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.answer.clone())
    }
}

impl InfrastructureContext {
    /// Build a context with deterministic mock agents. Test-only and
    /// downstream opt-in (Cargo feature `mock`); never used by production
    /// code paths. The persistent store is backed by `InMemorySessionStore`,
    /// not the real `DieselSessionStore`.
    #[cfg(any(test, feature = "mock"))]
    pub fn mock() -> Self {
        Self::new(
            Arc::new(MockAgent::new(
                AgentTag::Intent,
                r#"{"intent":"research","depth":"shallow"}"#,
            )),
            Arc::new(MockAgent::new(AgentTag::Search, "Mock shallow answer.")),
            Arc::new(MockAgent::new(
                AgentTag::Clarify,
                r#"{"needs_clarification":false,"clarification_question":""}"#,
            )),
            Arc::new(MockAgent::new(AgentTag::Orchestrate, "Mock deep report.")),
            Arc::new(MockAgent::new(AgentTag::Plan, "Mock plan.")),
            Arc::new(MockAgent::new(AgentTag::Search, "Mock researcher answer.")),
            Arc::new(InMemorySessionStore::new()),
        )
    }
}
