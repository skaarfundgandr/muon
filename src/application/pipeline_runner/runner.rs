use crate::application::bridge::BridgeChannels;
use crate::application::pipeline::{PipelineStage, PipelineState};
use crate::application::pipeline_runner::citation_verifier;
use crate::application::pipeline_runner::finalize::finalize_session;
use crate::config::MuonConfig;
use crate::domain::models::log_entry::{AgentTag, LogLevel};
use crate::domain::models::query::Intent;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::SessionId;
use crate::domain::models::source::SourceType;
use crate::domain::traits::agent::MuonAgent;
use crate::application::deps::PipelineDeps;
use crate::domain::error::MuonError;
use crate::infrastructure::source_registry::SourceRegistry;

use super::escalation;

fn snapshot_sources(deps: &PipelineDeps) -> Vec<crate::domain::models::source::Source> {
    if let Ok(sink) = deps.source_sink.lock() {
        sink.sources().to_vec()
    } else {
        Vec::new()
    }
}

pub async fn run_pipeline(
    query: &str,
    state: &mut PipelineState,
    cfg: &MuonConfig,
    deps: &PipelineDeps,
    bridge: &BridgeChannels,
    session_id: Option<SessionId>,
) -> Result<ResearchReport, MuonError> {
    state.start();
    if let Ok(mut sink) = deps.source_sink.lock() {
        sink.clear();
    }
    let session_id = match session_id {
        Some(id) => {
            deps.session_store.create_with_id(id, query).await?;
            id
        }
        None => deps.session_store.create(query).await?,
    };

    match run_pipeline_inner(query, state, cfg, deps, bridge, session_id).await {
        Ok(report) => Ok(report),
        Err(e) => {
            mark_session_terminal(deps, bridge, session_id, &e).await;
            Err(e)
        }
    }
}

