#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use muon::domain::agents::clarifier::ClarifierResult;
use muon::domain::models::research_plan::ResearchPlan;
use muon::domain::traits::session_store::SessionStore;

#[tokio::test]
async fn clarifier_result_to_plan_returns_plan_when_title_present() {
    let cr = ClarifierResult {
        clarifier_log: String::new(),
        plan_title: Some("My Plan".into()),
        plan_sections: vec!["Section 1".into(), "Section 2".into()],
        plan_approved: true,
    };
    let plan = cr.to_plan();
    assert!(plan.is_some());
    let plan = plan.unwrap();
    assert_eq!(plan.title, "My Plan");
    assert_eq!(plan.sections.len(), 2);
    assert!(plan.approved);
}

#[tokio::test]
async fn clarifier_result_to_plan_returns_none_when_no_title() {
    let cr = ClarifierResult {
        clarifier_log: String::new(),
        plan_title: None,
        plan_sections: vec![],
        plan_approved: false,
    };
    assert!(cr.to_plan().is_none());
}

#[tokio::test]
async fn save_clarifier_outcome_round_trips_through_diesel_store() {
    let (_dir, store) = common::diesel_store().await;
    let id = store.create("test query").await.unwrap();

    let plan = ResearchPlan {
        title: "Test Plan".into(),
        sections: vec!["A".into(), "B".into()],
        approved: true,
    };
    let plan_json = serde_json::to_string(&plan).unwrap();
    let cr = ClarifierResult {
        clarifier_log: "log".into(),
        plan_title: Some("Test Plan".into()),
        plan_sections: vec!["A".into(), "B".into()],
        plan_approved: true,
    };
    let cr_json = serde_json::to_string(&cr).unwrap();

    store
        .save_clarifier_outcome(id, Some(&plan_json), Some(&cr_json))
        .await
        .unwrap();

    let (stored_plan, stored_cr) = store
        .get_clarifier_outcome(id)
        .await
        .unwrap()
        .expect("outcome missing");
    assert_eq!(stored_plan.as_deref(), Some(plan_json.as_str()));
    assert_eq!(stored_cr.as_deref(), Some(cr_json.as_str()));
}

#[tokio::test]
async fn save_clarifier_outcome_with_plan_none_stores_clarifier_result() {
    let (_dir, store) = common::diesel_store().await;
    let id = store.create("qa-only").await.unwrap();

    let cr = ClarifierResult {
        clarifier_log: "qa log".into(),
        plan_title: None,
        plan_sections: vec![],
        plan_approved: false,
    };
    let cr_json = serde_json::to_string(&cr).unwrap();

    store
        .save_clarifier_outcome(id, None, Some(&cr_json))
        .await
        .unwrap();

    let (stored_plan, stored_cr) = store
        .get_clarifier_outcome(id)
        .await
        .unwrap()
        .expect("outcome missing");
    assert!(stored_plan.is_none());
    assert_eq!(stored_cr.as_deref(), Some(cr_json.as_str()));
}
