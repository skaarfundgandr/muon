#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use async_trait::async_trait;
use muon::domain::error::MuonError;
use muon::domain::models::log_entry::AgentTag;
use muon::domain::traits::agent::MuonAgent;

pub struct StubAgent {
    tag: AgentTag,
    answer: String,
}

impl StubAgent {
    pub fn new(tag: AgentTag, answer: impl Into<String>) -> Self {
        Self {
            tag,
            answer: answer.into(),
        }
    }
}

#[async_trait]
impl MuonAgent for StubAgent {
    fn tag(&self) -> AgentTag {
        self.tag
    }

    async fn prompt_raw(&self, _prompt: &str) -> Result<String, MuonError> {
        Ok(self.answer.clone())
    }
}
