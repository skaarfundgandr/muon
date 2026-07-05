use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::domain::models::log_entry::LogEntry;
use crate::error::MuonError;
use crate::infrastructure::models::log_entry_row::{LogEntryRow, NewLogEntryRow};
use crate::infrastructure::storage::pool::DbPool;
use crate::infrastructure::storage::schema::log_entries;

pub struct LogStore {
    pool: DbPool,
}

impl LogStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn append(&self, session_id: &str, log: LogEntry) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let new_row = NewLogEntryRow {
            session_id: session_id.to_string(),
            agent_tag: log.agent_tag.as_str().to_string(),
            message: log.message.clone(),
            level: log.level.as_str().to_string(),
            timestamp: log.timestamp.naive_utc(),
        };
        diesel::insert_into(log_entries::table)
            .values(&new_row)
            .execute(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn list_for_session(&self, session_id: &str) -> Result<Vec<LogEntry>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let rows: Vec<LogEntryRow> = log_entries::table
            .filter(log_entries::session_id.eq(session_id))
            .order(log_entries::timestamp.asc())
            .load(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        rows.into_iter()
            .map(LogEntry::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
