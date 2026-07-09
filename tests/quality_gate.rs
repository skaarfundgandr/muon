#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::application::deep_researcher::is_report_complete;

fn url(s: &str) -> Vec<String> {
    vec![s.to_string()]
}

#[test]
fn passes_when_all_checks_met() {
    let draft = "# Title\n\n## Summary\nLong enough body text here. Yes indeed. \
                 Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
                 eiusmod tempor incididunt ut labore et dolore magna aliqua.\n\n\
                 ## Sources\n- https://example.com/page\n";
    let (complete, reason) = is_report_complete(draft, &url("https://example.com/page"), 100, 2);
    assert!(complete, "expected complete, reason: {reason}");
    assert!(reason.is_empty());
}

#[test]
fn fails_when_too_short() {
    let draft = "short";
    let (complete, reason) = is_report_complete(draft, &[], 1000, 0);
    assert!(!complete);
    assert!(reason.contains("too short"), "reason: {reason}");
    assert!(reason.contains("5 chars"), "reason: {reason}");
}

#[test]
fn fails_when_missing_section_headers() {
    let mut draft = String::new();
    for _ in 0..200 {
        draft.push('x');
    }
    let (complete, reason) = is_report_complete(&draft, &[], 100, 2);
    assert!(!complete);
    assert!(reason.contains("section"), "reason: {reason}");
}

#[test]
fn fails_when_no_sources_section_and_no_known_url() {
    let draft = "# Title\n\n## Body\nVery long body content here that easily exceeds \
                 the length threshold but has no sources heading or known url. \
                 Additional padding to ensure length compliance with the gate. \
                 Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
                 eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim \
                 ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut \
                 aliquip ex ea commodo consequat.\n";
    let (complete, reason) =
        is_report_complete(draft, &url("https://example.com/missing"), 100, 1);
    assert!(!complete);
    assert!(reason.contains("citation"), "reason: {reason}");
}

#[test]
fn passes_with_references_heading() {
    let draft = "# Title\n\n## References\n\
                 Long enough body text. Lorem ipsum dolor sit amet, consectetur \
                 adipiscing elit, sed do eiusmod tempor incididunt ut labore et \
                 dolore magna aliqua. Ut enim ad minim veniam, quis nostrud.\n";
    let (complete, _reason) = is_report_complete(draft, &url("https://example.com/x"), 100, 1);
    assert!(complete);
}

#[test]
fn passes_when_registry_empty_even_without_sources_heading() {
    let draft = "# Title\n\n## Body\nLong enough body text. Lorem ipsum dolor sit \
                 amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt \
                 ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis \
                 nostrud exercitation ullamco laboris nisi ut aliquip.\n";
    let (complete, _reason) = is_report_complete(draft, &[], 100, 1);
    assert!(complete);
}

#[test]
fn fails_on_gave_up_phrase() {
    let draft = "# Title\n\n## Body\nLong enough body. Lorem ipsum dolor sit amet, \
                 consectetur adipiscing elit, sed do eiusmod tempor incididunt ut \
                 labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud.\
                 \nPlease confirm if you want me to continue.\n";
    let (complete, reason) = is_report_complete(draft, &[], 100, 1);
    assert!(!complete);
    assert!(reason.contains("gave up"), "reason: {reason}");
}

#[test]
fn gave_up_phrase_case_insensitive() {
    let draft = "# Title\n\n## Body\nLong enough body. Lorem ipsum dolor sit amet, \
                 consectetur adipiscing elit, sed do eiusmod tempor incididunt ut \
                 labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud.\
                 \nShould I proceed?\n";
    let (complete, reason) = is_report_complete(draft, &[], 100, 1);
    assert!(!complete);
    assert!(reason.contains("gave up"), "reason: {reason}");
}

#[test]
fn cites_known_url_passes_sources_check() {
    let draft = "# Title\n\n## Body\nThis is long enough body text. Lorem ipsum dolor \
                 sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
                 incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam.\
                 See https://example.com/known for details.\n";
    let (complete, _reason) =
        is_report_complete(draft, &url("https://example.com/known"), 100, 1);
    assert!(complete);
}

#[test]
fn zero_section_threshold_passes_with_zero_headers() {
    let draft = "# Only a top-level title\nThis body is long enough to pass the \
                 length check. Lorem ipsum dolor sit amet, consectetur adipiscing \
                 elit, sed do eiusmod tempor incididunt ut labore et dolore magna \
                 aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco.\n";
    let (complete, _reason) = is_report_complete(draft, &[], 100, 0);
    assert!(complete);
}
