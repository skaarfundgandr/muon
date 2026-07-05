use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryIntent {
    pub intent: Intent,
    pub depth: Depth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    Meta(String),
    Research,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Depth {
    Shallow,
    Deep,
}