async fn run_pipeline_inner(
    query: &str,
    state: &mut PipelineState,
    cfg: &MuonConfig,
    deps: &PipelineDeps,
    bridge: &BridgeChannels,
    session_id: SessionId,
) -> Result<ResearchReport, MuonError> {
    bridge.stage(PipelineStage::IntentClassification);
    bridge.log(AgentTag::Intent, LogLevel::Info, "classifying query");
    let intent = classify_intent(deps.intent_classifier.as_ref(), query).await?;

    let report = match intent.intent {
        Intent::Meta(resp) => {
            bridge.log(
                AgentTag::Intent,
                LogLevel::Info,
                "meta response".to_string(),
            );
            ResearchReport::direct(&resp)
        }
        Intent::Research => match intent.depth {
            crate::domain::models::query::Depth::Shallow => {
                let report =
                    shallow_research(deps.shallow.as_ref(), query, cfg, bridge, deps).await?;
                if cfg.advanced.escalate_agent && escalation::should_escalate(&report) {
                    bridge.log(
                        AgentTag::Sys,
                        LogLevel::Info,
                        "shallow result triggered escalation",
                    );
                    return run_deep_path(query, state, cfg, deps, bridge, session_id).await;
                }
                report
            }
            crate::domain::models::query::Depth::Deep => {
                return run_deep_path(query, state, cfg, deps, bridge, session_id).await;
            }
        },
    };

    let sources = snapshot_sources(deps);
    finalize_session(
        deps.session_store.as_ref(),
        session_id,
        &report,
        &sources,
        PipelineStage::Complete,
    )
    .await?;
    bridge.stage(PipelineStage::Complete);
    state.finish();
    bridge.log(
        AgentTag::Sys,
        LogLevel::Info,
        format!("session finalized, report length {}", report.summary.len()),
    );
    Ok(report)
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
    cfg: &MuonConfig,
    bridge: &BridgeChannels,
    deps: &PipelineDeps,
) -> Result<ResearchReport, MuonError> {
    bridge.stage(PipelineStage::ShallowResearch);
    bridge.log(AgentTag::Search, LogLevel::Info, "running shallow research");
    let start = std::time::Instant::now();
    let raw = agent.prompt_raw(query).await?;

    let mut registry = SourceRegistry::new();
    let urls = citation_verifier::extract_urls(&raw)?;
    for url in &urls {
        registry.record(url.as_str(), SourceType::Web);
    }
    if let Ok(sink) = deps.source_sink.lock() {
        for src in sink.sources() {
            registry.record(&src.url, src.source_type);
        }
    }

    let (summary, sections) = crate::application::services::report_builder::split_sections(&raw);
    let citations: Vec<crate::domain::models::report::Citation> = registry
        .sources()
        .iter()
        .enumerate()
        .map(|(i, s)| crate::domain::models::report::Citation {
            reference_number: (i + 1) as u32,
            url: s.url.clone(),
            title: s.title.clone(),
            context_snippet: s.snippet.clone(),
            verification_level: crate::domain::models::report::VerificationLevel::Exact,
        })
        .collect();
    let total_sources = registry.sources().len();
    let unverified = ResearchReport {
        title: "Shallow Research".to_string(),
        summary,
        sections,
        citations,
        stats: crate::domain::models::session::ReportStats {
            total_sources,
            ..crate::domain::models::session::ReportStats::default()
        },
    };

    let registry_urls = registry.urls();
    bridge.stage(PipelineStage::CitationVerify);
    let verified = if cfg.agents.deep_researcher.citation_verify {
        citation_verifier::verify(&unverified, &registry_urls, &[])?
    } else {
        citation_verifier::VerificationOutput {
            verified_report: raw.clone(),
            removed_citations: Vec::new(),
            valid_citations: registry
                .sources()
                .iter()
                .map(|s| citation_verifier::ValidCitation {
                    url: s.url.clone(),
                    level: crate::domain::models::report::VerificationLevel::Exact,
                })
                .collect(),
        }
    };

    let elapsed = start.elapsed().as_secs();
    let plan = crate::domain::agents::clarifier::ClarifierResult::default();
    bridge.stage(PipelineStage::Report);
    let final_report = crate::application::services::report_builder::build(verified, &plan, elapsed)?;
    bridge.log(
        AgentTag::Verify,
        LogLevel::Info,
        format!(
            "shallow citation verification: {} verified, {} removed",
            final_report.stats.verified_sources, final_report.stats.removed_citations
        ),
    );
    Ok(final_report)
}

async fn run_deep_path(
    query: &str,
    state: &mut PipelineState,
    cfg: &MuonConfig,
    deps: &PipelineDeps,
    bridge: &BridgeChannels,
    session_id: crate::domain::models::session::SessionId,
) -> Result<ResearchReport, MuonError> {
    bridge.stage(PipelineStage::Clarification);
    let clarifier_result =
        super::clarifier::run_clarifier(query, cfg, deps.clarifier.as_ref(), bridge).await?;
    let researcher = crate::application::deep_researcher::DeepResearcher::new(cfg, deps, bridge);
    let report = researcher.run(query, &clarifier_result).await?;
    let sources = snapshot_sources(deps);
    finalize_session(
        deps.session_store.as_ref(),
        session_id,
        &report,
        &sources,
        PipelineStage::Complete,
    )
    .await?;
    bridge.stage(PipelineStage::Complete);
    state.finish();
    Ok(report)
}

async fn mark_session_terminal(
    deps: &PipelineDeps,
    bridge: &BridgeChannels,
    session_id: SessionId,
    err: &MuonError,
) {
    let stage = if matches!(err, MuonError::Cancelled) {
        PipelineStage::Cancelled
    } else {
        PipelineStage::Failed
    };
    if deps
        .session_store
        .update_stage(session_id, stage.as_str())
        .await
        .is_err()
    {
        bridge.log(
            AgentTag::Sys,
            LogLevel::Warn,
            format!("failed to persist terminal stage for session {session_id}"),
        );
    }
}
