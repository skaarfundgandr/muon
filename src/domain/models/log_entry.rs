use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentTag {
    Intent,
    Clarify,
    Plan,
    Search,
    Extract,
    Verify,
    Orchestrate,
    Sys,
}

impl AgentTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Clarify => "clarify",
            Self::Plan => "plan",
            Self::Search => "search",
            Self::Extract => "extract",
            Self::Verify => "verify",
            Self::Orchestrate => "orchestrate",
            Self::Sys => "sys",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub agent_tag: AgentTag,
    pub message: String,
    pub level: LogLevel,
}
