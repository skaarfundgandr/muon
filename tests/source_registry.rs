#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use muon::infrastructure::source_registry::SourceRegistry;

#[test]
fn enrich_page_new_entry() {
    let mut reg = SourceRegistry::new();
    reg.enrich_page("https://example.com", "Example", "some body text");
    let sources = reg.into_sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].url, "https://example.com");
    assert_eq!(sources[0].title, "Example");
    assert_eq!(sources[0].snippet, "some body text");
}

#[test]
fn enrich_page_updates_existing() {
    let mut reg = SourceRegistry::new();
    reg.record_with_meta(
        "https://example.com",
        muon::domain::models::source::SourceType::Web,
        "",
        "",
        0.0,
    );
    reg.enrich_page("https://example.com", "New Title", "longer body text");
    let sources = reg.into_sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].title, "New Title");
    assert_eq!(sources[0].snippet, "longer body text");
}

#[test]
fn enrich_page_keeps_longer_body() {
    let mut reg = SourceRegistry::new();
    reg.enrich_page("https://example.com", "", "shorter");
    reg.enrich_page("https://example.com", "", "longer body text here");
    let sources = reg.into_sources();
    assert_eq!(sources[0].snippet, "longer body text here");
}

#[test]
fn enrich_page_does_not_shrink_body() {
    let mut reg = SourceRegistry::new();
    reg.enrich_page("https://example.com", "", "longer body text here");
    reg.enrich_page("https://example.com", "", "short");
    let sources = reg.into_sources();
    assert_eq!(sources[0].snippet, "longer body text here");
}
