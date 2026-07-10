#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::config::{ClarifierConfig, MuonConfig};

#[test]
fn clarifier_default_plan_approval_is_true() {
    let c = ClarifierConfig::default();
    assert!(c.plan_approval, "default plan_approval must be true");
    assert_eq!(c.max_turns, 3);
    assert_eq!(c.max_iterations, 10);
}

#[test]
fn migrated_config_copies_advanced_to_agents_when_agents_at_defaults() {
    let mut cfg = MuonConfig::default();
    cfg.advanced.max_clarifier_turns = 5;
    cfg.advanced.plan_approval = false;
    cfg.advanced.max_plan_iterations = 7;

    cfg.migrate_clarifier_config();

    assert_eq!(cfg.agents.clarifier.max_turns, 5);
    assert!(!cfg.agents.clarifier.plan_approval);
    assert_eq!(cfg.agents.clarifier.max_iterations, 7);
}

#[test]
fn migrated_config_does_not_overwrite_agents_when_agents_were_customized() {
    let mut cfg = MuonConfig::default();
    cfg.agents.clarifier.max_turns = 9;
    cfg.agents.clarifier.plan_approval = false;
    cfg.advanced.max_clarifier_turns = 5;
    cfg.advanced.plan_approval = true;

    cfg.migrate_clarifier_config();

    assert_eq!(cfg.agents.clarifier.max_turns, 9);
    assert!(!cfg.agents.clarifier.plan_approval);
}

#[test]
fn mirror_clarifier_to_advanced_writes_back_compat_fields() {
    let mut cfg = MuonConfig::default();
    cfg.agents.clarifier.max_turns = 4;
    cfg.agents.clarifier.plan_approval = true;
    cfg.agents.clarifier.max_iterations = 12;

    cfg.mirror_clarifier_to_advanced();

    assert_eq!(cfg.advanced.max_clarifier_turns, 4);
    assert!(cfg.advanced.plan_approval);
    assert_eq!(cfg.advanced.max_plan_iterations, 12);
}

#[test]
fn advanced_at_defaults_does_not_migrate() {
    let mut cfg = MuonConfig::default();
    cfg.migrate_clarifier_config();
    assert!(cfg.agents.clarifier.plan_approval);
    assert_eq!(cfg.agents.clarifier.max_turns, 3);
}
