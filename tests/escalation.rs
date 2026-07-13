#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::pipeline_runner::escalation::should_escalate;
use muon::domain::models::report::{ReportSection, ResearchReport};

fn report_with_summary(s: &str) -> ResearchReport {
    ResearchReport {
        title: "t".into(),
        summary: s.to_string(),
        sections: vec![],
        citations: vec![],
        stats: Default::default(),
    }
}

fn report_with_sections(summary: &str, bodies: &[&str]) -> ResearchReport {
    ResearchReport {
        title: "t".into(),
        summary: summary.to_string(),
        sections: bodies
            .iter()
            .map(|b| ReportSection {
                heading: "H".into(),
                body_markdown: b.to_string(),
            })
            .collect(),
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

#[test]
fn keyword_in_section_body_escalates() {
    let report = report_with_sections(
        "Summary looks fine.",
        &["Some body text that says I am unable to find the answer here."],
    );
    assert!(should_escalate(&report));
}

#[test]
fn empty_summary_with_clean_body_does_not_escalate() {
    let report = report_with_sections(
        "",
        &["Rust is a systems programming language focused on safety and concurrency."],
    );
    assert!(!should_escalate(&report));
}

#[test]
fn keyword_outside_last_800_chars_does_not_escalate() {
    let padding = "x".repeat(900);
    let report = report_with_summary(&format!("unable to find something early on {padding}"));
    assert!(!should_escalate(&report));
}

#[test]
fn keyword_within_last_800_chars_escalates() {
    let padding = "x".repeat(700);
    let report = report_with_summary(&format!("{padding} unable to find something near the end"));
    assert!(should_escalate(&report));
}

#[test]
fn empty_full_text_escalates() {
    let report = report_with_sections("", &["   "]);
    assert!(should_escalate(&report));
}
