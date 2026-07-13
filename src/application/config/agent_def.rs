use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentDef {
    pub name: String,
    pub model: String,
    pub provider: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(skip)]
    pub preamble_markdown: String,
}

fn default_temperature() -> f64 {
    0.0
}

fn default_max_tokens() -> u32 {
    2048
}

fn default_timeout_secs() -> u64 {
    60
}
