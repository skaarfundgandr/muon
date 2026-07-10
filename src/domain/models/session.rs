use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{log_entry::LogEntry, report::ResearchReport, source::Source};
use super::pipeline::PipelineStage;

pub type SessionId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SessionStatus {
    #[default]
    Pending,
    Clarifying,
    Researching,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportStats {
    pub total_sources: usize,
    pub verified_sources: usize,
    pub removed_citations: usize,
    pub elapsed_secs: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub query: String,
    pub status: SessionStatus,
    pub pipeline_stage: PipelineStage,
    pub sources: Vec<Source>,
    pub report: Option<ResearchReport>,
    pub logs: Vec<LogEntry>,
    pub stats: ReportStats,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
