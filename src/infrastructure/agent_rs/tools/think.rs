use std::future::Future;

use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

const NAME: &str = "think";

#[derive(Debug, Deserialize)]
pub struct ThinkArgs {
    pub thought: String,
}

#[derive(Debug, Serialize)]
pub struct ThinkOutput {
    pub thought: String,
    pub acknowledged: bool,
}

pub struct ThinkTool;

impl Tool for ThinkTool {
    const NAME: &'static str = NAME;
    type Error = std::convert::Infallible;
    type Args = ThinkArgs;
    type Output = ThinkOutput;

    fn description(&self) -> String {
        "Use this tool to think through complex reasoning before acting. Records the thought in the agent's working memory.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "thought": { "type": "string", "description": "The reasoning or plan to record." }
            },
            "required": ["thought"]
        })
    }

    fn call(
        &self,
        args: Self::Args,
    ) -> impl Future<Output = Result<Self::Output, Self::Error>> + rig_core::wasm_compat::WasmCompatSend
    {
        let thought = args.thought;
        std::future::ready(Ok(ThinkOutput {
            thought,
            acknowledged: true,
        }))
    }
}
