use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::domain::error::MuonError;
use crate::domain::models::log_entry::LogEntry;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::SessionId;
use crate::domain::models::source::Source;
use crate::domain::traits::session_store::{SessionStore, SessionSummary};
use crate::infrastructure::models::session_row::{NewSessionRow, SessionRow};
use crate::infrastructure::models::source_row::NewSourceRow;
use crate::infrastructure::storage::pool::DbPool;
use crate::infrastructure::storage::schema::{
    citations, log_entries, research_reports, sessions, sources,
};

pub struct DieselSessionStore {
    pool: DbPool,
}

impl DieselSessionStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for DieselSessionStore {
    async fn create(&self, query: &str) -> Result<SessionId, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let id = Uuid::new_v4();
        let now = Utc::now().naive_utc();
        let new_row = NewSessionRow {
            id: id.to_string(),
            query: query.to_string(),
            status: "Pending".to_string(),
            pipeline_stage: "Idle".to_string(),
            plan_json: None,
            clarifier_result: None,
            telemetry_json: None,
            created_at: now,
            updated_at: now,
        };
        diesel::insert_into(sessions::table)
            .values(&new_row)
            .execute(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(id)
    }

    async fn create_with_id(&self, id: SessionId, query: &str) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let now = Utc::now().naive_utc();
        let new_row = NewSessionRow {
            id: id.to_string(),
            query: query.to_string(),
            status: "Pending".to_string(),
            pipeline_stage: "Idle".to_string(),
            plan_json: None,
            clarifier_result: None,
            telemetry_json: None,
            created_at: now,
            updated_at: now,
        };
        diesel::insert_into(sessions::table)
            .values(&new_row)
            .on_conflict(sessions::id)
            .do_nothing()
            .execute(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, id: SessionId) -> Result<Option<SessionSummary>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let row: Option<SessionRow> = sessions::table
            .find(id.to_string())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(row.map(SessionSummary::from))
    }

    async fn list(&self) -> Result<Vec<SessionSummary>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let rows: Vec<SessionRow> = sessions::table
            .order(sessions::created_at.desc())
            .load(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(rows.into_iter().map(SessionSummary::from).collect())
    }

    async fn update_stage(&self, id: SessionId, stage: &str) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let now = Utc::now().naive_utc();
        let status = match stage {
            "Idle" => "Pending",
            "IntentClassification" | "Clarification" => "Clarifying",
            "ShallowResearch" | "DeepResearch" | "CitationVerify" | "Report" => "Researching",
            "Complete" => "Complete",
            "Cancelled" => "Cancelled",
            "Failed" => "Failed",
            _ => "Researching",
        };
        diesel::update(sessions::table.find(id.to_string()))
            .set((
                sessions::pipeline_stage.eq(stage),
                sessions::status.eq(status),
                sessions::updated_at.eq(now),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(())
    }

    async fn save_report(&self, id: SessionId, report: &ResearchReport) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let content_json =
            serde_json::to_string(report).map_err(|e| MuonError::Database(e.to_string()))?;
        let stats_json =
            serde_json::to_string(&report.stats).map_err(|e| MuonError::Database(e.to_string()))?;
        let now = Utc::now().naive_utc();
        conn.transaction::<_, MuonError, _>(async |conn| {
            diesel::delete(
                research_reports::table
                    .filter(research_reports::session_id.eq(id.to_string())),
            )
            .execute(conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
            let report_row = crate::infrastructure::models::report_row::NewReportRow {
                session_id: id.to_string(),
                title: report.title.clone(),
                content: content_json.clone(),
                stats_json: stats_json.clone(),
                created_at: now,
            };
            diesel::insert_into(research_reports::table)
                .values(&report_row)
                .execute(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
            diesel::update(sessions::table.find(id.to_string()))
                .set((
                    sessions::status.eq("Complete"),
                    sessions::pipeline_stage.eq("Complete"),
                    sessions::updated_at.eq(now),
                ))
                .execute(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
            Ok(())
        })
        .await
    }

    async fn get_report(&self, id: SessionId) -> Result<Option<ResearchReport>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let row: Option<crate::infrastructure::models::report_row::ReportRow> =
            research_reports::table
                .filter(research_reports::session_id.eq(id.to_string()))
                .order(research_reports::created_at.desc())
                .first(&mut *conn)
                .await
                .optional()
                .map_err(|e| MuonError::Database(e.to_string()))?;
        row.map(ResearchReport::try_from).transpose()
    }

    async fn append_log(&self, id: SessionId, log: &LogEntry) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let new_row = crate::infrastructure::models::log_entry_row::NewLogEntryRow {
            session_id: id.to_string(),
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

    async fn save_sources(&self, id: SessionId, sources: &[Source]) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let sid = id.to_string();
        conn.transaction::<_, MuonError, _>(async |conn| {
            diesel::delete(sources::table.filter(sources::session_id.eq(&sid)))
                .execute(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
            for source in sources {
                let row = NewSourceRow::from_with_session(sid.clone(), source);
                diesel::insert_into(sources::table)
                    .values(&row)
                    .execute(conn)
                    .await
                    .map_err(|e| MuonError::Database(e.to_string()))?;
            }
            Ok(())
        })
        .await
    }

    async fn get_sources(&self, id: SessionId) -> Result<Vec<Source>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let rows: Vec<crate::infrastructure::models::source_row::SourceRow> = sources::table
            .filter(sources::session_id.eq(id.to_string()))
            .load(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        rows.into_iter()
            .map(Source::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn delete(&self, id: SessionId) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let sid = id.to_string();
        conn.transaction::<_, MuonError, _>(async |conn| {
            let report_ids: Vec<i32> = research_reports::table
                .filter(research_reports::session_id.eq(&sid))
                .select(research_reports::id)
                .load(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;

            if !report_ids.is_empty() {
                diesel::delete(citations::table.filter(citations::report_id.eq_any(&report_ids)))
                    .execute(conn)
                    .await
                    .map_err(|e| MuonError::Database(e.to_string()))?;
            }

            diesel::delete(research_reports::table.filter(research_reports::session_id.eq(&sid)))
                .execute(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
            diesel::delete(sources::table.filter(sources::session_id.eq(&sid)))
                .execute(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
            diesel::delete(log_entries::table.filter(log_entries::session_id.eq(&sid)))
                .execute(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
            diesel::delete(sessions::table.find(&sid))
                .execute(conn)
                .await
                .map_err(|e| MuonError::Database(e.to_string()))?;
            Ok(())
        })
        .await
    }

    async fn save_clarifier_outcome(
        &self,
        id: SessionId,
        plan_json: Option<&str>,
        clarifier_result_json: Option<&str>,
    ) -> Result<(), MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let now = Utc::now().naive_utc();
        diesel::update(sessions::table.find(id.to_string()))
            .set((
                sessions::plan_json.eq(plan_json),
                sessions::clarifier_result.eq(clarifier_result_json),
                sessions::updated_at.eq(now),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(())
    }

    async fn get_pipeline_stage(&self, id: SessionId) -> Result<Option<String>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let row: Option<SessionRow> = sessions::table
            .find(id.to_string())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(row.map(|r| r.pipeline_stage))
    }

    async fn get_clarifier_outcome(
        &self,
        id: SessionId,
    ) -> Result<Option<(Option<String>, Option<String>)>, MuonError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let row: Option<SessionRow> = sessions::table
            .find(id.to_string())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(row.map(|r| (r.plan_json, r.clarifier_result)))
    }
}
