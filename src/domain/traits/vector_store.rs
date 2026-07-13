use async_trait::async_trait;

use crate::domain::error::MuonError;
use crate::domain::models::source::Source;

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn add(&self, source: &Source, content: &str) -> Result<Option<String>, MuonError>;
    async fn query(&self, text: &str, k: usize) -> Result<Vec<Source>, MuonError>;
    /// Persist the turbovec ANN index to disk (SQLite is already durable per write).
    async fn save_index(&self) -> Result<(), MuonError>;
}
