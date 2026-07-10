use serde::{Deserialize, Serialize};

use crate::domain::models::research_plan::ResearchPlan;

#[derive(Debug, Clone, Default)]
pub struct ClarifierState {
    pub max_turns: u32,
    pub clarifier_log: String,
    pub iteration: u32,
    pub plan_title: Option<String>,
    pub plan_sections: Vec<String>,
    pub plan_approved: bool,
    pub plan_rejected: bool,
    pub plan_feedback_history: Vec<String>,
    pub enable_plan_approval: bool,
    pub max_plan_iterations: u32,
}

impl ClarifierState {
    pub fn new(max_turns: u32, enable_plan_approval: bool, max_plan_iterations: u32) -> Self {
        Self {
            max_turns,
            enable_plan_approval,
            max_plan_iterations,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClarifierResult {
    pub clarifier_log: String,
    pub plan_title: Option<String>,
    pub plan_sections: Vec<String>,
    pub plan_approved: bool,
}

impl ClarifierResult {
    pub fn to_plan(&self) -> Option<ResearchPlan> {
        let title = self.plan_title.clone()?;
        if title.is_empty() {
            return None;
        }
        Some(ResearchPlan {
            title,
            sections: self.plan_sections.clone(),
            approved: self.plan_approved,
        })
    }
}
