use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::domain::models::report::Citation;
use crate::domain::error::MuonError;
use crate::infrastructure::models::citation_row::{CitationRow, NewCitationRow};
use crate::infrastructure::storage::pool::DbPool;
use crate::infrastructure::storage::schema::citations;

pub struct CitationStore {
    pool: DbPool,
}

impl CitationStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn replace_for_report(
        &self,
        report_id: i32,
        citations: &[Citation],
    ) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        diesel::delete(citations::table.filter(citations::report_id.eq(report_id)))
            .execute(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let new_rows: Vec<NewCitationRow> = citations
            .iter()
            .map(|c| NewCitationRow::from_with_report(report_id, c))
            .collect();
        for row in new_rows {
            diesel::insert_into(citations::table)
                .values(&row)
                .execute(&mut *conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn list_for_report(&self, report_id: i32) -> Result<Vec<Citation>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let rows: Vec<CitationRow> = citations::table
            .filter(citations::report_id.eq(report_id))
            .order(citations::reference_number.asc())
            .load(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        rows.into_iter()
            .map(Citation::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}
