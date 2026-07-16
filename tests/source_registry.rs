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

#[test]
fn record_source_upgrades_serp_teaser_from_enriched_sink() {
    // Mid-loop re-merge: local registry has SERP teaser; sink was enrich_page'd.
    let mut local = SourceRegistry::new();
    local.record_with_meta(
        "https://example.com",
        muon::domain::models::source::SourceType::Web,
        "SERP Title",
        "short teaser",
        0.5,
    );

    let mut sink = SourceRegistry::new();
    sink.record_with_meta(
        "https://example.com",
        muon::domain::models::source::SourceType::Web,
        "SERP Title",
        "short teaser",
        0.5,
    );
    sink.enrich_page(
        "https://example.com",
        "Fetched Title",
        "full page body text that is much longer than the teaser",
    );

    for src in sink.sources() {
        local.record_source(src);
    }

    let sources = local.into_sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(
        sources[0].snippet,
        "full page body text that is much longer than the teaser"
    );
    assert_eq!(sources[0].title, "Fetched Title");
}

#[test]
fn record_with_meta_does_not_shrink_snippet() {
    let mut reg = SourceRegistry::new();
    reg.record_with_meta(
        "https://example.com",
        muon::domain::models::source::SourceType::Web,
        "T",
        "longer body text here",
        0.0,
    );
    reg.record_with_meta(
        "https://example.com",
        muon::domain::models::source::SourceType::Web,
        "Other",
        "short",
        0.0,
    );
    let sources = reg.into_sources();
    assert_eq!(sources[0].snippet, "longer body text here");
    assert_eq!(sources[0].title, "T");
}
