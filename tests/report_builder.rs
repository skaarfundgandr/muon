#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use muon::application::services::{derive_title, split_sections};

#[test]
fn test_split_sections_no_headings() {
    let md = "This is a summary.\nMore summary.";
    let (summary, sections) = split_sections(md);
    assert_eq!(summary, "This is a summary.\nMore summary.");
    assert!(sections.is_empty());
}

#[test]
fn test_split_sections_with_headings() {
    let md = "Summary text.\n\n## First Section\n\nBody one.\n\n## Second Section\n\nBody two.";
    let (summary, sections) = split_sections(md);
    assert_eq!(summary, "Summary text.");
    assert_eq!(sections.len(), 2);
    assert_eq!(sections[0].heading, "First Section");
    assert_eq!(sections[0].body_markdown, "Body one.");
    assert_eq!(sections[1].heading, "Second Section");
    assert_eq!(sections[1].body_markdown, "Body two.");
}

#[test]
fn test_derive_title_from_url() {
    assert_eq!(derive_title("https://example.com/docs/my-page"), "my page");
}

#[test]
fn test_derive_title_empty() {
    assert_eq!(derive_title(""), "");
}
