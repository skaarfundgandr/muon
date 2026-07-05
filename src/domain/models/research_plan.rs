use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub title: String,
    pub sections: Vec<String>,
    pub approved: bool,
}
