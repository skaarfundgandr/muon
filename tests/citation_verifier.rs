#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use muon::application::pipeline_runner::citation_verifier::{
    extract_urls, match_url, normalize, normalize_url, report_reflow, sanitize, verify,
    RemovalReason, ValidCitation, VerificationLevel,
};
use muon::domain::models::report::{Citation, ReportSection, ResearchReport};
use muon::domain::models::session::ReportStats;
use muon::error::MuonError;

fn make_report(citations: Vec<Citation>, body: &str) -> ResearchReport {
    ResearchReport {
        title: "Test".to_string(),
        summary: body.to_string(),
        sections: vec![ReportSection {
            heading: "Section".to_string(),
            body_markdown: body.to_string(),
        }],
        citations,
        stats: ReportStats::default(),
    }
}

#[test]
fn exact_match() {
    let registry = vec!["https://example.com/page".to_string()];
    assert_eq!(
        match_url("https://example.com/page", &registry),
        Some(VerificationLevel::Exact)
    );
}

#[test]
fn prefix_match() {
    let registry = vec!["https://example.com/page/sub".to_string()];
    assert_eq!(
        match_url("https://example.com/page", &registry),
        Some(VerificationLevel::Prefix)
    );
}

#[test]
fn no_match() {
    let registry = vec!["https://other.com".to_string()];
    assert_eq!(match_url("https://example.com", &registry), None);
}

#[test]
fn normalize_strips_trailing_slash() {
    assert_eq!(normalize("https://Example.com/"), "https://example.com");
}

#[test]
fn empty_url_no_match() {
    let registry = vec!["https://example.com".to_string()];
    assert_eq!(match_url("", &registry), None);
}

#[test]
fn test_truncation_match() {
    let urls = vec!["https://example.com/blog/2024-01-post-title".to_string()];
    let result = match_url("https://example.com/blog/", &urls);
    assert_eq!(result, Some(VerificationLevel::Prefix));
}

#[test]
fn test_child_path_match() {
    let urls = vec!["https://example.com/archive/other-doc".to_string()];
    let result = match_url("https://example.com/some/path/doc", &urls);
    assert_eq!(result, Some(VerificationLevel::ChildPath));
}

#[test]
fn test_query_subset_match() {
    let urls = vec!["https://example.com/search?q=rust&page=1&lang=en".to_string()];
    let result = match_url("https://example.com/search?q=rust&page=1", &urls);
    assert_eq!(result, Some(VerificationLevel::QuerySubset));
}

#[test]
fn test_sanitization_bit_ly() -> Result<(), MuonError> {
    assert!(!sanitize("https://bit.ly/abc123")?);
    Ok(())
}

#[test]
fn test_sanitization_t_co() -> Result<(), MuonError> {
    assert!(!sanitize("https://t.co/abc123")?);
    Ok(())
}

#[test]
fn test_sanitization_dots() -> Result<(), MuonError> {
    assert!(!sanitize("https://example.com/.../path")?);
    Ok(())
}

#[test]
fn test_sanitization_ip_literal() -> Result<(), MuonError> {
    assert!(!sanitize("https://192.168.1.1/page")?);
    Ok(())
}

#[test]
fn test_sanitization_ipv6() -> Result<(), MuonError> {
    assert!(!sanitize("https://[::1]/page")?);
    Ok(())
}

#[test]
fn test_sanitization_data_uri() -> Result<(), MuonError> {
    assert!(!sanitize("data:text/html,<h1>hi</h1>")?);
    Ok(())
}

#[test]
fn test_sanitization_javascript() -> Result<(), MuonError> {
    assert!(!sanitize("javascript:alert(1)")?);
    Ok(())
}

#[test]
fn test_report_reflow_numbered() -> Result<(), MuonError> {
    let valid = vec![
        ValidCitation {
            url: "https://a.com".to_string(),
            level: VerificationLevel::Exact,
        },
        ValidCitation {
            url: "https://c.com".to_string(),
            level: VerificationLevel::Prefix,
        },
    ];
    let md = "See [1] and [2] and [3]. Also [url1] and [url2].";
    let result = report_reflow(md, &valid)?;
    assert!(result.contains("[1]"));
    assert!(result.contains("[2]"));
    assert!(!result.contains("[3]"));
    Ok(())
}

#[test]
fn test_report_reflow_url_style() -> Result<(), MuonError> {
    let valid = vec![
        ValidCitation {
            url: "https://b.com".to_string(),
            level: VerificationLevel::Exact,
        },
        ValidCitation {
            url: "https://a.com".to_string(),
            level: VerificationLevel::Prefix,
        },
    ];
    let md = "Check [https://a.com] and [https://b.com].";
    let result = report_reflow(md, &valid)?;
    assert!(result.contains("[1]"));
    assert!(result.contains("[2]"));
    assert!(!result.contains("https://"));
    Ok(())
}

