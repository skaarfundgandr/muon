use serde::{Deserialize, Serialize};

use crate::application::bridge::{AgentEvent, BridgeChannels, PlanDecision};
use crate::config::MuonConfig;
use crate::domain::agents::clarifier::{ClarifierResult, ClarifierState};
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::traits::agent::MuonAgent;
use crate::error::MuonError;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClarifyDecision {
    needs_clarification: bool,
    clarification_question: String,
}

fn build_q_prompt(query: &str, state: &ClarifierState) -> String {
    let prior = if state.clarifier_log.is_empty() {
        "None yet.".to_string()
    } else {
        state.clarifier_log.clone()
    };
    format!(
        "You are the \u{03BC}on clarifier.\n\
         User query: {query}\n\
         Prior Q/A: {prior}\n\
         Decide if a clarifying question is required. Respond with strict JSON:\n\
         {{\"needs_clarification\": bool, \"clarification_question\": string}}\n\
         Keep the question short and focused on the most blocking ambiguity."
    )
}

fn parse_clarify_json(text: &str) -> Result<ClarifyDecision, MuonError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(MuonError::Pipeline("clarifier returned empty".into()));
    }
    let value: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| {
        MuonError::Pipeline(format!("clarifier returned non-JSON: {e}; raw={trimmed}"))
    })?;
    let needs = value
        .get("needs_clarification")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| MuonError::Pipeline("missing needs_clarification".into()))?;
    let q = value
        .get("clarification_question")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(ClarifyDecision {
        needs_clarification: needs,
        clarification_question: q,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlanProposal {
    plan_title: String,
    plan_sections: Vec<String>,
}

fn build_plan_prompt(query: &str, state: &ClarifierState) -> String {
    let prior = if state.clarifier_log.is_empty() {
        "None".to_string()
    } else {
        state.clarifier_log.clone()
    };
    let feedback = if state.plan_feedback_history.is_empty() {
        "None".to_string()
    } else {
        state.plan_feedback_history.join("\n--\n")
    };
    format!(
        "You are the \u{03BC}on planner.\n\
         User query: {query}\n\
         Clarification log:\n{prior}\n\
         User feedback on prior plans:\n{feedback}\n\
         Produce a concise research plan. Respond with strict JSON:\n\
         {{\"plan_title\": string, \"plan_sections\": [string, ...]}}"
    )
}

fn parse_plan_json(text: &str) -> Result<PlanProposal, MuonError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(MuonError::Pipeline("planner returned empty".into()));
    }
    let value: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| {
        MuonError::Pipeline(format!("planner returned non-JSON: {e}; raw={trimmed}"))
    })?;
    let title = value
        .get("plan_title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| MuonError::Pipeline("missing plan_title".into()))?
        .to_string();
    let sections_value = value
        .get("plan_sections")
        .ok_or_else(|| MuonError::Pipeline("missing plan_sections".into()))?;
    let sections: Vec<String> = if let Some(arr) = sections_value.as_array() {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    } else if let Some(s) = sections_value.as_str() {
        s.lines().map(|l| l.trim().to_string()).collect()
    } else {
        return Err(MuonError::Pipeline("plan_sections wrong type".into()));
    };
    Ok(PlanProposal {
        plan_title: title,
        plan_sections: sections,
    })
}

pub async fn run_clarifier(
    query: &str,
    cfg: &MuonConfig,
    agent: &dyn MuonAgent,
    bridge: &BridgeChannels,
) -> Result<ClarifierResult, MuonError> {
    let mut state = ClarifierState::new(
        cfg.advanced.max_clarifier_turns as u32,
        cfg.advanced.plan_approval,
        cfg.advanced.max_plan_iterations as u32,
    );

    bridge.stage(crate::application::pipeline::PipelineStage::Clarification);

    while state.iteration < state.max_turns {
        let prompt = build_q_prompt(query, &state);
        let raw = agent.prompt_raw(&prompt).await?;
        let parsed = parse_clarify_json(&raw)?;
        if !parsed.needs_clarification || parsed.clarification_question.is_empty() {
            break;
        }
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        bridge.events.send(AgentEvent::ClarifierQuestion {
            question: parsed.clarification_question.clone(),
            responder: tx,
        })?;
        let answer = rx.await.map_err(|_| MuonError::Cancelled)?;
        state
            .clarifier_log
            .push_str(&format!("Q: {}\nA: {}\n", parsed.clarification_question, answer));
        state.iteration += 1;
    }

    if state.enable_plan_approval {
        let mut approved = false;
        for _ in 0..state.max_plan_iterations {
            let prompt = build_plan_prompt(query, &state);
            let raw = agent.prompt_raw(&prompt).await?;
            let proposal = match parse_plan_json(&raw) {
                Ok(p) => p,
                Err(e) => {
                    bridge.log(
                        AgentTag::Plan,
                        LogLevel::Warn,
                        format!("plan parse failed: {e}"),
                    );
                    continue;
                }
            };
            state.plan_title = Some(proposal.plan_title.clone());
            state.plan_sections = proposal.plan_sections.clone();

            let result = ClarifierResult {
                clarifier_log: state.clarifier_log.clone(),
                plan_title: Some(proposal.plan_title),
                plan_sections: proposal.plan_sections,
                plan_approved: false,
            };
            let (tx, rx) = tokio::sync::oneshot::channel::<PlanDecision>();
            bridge.events.send(AgentEvent::PlanProposed {
                plan: result,
                responder: tx,
            })?;
            match rx.await.map_err(|_| MuonError::Cancelled)? {
                PlanDecision::Approve => {
                    state.plan_approved = true;
                    approved = true;
                    break;
                }
                PlanDecision::Reject => {
                    state.plan_rejected = true;
                    break;
                }
                PlanDecision::Feedback(f) => state.plan_feedback_history.push(f),
            }
        }
        let _ = approved;
    }

    Ok(ClarifierResult {
        clarifier_log: state.clarifier_log,
        plan_title: state.plan_title,
        plan_sections: state.plan_sections,
        plan_approved: state.plan_approved,
    })
}
