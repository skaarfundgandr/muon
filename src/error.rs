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
}

pub type Result<T> = std::result::Result<T, MuonError>;
