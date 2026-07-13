use async_trait::async_trait;

use crate::domain::error::MuonError;
use crate::domain::models::source::Source;

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn search(&self, query: &str, max: usize) -> Result<Vec<Source>, MuonError>;
    fn provider_name(&self) -> &'static str;
}
