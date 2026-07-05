use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::domain::models::report::ResearchReport;
use crate::error::MuonError;
use crate::infrastructure::models::report_row::{NewReportRow, ReportRow};
use crate::infrastructure::storage::pool::DbPool;
use crate::infrastructure::storage::schema::research_reports;

pub struct ReportStore {
    pool: DbPool,
}

impl ReportStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn save(
        &self,
        session_id: &str,
        report: &ResearchReport,
    ) -> Result<i32, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let content_json = serde_json::to_string(report)
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let stats_json = serde_json::to_string(&report.stats)
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let now = Utc::now().naive_utc();
        let new_row = NewReportRow {
            session_id: session_id.to_string(),
            title: report.title.clone(),
            content: content_json,
            stats_json,
            created_at: now,
        };
        let row: ReportRow = diesel::insert_into(research_reports::table)
            .values(&new_row)
            .get_result(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(row.id)
    }

    pub async fn get_for_session(
        &self,
        session_id: &str,
    ) -> Result<Option<ResearchReport>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let row: Option<ReportRow> = research_reports::table
            .filter(research_reports::session_id.eq(session_id))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| MuonError::Database(e.to_string()))?;
        row.map(ResearchReport::try_from)
            .transpose()
    }
}
