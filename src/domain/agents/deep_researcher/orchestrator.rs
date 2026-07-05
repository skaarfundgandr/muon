use serde::{Deserialize, Serialize};

use super::{planner::PlannerSpec, researcher::ResearcherSpec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearcherSpec {
    pub orchestrator_llm: OrchestratorLlm,
    pub planner_llm: Option<PlannerSpec>,
    pub researcher_llm: Option<ResearcherSpec>,
    pub max_loops: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorLlm {
    pub model: String,
    pub provider: String,
}

impl Default for DeepResearcherSpec {
    fn default() -> Self {
        Self {
            orchestrator_llm: OrchestratorLlm {
                model: "glm-5.2".to_string(),
                provider: "opencode-go".to_string(),
            },
            planner_llm: Some(PlannerSpec::default()),
            researcher_llm: Some(ResearcherSpec::default()),
            max_loops: 2,
        }
    }
}

impl DeepResearcherSpec {
    pub fn planner_llm(&self) -> PlannerSpec {
        self.planner_llm
            .clone()
            .unwrap_or_else(|| PlannerSpec {
                model: self.orchestrator_llm.model.clone(),
                provider: self.orchestrator_llm.provider.clone(),
            })
    }

    pub fn researcher_llm(&self) -> ResearcherSpec {
        self.researcher_llm
            .clone()
            .unwrap_or_else(|| ResearcherSpec {
                model: self.orchestrator_llm.model.clone(),
                provider: self.orchestrator_llm.provider.clone(),
            })
    }
}
