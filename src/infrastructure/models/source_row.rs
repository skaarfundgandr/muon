use diesel::prelude::*;

use crate::domain::models::source::{Source, SourceType, VerificationStatus};
use crate::error::MuonError;
use crate::infrastructure::storage::schema::sources;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = sources)]
pub struct SourceRow {
    pub id: i32,
    pub session_id: String,
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub relevance: f64,
    pub source_type: String,
    pub verification_status: String,
    pub embedding_id: Option<String>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = sources)]
pub struct NewSourceRow {
    pub session_id: String,
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub relevance: f64,
    pub source_type: String,
    pub verification_status: String,
    pub embedding_id: Option<String>,
}

impl TryFrom<SourceRow> for Source {
    type Error = MuonError;

    fn try_from(row: SourceRow) -> Result<Self, Self::Error> {
        let source_type = SourceType::try_from_str(&row.source_type)?;
        let verification_status =
            VerificationStatus::try_from_str(&row.verification_status)?;
        Ok(Self {
            url: row.url,
            title: row.title,
            snippet: row.snippet,
            relevance: row.relevance,
            source_type,
            verified: row.verification_status != "Unverified"
                && row.verification_status != "Removed",
            verification_status,
            embedding_id: row.embedding_id,
        })
    }
}

impl NewSourceRow {
    pub fn from_with_session(session_id: String, source: &Source) -> Self {
        Self {
            session_id,
            url: source.url.clone(),
            title: source.title.clone(),
            snippet: source.snippet.clone(),
            relevance: source.relevance,
            source_type: source.source_type.as_str().to_string(),
            verification_status: source.verification_status.as_str().to_string(),
            embedding_id: source.embedding_id.clone(),
        }
    }
}

impl SourceType {
    fn try_from_str(s: &str) -> Result<Self, MuonError> {
        match s {
            "Web" => Ok(Self::Web),
            "Paper" => Ok(Self::Paper),
            "Code" => Ok(Self::Code),
            "Enterprise" => Ok(Self::Enterprise),
            "Knowledge" => Ok(Self::Knowledge),
            other => Err(MuonError::Database(format!(
                "unknown source type: {other}"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "Web",
            Self::Paper => "Paper",
            Self::Code => "Code",
            Self::Enterprise => "Enterprise",
            Self::Knowledge => "Knowledge",
        }
    }
}

impl VerificationStatus {
    fn try_from_str(s: &str) -> Result<Self, MuonError> {
        match s {
            "Exact" => Ok(Self::Exact),
            "Prefix" => Ok(Self::Prefix),
            "ChildPath" => Ok(Self::ChildPath),
            "QuerySubset" => Ok(Self::QuerySubset),
            "Unverified" => Ok(Self::Unverified),
            "Removed" => Ok(Self::Removed),
            other => Err(MuonError::Database(format!(
                "unknown verification status: {other}"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::Prefix => "Prefix",
            Self::ChildPath => "ChildPath",
            Self::QuerySubset => "QuerySubset",
            Self::Unverified => "Unverified",
            Self::Removed => "Removed",
        }
    }
}
