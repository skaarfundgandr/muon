use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerSpec {
    pub model: String,
    pub provider: String,
}

impl PlannerSpec {
    pub fn new(model: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            provider: provider.into(),
        }
    }
}

impl Default for PlannerSpec {
    fn default() -> Self {
        Self {
            model: String::new(),
            provider: String::new(),
        }
    }
}
