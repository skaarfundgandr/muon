use diesel::prelude::*;

use crate::domain::models::report::{Citation, VerificationLevel};
use crate::error::MuonError;
use crate::infrastructure::storage::schema::citations;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = citations)]
pub struct CitationRow {
    pub id: i32,
    pub report_id: i32,
    pub reference_number: i32,
    pub url: String,
    pub title: String,
    pub context_snippet: String,
    pub verification_level: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = citations)]
pub struct NewCitationRow {
    pub report_id: i32,
    pub reference_number: i32,
    pub url: String,
    pub title: String,
    pub context_snippet: String,
    pub verification_level: String,
}

impl TryFrom<CitationRow> for Citation {
    type Error = MuonError;

    fn try_from(row: CitationRow) -> Result<Self, Self::Error> {
        let verification_level =
            VerificationLevel::try_from_str(&row.verification_level)?;
        Ok(Self {
            reference_number: row.reference_number as u32,
            url: row.url,
            title: row.title,
            context_snippet: row.context_snippet,
            verification_level,
        })
    }
}

impl NewCitationRow {
    pub fn from_with_report(report_id: i32, citation: &Citation) -> Self {
        Self {
            report_id,
            reference_number: citation.reference_number as i32,
            url: citation.url.clone(),
            title: citation.title.clone(),
            context_snippet: citation.context_snippet.clone(),
            verification_level: citation.verification_level.as_str().to_string(),
        }
    }
}

impl VerificationLevel {
    fn try_from_str(s: &str) -> Result<Self, MuonError> {
        match s {
            "Exact" => Ok(Self::Exact),
            "Prefix" => Ok(Self::Prefix),
            "ChildPath" => Ok(Self::ChildPath),
            "QuerySubset" => Ok(Self::QuerySubset),
            other => Err(MuonError::Database(format!(
                "unknown verification level: {other}"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "Exact",
            Self::Prefix => "Prefix",
            Self::ChildPath => "ChildPath",
            Self::QuerySubset => "QuerySubset",
        }
    }
}
