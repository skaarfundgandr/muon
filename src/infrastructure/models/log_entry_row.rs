use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::error::MuonError;
use crate::domain::models::log_entry::{AgentTag, LogEntry, LogLevel};
use crate::infrastructure::storage::schema::log_entries;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = log_entries)]
pub struct LogEntryRow {
    pub id: i32,
    pub session_id: String,
    pub agent_tag: String,
    pub message: String,
    pub level: String,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = log_entries)]
pub struct NewLogEntryRow {
    pub session_id: String,
    pub agent_tag: String,
    pub message: String,
    pub level: String,
    pub timestamp: NaiveDateTime,
}

impl TryFrom<LogEntryRow> for LogEntry {
    type Error = MuonError;

    fn try_from(row: LogEntryRow) -> Result<Self, Self::Error> {
        let agent_tag = AgentTag::try_from_str(&row.agent_tag)?;
        let level = LogLevel::try_from_str(&row.level)?;
        let ts = row.timestamp.and_utc();
        Ok(Self {
            timestamp: ts,
            agent_tag,
            message: row.message,
            level,
        })
    }
}

impl From<&LogEntry> for NewLogEntryRow {
    fn from(entry: &LogEntry) -> Self {
        Self {
            session_id: String::new(),
            agent_tag: entry.agent_tag.as_str().to_string(),
            message: entry.message.clone(),
            level: entry.level.as_str().to_string(),
            timestamp: entry.timestamp.naive_utc(),
        }
    }
}

impl AgentTag {
    fn try_from_str(s: &str) -> Result<Self, MuonError> {
        match s {
            "intent" => Ok(Self::Intent),
            "clarify" => Ok(Self::Clarify),
            "plan" => Ok(Self::Plan),
            "search" => Ok(Self::Search),
            "extract" => Ok(Self::Extract),
            "verify" => Ok(Self::Verify),
            "orchestrate" => Ok(Self::Orchestrate),
            "sys" => Ok(Self::Sys),
            other => Err(MuonError::Database(format!("unknown agent tag: {other}"))),
        }
    }
}

impl LogLevel {
    fn try_from_str(s: &str) -> Result<Self, MuonError> {
        match s {
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            other => Err(MuonError::Database(format!("unknown log level: {other}"))),
        }
    }
}
