use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShallowResearcherSpec {
    pub model: String,
    pub provider: String,
    pub max_cycles: u32,
    pub tool_timeout_secs: u64,
}

impl Default for ShallowResearcherSpec {
    fn default() -> Self {
        Self {
            model: "glm-5.2".to_string(),
            provider: "opencode-go".to_string(),
            max_cycles: 10,
            tool_timeout_secs: 60,
        }
    }
}
