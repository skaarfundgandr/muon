use diesel::result::Error as DieselError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MuonError {
    #[error("config error: {0}")]
    Config(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("terminal error: {0}")]
    Terminal(String),

    #[error("render error: {0}")]
    Render(String),

    #[error("cancelled")]
    Cancelled,

    #[error("agent {agent} failed: {message}")]
    Agent { agent: String, message: String },

    #[error("search provider {provider} failed: {message}")]
    Search { provider: String, message: String },

    #[error("database error: {0}")]
    Database(String),

    #[error("session error: {0}")]
    Session(String),

    #[error("timeout: agent {agent}")]
    Timeout { agent: String },

    #[error("pipeline error: {0}")]
    Pipeline(String),

    #[error("agent {agent} exceeded max cycles ({cycles})")]
    MaxCycles { agent: String, cycles: usize },
}

pub type Result<T> = std::result::Result<T, MuonError>;

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for MuonError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::Pipeline("agent event channel closed".to_string())
    }
}

impl From<agent_rs::domain::errors::ReActError> for MuonError {
    fn from(e: agent_rs::domain::errors::ReActError) -> Self {
        match e {
            agent_rs::domain::errors::ReActError::MaxCyclesExceeded { cycles } => Self::MaxCycles {
                agent: "unknown".to_string(),
                cycles,
            },
            other => Self::Agent {
                agent: "unknown".to_string(),
                message: other.to_string(),
            },
        }
    }
}

impl From<DieselError> for MuonError {
    fn from(e: DieselError) -> Self {
        Self::Database(e.to_string())
    }
}

impl From<anyhow::Error> for MuonError {
    fn from(e: anyhow::Error) -> Self {
        Self::Pipeline(e.to_string())
    }
}
