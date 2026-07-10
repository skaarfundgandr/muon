use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::models::log_entry::LogEntry;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::SessionId;
use crate::domain::models::source::Source;
use crate::domain::error::MuonError;

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: SessionId,
    pub query: String,
    pub created_at: DateTime<Utc>,
    pub title: String,
    pub is_active: bool,
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, query: &str) -> Result<SessionId, MuonError>;
    async fn create_with_id(&self, id: SessionId, query: &str) -> Result<(), MuonError>;
    async fn get(&self, id: SessionId) -> Result<Option<SessionSummary>, MuonError>;
    async fn list(&self) -> Result<Vec<SessionSummary>, MuonError>;
    async fn update_stage(&self, id: SessionId, stage: &str) -> Result<(), MuonError>;
    async fn save_report(&self, id: SessionId, report: &ResearchReport) -> Result<(), MuonError>;
    async fn get_report(&self, id: SessionId) -> Result<Option<ResearchReport>, MuonError>;
    async fn append_log(&self, id: SessionId, log: &LogEntry) -> Result<(), MuonError>;
    async fn save_sources(&self, id: SessionId, sources: &[Source]) -> Result<(), MuonError>;
    async fn get_sources(&self, id: SessionId) -> Result<Vec<Source>, MuonError>;
    async fn delete(&self, id: SessionId) -> Result<(), MuonError>;
}
