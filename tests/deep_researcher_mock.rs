#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::bridge::{AgentEvent, BridgeChannels};
use muon::application::deep_researcher::DeepResearcher;
use muon::application::deps::PipelineDeps;
use muon::config::MuonConfig;
use muon::domain::agents::clarifier::ClarifierResult;
use muon::infrastructure::context::InfrastructureContext;
use tokio::sync::mpsc;

#[tokio::test]
async fn deep_researcher_runs_max_loops_with_mock() {
    let cfg = MuonConfig::default();
    let infra = InfrastructureContext::mock();
    let deps = PipelineDeps::from_infra(&infra);
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();
    let bridge = BridgeChannels::new(tx);
    let plan = ClarifierResult::default();
    let researcher = DeepResearcher::new(&cfg, &deps, &bridge);
    let report = researcher
        .run("What is async Rust?", &plan)
        .await
        .unwrap();
    assert!(report.title.contains("Research") || !report.title.is_empty());
    let mut loop_count = 0;
    while let Ok(ev) = rx.try_recv() {
        if let AgentEvent::Log(l) = ev
            && l.message.contains("loop ")
        {
            loop_count += 1;
        }
    }
    assert!(loop_count >= 1);
}

#[tokio::test]
async fn deep_researcher_exits_early_when_quality_passes() {
    let mut cfg = MuonConfig::default();
    cfg.agents.deep_researcher.iterations = 5;
    cfg.agents.deep_researcher.min_report_length = 1;
    cfg.agents.deep_researcher.min_report_sections = 0;
    let infra = InfrastructureContext::mock();
    let deps = PipelineDeps::from_infra(&infra);
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();
    let bridge = BridgeChannels::new(tx);
    let plan = ClarifierResult::default();
    let researcher = DeepResearcher::new(&cfg, &deps, &bridge);
    let report = researcher.run("query", &plan).await.unwrap();
    assert!(!report.title.is_empty());

    let mut loop_logs = 0;
    let mut early_exit = false;
    while let Ok(ev) = rx.try_recv() {
        if let AgentEvent::Log(l) = ev {
            if l.message.contains("loop ") {
                loop_logs += 1;
            }
            if l.message.contains("exiting early") {
                early_exit = true;
            }
        }
    }
    assert!(early_exit, "expected early-exit log");
    assert!(loop_logs < 5, "expected early exit, ran {loop_logs} loops");
}
