use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub relevance: f64,
    pub source_type: SourceType,
    pub verified: bool,
    pub verification_status: VerificationStatus,
    pub embedding_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    Web,
    Paper,
    Knowledge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Exact,
    Prefix,
    ChildPath,
    QuerySubset,
    Unverified,
    Removed,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            url: String::new(),
            title: String::new(),
            snippet: String::new(),
            relevance: 0.0,
            source_type: SourceType::Web,
            verified: false,
            verification_status: VerificationStatus::Unverified,
            embedding_id: None,
        }
    }
}
