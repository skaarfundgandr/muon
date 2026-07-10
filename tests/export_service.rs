#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use chrono::Utc;
use muon::application::pipeline::PipelineStage;
use muon::application::services::*;
use muon::domain::models::report::ResearchReport;
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
