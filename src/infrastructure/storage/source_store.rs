use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::domain::models::source::Source;
use crate::domain::error::MuonError;
use crate::infrastructure::models::source_row::{NewSourceRow, SourceRow};
use crate::infrastructure::storage::pool::DbPool;
use crate::infrastructure::storage::schema::sources;

pub struct SourceStore {
    pool: DbPool,
}

impl SourceStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn bulk_insert(
        &self,
        session_id: &str,
        sources: &[Source],
    ) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        for source in sources {
            let row = NewSourceRow::from_with_session(session_id.to_string(), source);
            diesel::insert_into(sources::table)
                .values(&row)
                .execute(&mut *conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn list_for_session(&self, session_id: &str) -> Result<Vec<Source>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let rows: Vec<SourceRow> = sources::table
            .filter(sources::session_id.eq(session_id))
            .load(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        rows.into_iter()
            .map(Source::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
