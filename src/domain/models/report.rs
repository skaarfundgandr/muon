use serde::{Deserialize, Serialize};

use super::session::ReportStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub title: String,
    pub summary: String,
    pub sections: Vec<ReportSection>,
    pub citations: Vec<Citation>,
    pub stats: ReportStats,
}

impl ResearchReport {
    pub fn direct(text: &str) -> Self {
        Self {
            title: "Direct Answer".to_string(),
            summary: text.to_string(),
            sections: Vec::new(),
            citations: Vec::new(),
            stats: ReportStats::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub heading: String,
    pub body_markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub reference_number: u32,
    pub url: String,
    pub title: String,
    pub context_snippet: String,
    pub verification_level: VerificationLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationLevel {
    Exact,
    Prefix,
    ChildPath,
    QuerySubset,
}
