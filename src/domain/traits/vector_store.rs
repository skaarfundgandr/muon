use async_trait::async_trait;

use crate::domain::models::source::Source;
use crate::domain::error::MuonError;

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn add(&self, source: &Source, content: &str) -> Result<Option<String>, MuonError>;
    async fn query(&self, text: &str, k: usize) -> Result<Vec<Source>, MuonError>;
}
