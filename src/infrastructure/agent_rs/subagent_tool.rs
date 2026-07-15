use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::domain::error::MuonError;
use crate::domain::traits::agent::MuonAgent;

pub trait SubagentKind: Send + Sync + 'static {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}

pub struct PlannerKind;
impl SubagentKind for PlannerKind {
    const NAME: &'static str = "delegate_planner";
    const DESCRIPTION: &'static str = "Delegate to the Planner sub-agent to decompose the query into search sub-queries and a research plan with evidence. Args: { prompt }. Returns the planner's outline/plan.";
}

pub struct ResearcherKind;
impl SubagentKind for ResearcherKind {
    const NAME: &'static str = "delegate_researcher";
    const DESCRIPTION: &'static str = "Delegate to the Researcher sub-agent to execute a search sub-query and synthesize findings with citations. Args: { prompt }. Returns the researcher's findings block.";
}

#[derive(Debug, Deserialize)]
pub struct SubagentArgs {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct SubagentOutput {
    pub result: String,
}

pub struct SubagentTool<K: SubagentKind> {
    agent: Arc<dyn MuonAgent>,
    _marker: PhantomData<K>,
}

impl<K: SubagentKind> SubagentTool<K> {
    pub fn new(agent: Arc<dyn MuonAgent>) -> Self {
        Self {
            agent,
            _marker: PhantomData,
        }
    }
}

impl<K: SubagentKind> Tool for SubagentTool<K> {
    const NAME: &'static str = K::NAME;
    type Error = MuonError;
    type Args = SubagentArgs;
    type Output = SubagentOutput;

    fn description(&self) -> String {
        K::DESCRIPTION.to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "The prompt or sub-query to delegate." }
            },
            "required": ["prompt"]
        })
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + rig_core::wasm_compat::WasmCompatSend
    {
        let agent = Arc::clone(&self.agent);
        async move {
            let result = agent.prompt_raw(&args.prompt).await?;
            Ok(SubagentOutput { result })
        }
    }
}
