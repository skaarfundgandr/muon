use muon::application::pipeline_runner::escalation::should_escalate;
use muon::domain::models::report::ResearchReport;

fn report_with_summary(s: &str) -> ResearchReport {
    ResearchReport {
        title: "t".into(),
        summary: s.to_string(),
        sections: vec![],
        citations: vec![],
        stats: Default::default(),
    }
}

#[test]
fn empty_summary_escalates() {
    assert!(should_escalate(&report_with_summary("")));
    assert!(should_escalate(&report_with_summary("   ")));
}

#[test]
fn clean_summary_does_not_escalate() {
    assert!(!should_escalate(&report_with_summary(
        "Rust is a systems programming language focused on safety and concurrency."
    )));
}

#[test]
fn unable_to_find_escalates() {
    assert!(should_escalate(&report_with_summary(
        "I am unable to find a definitive source on this exact topic."
    )));
}

#[test]
fn need_more_research_escalates() {
    assert!(should_escalate(&report_with_summary(
        "We will need more research before drawing conclusions."
    )));
}

#[test]
fn i_dont_have_enough_escalates() {
    assert!(should_escalate(&report_with_summary(
        "I don't have enough information to answer this with confidence."
    )));
}
