use serde::{Deserialize, Serialize};

use crate::domain::error::MuonError;
use crate::domain::models::query::{Depth, Intent, QueryIntent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassifierSpec {
    pub model: String,
    pub provider: String,
    pub timeout_sec: u64,
}

impl Default for IntentClassifierSpec {
    fn default() -> Self {
        Self {
            model: String::new(),
            provider: String::new(),
            timeout_sec: 90,
        }
    }
}

pub fn parse_intent(text: &str) -> Result<QueryIntent, MuonError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(MuonError::Pipeline(
            "intent classifier returned empty text".to_string(),
        ));
    }
    let json_str = crate::domain::extract_json(text).unwrap_or(text);
    let value: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        MuonError::Pipeline(format!(
            "intent classifier returned non-JSON: {e}; raw={trimmed}"
        ))
    })?;
    let intent_str = value
        .get("intent")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MuonError::Pipeline("missing `intent` field".to_string()))?;
    let intent = match intent_str {
        "research" => Intent::Research,
        "meta" => {
            let resp = value
                .get("response")
                .and_then(|v| v.as_str())
                .ok_or_else(|| MuonError::Pipeline("meta intent requires `response`".to_string()))?
                .to_string();
            Intent::Meta(resp)
        }
        other => {
            return Err(MuonError::Pipeline(format!(
                "unknown intent discriminant: {other}"
            )));
        }
    };
    let depth = match value.get("depth").and_then(|v| v.as_str()) {
        Some("shallow") | None => Depth::Shallow,
        Some("deep") => Depth::Deep,
        Some(other) => {
            return Err(MuonError::Pipeline(format!(
                "unknown depth discriminant: {other}"
            )));
        }
    };
    Ok(QueryIntent { intent, depth })
}
