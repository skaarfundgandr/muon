use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::models::session::{Session, SessionId, SessionStatus};
use crate::domain::traits::session_store::SessionSummary;
use crate::domain::error::MuonError;
use crate::infrastructure::storage::schema::sessions;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = sessions)]
pub struct SessionRow {
    pub id: String,
    pub query: String,
    pub status: String,
    pub pipeline_stage: String,
    pub plan_json: Option<String>,
    pub clarifier_result: Option<String>,
    pub telemetry_json: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSessionRow {
    pub id: String,
    pub query: String,
    pub status: String,
    pub pipeline_stage: String,
    pub plan_json: Option<String>,
    pub clarifier_result: Option<String>,
    pub telemetry_json: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl TryFrom<SessionRow> for Session {
    type Error = MuonError;

    fn try_from(row: SessionRow) -> Result<Self, Self::Error> {
        let id: SessionId = row
            .id
            .parse()
            .map_err(|e: uuid::Error| MuonError::Database(e.to_string()))?;
        let status = SessionStatus::parse_status(&row.status)?;
        let pipeline_stage =
            crate::domain::models::pipeline::PipelineStage::parse_stage(&row.pipeline_stage)?;
        let stats = row
            .telemetry_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|e| MuonError::Database(e.to_string()))?
            .unwrap_or_default();
        Ok(Self {
            id,
            query: row.query,
            status,
            pipeline_stage,
            sources: Vec::new(),
            report: None,
            logs: Vec::new(),
            stats,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
        })
    }
}

impl From<SessionRow> for SessionSummary {
    fn from(row: SessionRow) -> Self {
        let query = row.query;
        let title: String = {
            let words: Vec<&str> = query.split_whitespace().take(5).collect();
            let t = words.join(" ");
            if t.is_empty() {
                "Untitled Session".to_string()
            } else if t.len() > 40 {
                format!("{}...", &t[..37])
            } else {
                t
            }
        };
        Self {
            id: uuid::Uuid::parse_str(&row.id).unwrap_or_default(),
            created_at: row.created_at.and_utc(),
            title,
            query,
            is_active: false,
        }
    }
}

impl From<&Session> for NewSessionRow {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.to_string(),
            query: session.query.clone(),
            status: session.status.as_str().to_string(),
            pipeline_stage: session.pipeline_stage.as_str().to_string(),
            plan_json: None,
            clarifier_result: None,
            telemetry_json: serde_json::to_string(&session.stats).ok(),
            created_at: session.created_at.naive_utc(),
            updated_at: session.updated_at.naive_utc(),
        }
    }
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Clarifying => "Clarifying",
            Self::Researching => "Researching",
            Self::Complete => "Complete",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
        }
    }

    pub fn parse_status(s: &str) -> Result<Self, MuonError> {
        match s {
            "Pending" => Ok(Self::Pending),
            "Clarifying" => Ok(Self::Clarifying),
            "Researching" => Ok(Self::Researching),
            "Complete" => Ok(Self::Complete),
            "Cancelled" => Ok(Self::Cancelled),
            "Failed" => Ok(Self::Failed),
            other => Err(MuonError::Database(format!(
                "unknown session status: {other}"
            ))),
        }
    }
}


