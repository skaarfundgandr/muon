use async_trait::async_trait;

use crate::domain::error::MuonError;
use crate::domain::models::log_entry::AgentTag;
use crate::domain::traits::agent::MuonAgent;

pub struct ConfigRequiredAgent {
    tag: AgentTag,
}

impl ConfigRequiredAgent {
    pub fn new(tag: AgentTag) -> Self {
        Self { tag }
    }
}

#[async_trait]
impl MuonAgent for ConfigRequiredAgent {
    fn tag(&self) -> AgentTag {
        self.tag
    }

    async fn prompt_raw(&self, _prompt: &str) -> Result<String, MuonError> {
        Err(MuonError::Config(
            "no [[providers]] configured — add at least one via Settings → Providers".into(),
        ))
    }
}
