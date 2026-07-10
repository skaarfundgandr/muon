use std::sync::Arc;
use std::time::Duration;

use muon::application::bridge::{AgentEvent, BridgeChannels};
use muon::application::deps::PipelineDeps;
use muon::application::pipeline::{PipelineStage, PipelineState};
use muon::application::pipeline_runner::run_pipeline;
use muon::config::MuonConfig;
use muon::domain::models::log_entry::AgentTag;
use muon::infrastructure::context::InfrastructureContext;
use muon::infrastructure::mock::MockAgent;

fn collect_events() -> (BridgeChannels, tokio::sync::mpsc::UnboundedReceiver<AgentEvent>) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<AgentEvent>();
    (BridgeChannels::new(tx), rx)
}

#[tokio::test(flavor = "current_thread")]
async fn pipeline_completes_for_shallow_intent() -> Result<(), Box<dyn std::error::Error>> {
    let (bridge, mut rx) = collect_events();
    let mut state = PipelineState::default();
    let cfg = MuonConfig::default();
    let infra = InfrastructureContext::mock();
    let deps = PipelineDeps::from_infra(&infra);

    let report = run_pipeline(
        "what is rust?",
        &mut state,
        &cfg,
        &deps,
        &bridge,
        None,
    )
    .await?;
    assert!(!report.summary.is_empty());

    let mut saw_complete = false;
    let mut saw_shallow = false;
    let timeout = std::time::Instant::now() + Duration::from_secs(2);
    while std::time::Instant::now() < timeout {
        match rx.try_recv() {
            Ok(AgentEvent::StageChanged(PipelineStage::Complete)) => saw_complete = true,
            Ok(AgentEvent::StageChanged(PipelineStage::ShallowResearch)) => saw_shallow = true,
            Ok(AgentEvent::StageChanged(_)) => {}
            Ok(AgentEvent::Log(_)) => {}
            Ok(_) => {}
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
        }
    }
    assert!(saw_complete, "expected Complete stage event");
    assert!(saw_shallow, "expected ShallowResearch stage event");
    assert_eq!(state.stage, PipelineStage::Complete);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn pipeline_meta_intent_returns_direct() -> Result<(), Box<dyn std::error::Error>> {
    let (bridge, _rx) = collect_events();
    let mut state = PipelineState::default();
    let cfg = MuonConfig::default();
    let mut infra = InfrastructureContext::mock();
    infra.intent_classifier = Arc::new(MockAgent::new(
        AgentTag::Intent,
        r#"{"intent":"meta","response":"hi there"}"#,
    ));
    let deps = PipelineDeps::from_infra(&infra);

    let report = run_pipeline(
        "hello",
        &mut state,
        &cfg,
        &deps,
        &bridge,
        None,
    )
    .await?;
    assert_eq!(report.title, "Direct Answer");
    assert_eq!(report.summary, "hi there");
    assert_eq!(state.stage, PipelineStage::Complete);
    Ok(())
}
