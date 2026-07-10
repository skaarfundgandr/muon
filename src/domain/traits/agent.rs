use async_trait::async_trait;

use crate::domain::models::log_entry::AgentTag;
use crate::domain::error::MuonError;

#[async_trait]
pub trait MuonAgent: Send + Sync {
    fn tag(&self) -> AgentTag;
    async fn prompt_raw(&self, prompt: &str) -> Result<String, MuonError>;
}
