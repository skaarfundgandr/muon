#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::config::ClarifierConfig;

#[test]
fn clarifier_default_plan_approval_is_true() {
    let c = ClarifierConfig::default();
    assert!(c.plan_approval, "default plan_approval must be true");
    assert_eq!(c.max_turns, 3);
    assert_eq!(c.max_iterations, 10);
}
