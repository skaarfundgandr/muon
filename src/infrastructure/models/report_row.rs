use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::domain::models::report::{ReportSection, ResearchReport};
use crate::domain::models::session::ReportStats;
use crate::domain::error::MuonError;
use crate::infrastructure::storage::schema::research_reports;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = research_reports)]
pub struct ReportRow {
    pub id: i32,
    pub session_id: String,
    pub title: String,
    pub content: String,
    pub stats_json: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = research_reports)]
pub struct NewReportRow {
    pub session_id: String,
    pub title: String,
    pub content: String,
    pub stats_json: String,
    pub created_at: NaiveDateTime,
}

impl TryFrom<ReportRow> for ResearchReport {
    type Error = MuonError;

    fn try_from(row: ReportRow) -> Result<Self, Self::Error> {
        let sections: Vec<ReportSection> = serde_json::from_str(&row.content)
            .map_err(|e| MuonError::Database(e.to_string()))?;
        let citations = Vec::new();
        let stats: ReportStats = serde_json::from_str(&row.stats_json)
            .map_err(|e| MuonError::Database(e.to_string()))?;
        Ok(Self {
            title: row.title,
            summary: String::new(),
            sections,
            citations,
            stats,
        })
    }
}

impl From<&ResearchReport> for NewReportRow {
    fn from(report: &ResearchReport) -> Self {
        let content = serde_json::to_string(&report.sections).unwrap_or_default();
        let stats_json = serde_json::to_string(&report.stats).unwrap_or_default();
        Self {
            session_id: String::new(),
            title: report.title.clone(),
            content,
            stats_json,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
