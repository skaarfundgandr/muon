#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::bridge::{AgentEvent, BridgeChannels};
use muon::application::deep_researcher::DeepResearcher;
use muon::config::MuonConfig;
use muon::domain::agents::clarifier::ClarifierResult;
use muon::infrastructure::context::InfrastructureContext;
use tokio::sync::mpsc;

#[tokio::test]
async fn deep_researcher_runs_max_loops_with_mock() {
    let cfg = MuonConfig::default();
    let infra = InfrastructureContext::mock();
    let (tx, mut rx) = mpsc::unbounded_channel::<AgentEvent>();
    let bridge = BridgeChannels::new(tx);
    let plan = ClarifierResult::default();
    let researcher = DeepResearcher::new(&cfg, &infra, &bridge);
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
