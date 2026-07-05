use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearcherSpec {
    pub model: String,
    pub provider: String,
}

impl ResearcherSpec {
    pub fn new(model: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            provider: provider.into(),
        }
    }
}

impl Default for ResearcherSpec {
    fn default() -> Self {
        Self {
            model: "glm-5.2-flex".to_string(),
            provider: "NeuralWatt".to_string(),
        }
    }
}
