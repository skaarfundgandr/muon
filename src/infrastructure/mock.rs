use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::application::pipeline_runner::services::session_service::InMemorySessionStore;
use crate::domain::models::log_entry::AgentTag;
use crate::domain::traits::agent::MuonAgent;
use crate::error::MuonError;
use crate::infrastructure::context::InfrastructureContext;

/// Trivial in-process mock agent. Returns a configured answer for every
/// prompt. Used by the Phase 3 mock verification path; replaced in Phase 4.
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
    /// Build a context with deterministic mock agents. Used in tests and
    /// during Phase 3 verification.
    pub fn mock() -> Self {
        Self::new(
            Box::new(MockAgent::new(
                AgentTag::Intent,
                r#"{"intent":"research","depth":"shallow"}"#,
            )),
            Box::new(MockAgent::new(
                AgentTag::Search,
                "Mock shallow answer.",
            )),
            Box::new(MockAgent::new(
                AgentTag::Clarify,
                r#"{"needs_clarification":false,"clarification_question":""}"#,
            )),
            Box::new(MockAgent::new(
                AgentTag::Orchestrate,
                "Mock deep report.",
            )),
            Box::new(MockAgent::new(
                AgentTag::Plan,
                "Mock plan.",
            )),
            Box::new(MockAgent::new(
                AgentTag::Search,
                "Mock researcher answer.",
            )),
            Box::new(InMemorySessionStore::new()),
        )
    }
}
