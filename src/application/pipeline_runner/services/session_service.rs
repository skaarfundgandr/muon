use async_trait::async_trait;

use crate::domain::models::log_entry::LogEntry;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::SessionId;
use crate::domain::models::source::Source;
use crate::domain::traits::session_store::{SessionStore, SessionSummary};
use crate::error::MuonError;

/// In-memory `SessionStore` retained exclusively for tests. The real
/// production implementation is `DieselSessionStore` in
/// `infrastructure::storage`, which persists sessions, sources, logs, and
/// reports to SQLite.
pub struct InMemorySessionStore {
    summaries: std::sync::Mutex<Vec<SessionSummary>>,
    stages: std::sync::Mutex<std::collections::HashMap<SessionId, String>>,
    reports: std::sync::Mutex<std::collections::HashMap<SessionId, ResearchReport>>,
    logs: std::sync::Mutex<std::collections::HashMap<SessionId, Vec<LogEntry>>>,
    sources: std::sync::Mutex<std::collections::HashMap<SessionId, Vec<Source>>>,
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self {
            summaries: std::sync::Mutex::new(Vec::new()),
            stages: std::sync::Mutex::new(std::collections::HashMap::new()),
            reports: std::sync::Mutex::new(std::collections::HashMap::new()),
            logs: std::sync::Mutex::new(std::collections::HashMap::new()),
            sources: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(&self, query: &str) -> Result<SessionId, MuonError> {
        let id = SessionId::new_v4();
        self.create_with_id(id, query).await?;
        Ok(id)
    }

    async fn create_with_id(&self, id: SessionId, query: &str) -> Result<(), MuonError> {
        let mut guard = self
            .summaries
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?;
        if guard.iter().any(|s| s.id == id) {
            return Ok(());
        }
        let title = query
            .split_whitespace()
            .take(5)
            .collect::<Vec<_>>()
            .join(" ");
        guard.push(SessionSummary {
            id,
            query: query.to_string(),
            created_at: chrono::Utc::now(),
            title,
            is_active: true,
        });
        Ok(())
    }

    async fn get(&self, id: SessionId) -> Result<Option<SessionSummary>, MuonError> {
        let guard = self
            .summaries
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?;
        Ok(guard.iter().find(|s| s.id == id).cloned())
    }

    async fn list(&self) -> Result<Vec<SessionSummary>, MuonError> {
        let guard = self
            .summaries
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?;
        Ok(guard.clone())
    }

    async fn update_stage(&self, id: SessionId, stage: &str) -> Result<(), MuonError> {
        self.stages
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?
            .insert(id, stage.to_string());
        Ok(())
    }

    async fn save_report(&self, id: SessionId, report: &ResearchReport) -> Result<(), MuonError> {
        self.reports
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?
            .insert(id, report.clone());
        Ok(())
    }

    async fn get_report(&self, id: SessionId) -> Result<Option<ResearchReport>, MuonError> {
        let guard = self
            .reports
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?;
        Ok(guard.get(&id).cloned())
    }

    async fn append_log(&self, id: SessionId, log: &LogEntry) -> Result<(), MuonError> {
        let mut guard = self
            .logs
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?;
        guard.entry(id).or_default().push(log.clone());
        Ok(())
    }

    async fn save_sources(&self, id: SessionId, sources: &[Source]) -> Result<(), MuonError> {
        self.sources
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?
            .insert(id, sources.to_vec());
        Ok(())
    }

    async fn get_sources(&self, id: SessionId) -> Result<Vec<Source>, MuonError> {
        let guard = self
            .sources
            .lock()
            .map_err(|e| MuonError::Session(format!("poisoned: {e}")))?;
        Ok(guard.get(&id).cloned().unwrap_or_default())
    }
}
