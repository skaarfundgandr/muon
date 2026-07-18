#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::Utc;
use muon::application::pipeline::PipelineStage;
use muon::application::services::*;
use muon::domain::models::report::{Citation, ReportSection, ResearchReport, VerificationLevel};
use muon::domain::models::session::{ReportStats, Session, SessionStatus};
use uuid::Uuid;

fn make_session() -> Session {
    Session {
        id: Uuid::new_v4(),
        query: "test query".to_string(),
        status: SessionStatus::Complete,
        pipeline_stage: PipelineStage::Complete,
        intent: None,
        plan: None,
        clarifier_result: None,
        sources: Vec::new(),
        report: None,
        logs: Vec::new(),
        stats: ReportStats::default(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn make_report() -> ResearchReport {
    let mut report = ResearchReport::direct("Test summary content.");
    report.title = "Test Report".to_string();
    report
}

fn make_report_with_h1_summary(title: &str, h1: &str, body: &str) -> ResearchReport {
    ResearchReport {
        title: title.into(),
        summary: format!("# {}\n\n{}", h1, body),
        sections: vec![ReportSection {
            heading: "Section 1".into(),
            body_markdown: "Section body text.".into(),
        }],
        citations: vec![Citation {
            reference_number: 1,
            url: "https://example.com/article".into(),
            title: "Example Article".into(),
            context_snippet: String::new(),
            verification_level: VerificationLevel::Exact,
        }],
        stats: ReportStats::default(),
    }
}

#[test]
fn markdown_export_writes_file_with_frontmatter() {
    let tmp = tempfile::tempdir().unwrap();
    let report = make_report();
    let session = make_session();
    let path = MarkdownExporter::export_to(&report, &session, tmp.path()).unwrap();
    assert!(path.exists());
    assert!(path.starts_with(tmp.path()));
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("title: Test Report"));
    assert!(content.contains("query: test query"));
    assert!(content.contains("---"));
}

#[test]
fn obsidian_export_writes_to_vault_and_appends_moc() {
    let tmp = tempfile::tempdir().unwrap();
    let report = make_report();
    let session = make_session();
    let path = ObsidianExporter::export(&report, &session, tmp.path()).unwrap();
    assert!(path.exists());
    assert!(path.file_name().unwrap().to_string_lossy() == "test-report.md");

    let moc = tmp.path().join("Muon").join("MOC.md");
    assert!(moc.exists());
    let moc_content = std::fs::read_to_string(&moc).unwrap();
    assert!(moc_content.contains("[[test-report]]"));
    assert!(moc_content.contains("Test Report"));
}

#[test]
fn export_service_routes_format() {
    let tmp = tempfile::tempdir().unwrap();
    let report = make_report();
    let session = make_session();

    let req_md = ExportRequest {
        report: &report,
        session: &session,
        format: ExportFormat::Markdown,
        obsidian_vault: None,
        markdown_dir: Some(tmp.path()),
    };
    let path = ExportService::export(req_md).unwrap();
    assert!(path.exists());
    assert!(path.starts_with(tmp.path()));

    let vault = tempfile::tempdir().unwrap();
    let req_obs = ExportRequest {
        report: &report,
        session: &session,
        format: ExportFormat::Obsidian,
        obsidian_vault: Some(vault.path()),
        markdown_dir: None,
    };
    let path = ExportService::export(req_obs).unwrap();
    assert!(path.exists());

    let req_no_vault = ExportRequest {
        report: &report,
        session: &session,
        format: ExportFormat::Obsidian,
        obsidian_vault: None,
        markdown_dir: None,
    };
    assert!(ExportService::export(req_no_vault).is_err());
}

#[test]
fn export_format_from_str() {
    assert_eq!(
        "markdown".parse::<ExportFormat>().unwrap(),
        ExportFormat::Markdown
    );
    assert_eq!(
        "Obsidian".parse::<ExportFormat>().unwrap(),
        ExportFormat::Obsidian
    );
    assert!("unknown".parse::<ExportFormat>().is_err());
}

#[test]
fn export_format_display() {
    assert_eq!(ExportFormat::Markdown.to_string(), "markdown");
    assert_eq!(ExportFormat::Obsidian.to_string(), "obsidian");
}

// ---- Phase 3 additions ----

#[test]
fn markdown_export_preserves_llm_h1_in_body() {
    let tmp = tempfile::tempdir().unwrap();
    let report = make_report_with_h1_summary("Different Plan Title", "Other Title", "Real content.");
    let session = make_session();
    let path = MarkdownExporter::export_to(&report, &session, tmp.path()).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains("title: Different Plan Title"), "YAML must contain the frontmatter title");

    // The body (after the closing `---\n`) keeps the LLM-drafted H1 line
    // rather than stripping it: title SSOT lives in YAML metadata, and the
    // body mirrors the LLM response verbatim.
    let body = content.split("---\n").nth(2).unwrap_or("");
    assert!(
        body.trim_start().starts_with("# Other Title"),
        "body after YAML must preserve the LLM-drafted H1, got: {body:?}"
    );
    assert!(body.contains("Real content."), "body must contain the real summary content");
    assert!(body.contains("Section 1"), "body must contain section headings");
}

#[test]
fn markdown_export_keeps_sections_and_references() {
    let tmp = tempfile::tempdir().unwrap();
    let report = make_report_with_h1_summary("Sections", "Title", "Body content here.");
    let session = make_session();
    let path = MarkdownExporter::export_to(&report, &session, tmp.path()).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    assert!(content.contains("## Section 1"), "body must contain section heading");
    assert!(content.contains("1. Example Article — https://example.com/article"), "body must contain citation with ref number, title, and URL");
}

#[test]
fn markdown_export_yaml_keys_present() {
    let tmp = tempfile::tempdir().unwrap();
    let report = make_report_with_h1_summary("Test Title", "Some H1", "Body.");
    let session = make_session();
    let path = MarkdownExporter::export_to(&report, &session, tmp.path()).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    // Everything before the second `---` is frontmatter
    let fm = content.split("---\n").nth(1).unwrap_or("");
    assert!(fm.contains("title:"), "frontmatter must contain title:");
    assert!(fm.contains("query:"), "frontmatter must contain query:");
    assert!(fm.contains("created_at:"), "frontmatter must contain created_at:");
    assert!(fm.contains("sources:"), "frontmatter must contain sources:");
}

#[test]
fn markdown_export_closing_yaml_fence_on_own_line() {
    let tmp = tempfile::tempdir().unwrap();
    let report = make_report_with_h1_summary("Title", "H1", "Body.");
    let session = make_session();
    let path = MarkdownExporter::export_to(&report, &session, tmp.path()).unwrap();
    let content = std::fs::read_to_string(&path).unwrap();

    let expected_sources_line = format!("sources: {}\n---\n", report.citations.len());
    assert!(
        content.contains(&expected_sources_line),
        "expected YAML to contain a standalone closing fence after `sources: {}`; got:\n{}",
        report.citations.len(),
        content
    );
}
