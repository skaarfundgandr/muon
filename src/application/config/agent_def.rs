use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for AgentDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            model: String::new(),
            provider: String::new(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            timeout_secs: default_timeout_secs(),
            preamble_markdown: String::new(),
        }
    }
}

fn default_temperature() -> f64 {
    0.0
}

fn default_max_tokens() -> u32 {
    6144
}

fn default_timeout_secs() -> u64 {
    60
}

/// In-memory set of the six pipeline agent definitions (YAML frontmatter SSOT).
#[derive(Debug, Clone, Default)]
pub struct AgentSettings {
    pub intent_classifier: AgentDef,
    pub clarifier: AgentDef,
    pub shallow_researcher: AgentDef,
    pub deep_orchestrator: AgentDef,
    pub planner: AgentDef,
    pub researcher: AgentDef,
}

impl AgentSettings {
    pub fn named_mut(&mut self, file_stem: &str) -> Option<&mut AgentDef> {
        match file_stem {
            "intent-classifier" => Some(&mut self.intent_classifier),
            "clarifier" => Some(&mut self.clarifier),
            "shallow-researcher" => Some(&mut self.shallow_researcher),
            "deep-orchestrator" => Some(&mut self.deep_orchestrator),
            "planner" => Some(&mut self.planner),
            "researcher" => Some(&mut self.researcher),
            _ => None,
        }
    }

    pub fn iter(&self) -> [(&str, &AgentDef); 6] {
        [
            ("intent-classifier", &self.intent_classifier),
            ("clarifier", &self.clarifier),
            ("shallow-researcher", &self.shallow_researcher),
            ("deep-orchestrator", &self.deep_orchestrator),
            ("planner", &self.planner),
            ("researcher", &self.researcher),
        ]
    }
}
