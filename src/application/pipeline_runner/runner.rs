use crate::application::bridge::BridgeChannels;
use crate::application::pipeline::{PipelineStage, PipelineState};
use crate::config::MuonConfig;
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::models::query::Intent;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::SessionId;
use crate::domain::traits::agent::MuonAgent;
use crate::error::MuonError;
use crate::infrastructure::context::InfrastructureContext;

use super::escalation;
use super::services::session_service::InMemorySessionStore;

pub async fn run_pipeline(
    query: &str,
    state: &mut PipelineState,
    session_id: SessionId,
    cfg: &MuonConfig,
    infra: &InfrastructureContext,
    bridge: &BridgeChannels,
) -> Result<ResearchReport, MuonError> {
    state.start();
    let _ = session_id;
    let _ = InMemorySessionStore::new;

    bridge.stage(PipelineStage::IntentClassification);
    bridge.log(AgentTag::Intent, LogLevel::Info, "classifying query");
    let intent = classify_intent(infra.intent_classifier.as_ref(), query).await?;

    match intent.intent {
        Intent::Meta(resp) => {
            bridge.stage(PipelineStage::Complete);
            state.finish();
            let report = ResearchReport::direct(&resp);
            bridge.log(
                AgentTag::Intent,
                LogLevel::Info,
                format!("meta response, report length {}", report.summary.len()),
            );
            Ok(report)
        }
        Intent::Research => match intent.depth {
            crate::domain::models::query::Depth::Shallow => {
                let report = shallow_research(infra.shallow.as_ref(), query, bridge).await?;
                if cfg.advanced.escalate_agent && escalation::should_escalate(&report) {
                    bridge.log(
                        AgentTag::Sys,
                        LogLevel::Info,
                        "shallow result triggered escalation",
                    );
                    run_deep_path(query, state, cfg, infra, bridge).await
                } else {
                    bridge.stage(PipelineStage::Complete);
                    state.finish();
                    Ok(report)
                }
            }
            crate::domain::models::query::Depth::Deep => {
                run_deep_path(query, state, cfg, infra, bridge).await
            }
        },
    }
}

async fn classify_intent(
    agent: &dyn MuonAgent,
    query: &str,
) -> Result<crate::domain::models::query::QueryIntent, MuonError> {
    let raw = agent.prompt_raw(query).await?;
    crate::domain::agents::intent_classifier::parse_intent(&raw)
}

async fn shallow_research(
    agent: &dyn MuonAgent,
    query: &str,
    bridge: &BridgeChannels,
) -> Result<ResearchReport, MuonError> {
    bridge.stage(PipelineStage::ShallowResearch);
    bridge.log(AgentTag::Search, LogLevel::Info, "running shallow research");
    let raw = agent.prompt_raw(query).await?;
    Ok(ResearchReport {
        title: "Shallow Research".to_string(),
        summary: raw,
        sections: Vec::new(),
        citations: Vec::new(),
        stats: Default::default(),
    })
}

async fn run_deep_path(
    query: &str,
    state: &mut PipelineState,
    cfg: &MuonConfig,
    infra: &InfrastructureContext,
    bridge: &BridgeChannels,
) -> Result<ResearchReport, MuonError> {
    bridge.stage(PipelineStage::Clarification);
    let clarifier_result =
        super::clarifier::run_clarifier(query, cfg, infra.clarifier.as_ref(), bridge).await?;
    let researcher =
        crate::application::deep_researcher::DeepResearcher::new(cfg, infra, bridge);
    let report = researcher.run(query, &clarifier_result).await?;
    bridge.stage(PipelineStage::Complete);
    state.finish();
    Ok(report)
}