#[test]
fn test_knowledge_path_match() -> Result<(), MuonError> {
    let kp = vec!["my-report.pdf".to_string()];
    let report = make_report(
        vec![Citation {
            reference_number: 1,
            url: String::new(),
            title: "Report".to_string(),
            context_snippet: "my-report.pdf, p.15".to_string(),
            verification_level: VerificationLevel::Exact,
        }],
        "Body text",
    );
    let result = verify(&report, &[], &kp)?;
    assert_eq!(result.valid_citations.len(), 1);
    assert!(result.removed_citations.is_empty());
    Ok(())
}

#[test]
fn test_empty_registry_all_removed() -> Result<(), MuonError> {
    let report = make_report(
        vec![Citation {
            reference_number: 1,
            url: "https://example.com".to_string(),
            title: "Ex".to_string(),
            context_snippet: "ctx".to_string(),
            verification_level: VerificationLevel::Exact,
        }],
        "Body",
    );
    let result = verify(&report, &[], &[])?;
    assert!(result.valid_citations.is_empty());
    assert_eq!(result.removed_citations.len(), 1);
    assert_eq!(
        result.removed_citations[0].reason,
        RemovalReason::UrlNotInRegistry
    );
    Ok(())
}

#[test]
fn test_empty_report_no_citations() -> Result<(), MuonError> {
    let report = make_report(vec![], "Body");
    let result = verify(&report, &[], &[])?;
    assert!(result.valid_citations.is_empty());
    assert!(result.removed_citations.is_empty());
    Ok(())
}

#[test]
fn test_normalize_url() {
    assert_eq!(
        normalize_url("HTTPS://Example.COM/Path/"),
        "https://example.com/path"
    );
    assert_eq!(
        normalize_url("https://www.example.com/"),
        "https://example.com"
    );
    assert_eq!(
        normalize_url("https://example.com/page#section"),
        "https://example.com/page"
    );
}

#[test]
fn test_extract_urls() -> Result<(), MuonError> {
    let md = "Visit [1](https://a.com) and https://b.com/path?q=1.";
    let urls = extract_urls(md)?;
    assert_eq!(urls.len(), 2);
    assert_eq!(urls[0], "https://a.com");
    assert_eq!(urls[1], "https://b.com/path?q=1");
    Ok(())
}

#[test]
fn test_extract_urls_json_garbage() -> Result<(), MuonError> {
    let junk = r#"{"url":"https://schema.org","@graph":[{"@type":"WebPage"}]}"#;
    let urls = extract_urls(junk)?;
    assert_eq!(urls, vec!["https://schema.org".to_string()]);
    Ok(())
}

#[test]
fn test_verify_full_pipeline() -> Result<(), MuonError> {
    let report = make_report(
        vec![
            Citation {
                reference_number: 1,
                url: "https://example.com/doc".to_string(),
                title: "Doc".to_string(),
                context_snippet: "ctx".to_string(),
                verification_level: VerificationLevel::Exact,
            },
            Citation {
                reference_number: 2,
                url: "https://unknown.com/page".to_string(),
                title: "Unknown".to_string(),
                context_snippet: "ctx".to_string(),
                verification_level: VerificationLevel::Exact,
            },
        ],
        "See [1] and [2].",
    );
    let registry = vec!["https://example.com/doc".to_string()];
    let result = verify(&report, &registry, &[])?;
    assert_eq!(result.valid_citations.len(), 1);
    assert_eq!(result.removed_citations.len(), 1);
    assert_eq!(result.valid_citations[0].level, VerificationLevel::Exact);
    assert_eq!(
        result.removed_citations[0].reason,
        RemovalReason::UrlNotInRegistry
    );
    Ok(())
}

#[test]
fn test_unverifiable_citation_no_url() -> Result<(), MuonError> {
    let report = make_report(
        vec![Citation {
            reference_number: 1,
            url: String::new(),
            title: "No URL".to_string(),
            context_snippet: "random text".to_string(),
            verification_level: VerificationLevel::Exact,
        }],
        "Body",
    );
    let result = verify(&report, &[], &[])?;
    assert_eq!(result.removed_citations.len(), 1);
    assert_eq!(
        result.removed_citations[0].reason,
        RemovalReason::CitationKeyNotInRegistry
    );
    Ok(())
}
