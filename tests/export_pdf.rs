#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use uuid::Uuid;

use muon::application::services::PdfExporter;
use muon::domain::models::pipeline::PipelineStage;
use muon::domain::models::report::{Citation, ReportSection, ResearchReport, VerificationLevel};
use muon::domain::models::session::{ReportStats, Session, SessionStatus};

fn test_report() -> ResearchReport {
    ResearchReport {
        title: "Test Report".into(),
        summary: "This is a test report summary.".into(),
        sections: vec![ReportSection {
            heading: "Introduction".into(),
            body_markdown: "This is the introduction section.".into(),
        }],
        citations: vec![],
        stats: ReportStats::default(),
    }
}

fn test_session() -> Session {
    Session {
        id: Uuid::new_v4(),
        query: "test query".into(),
        status: SessionStatus::Complete,
        pipeline_stage: PipelineStage::Complete,
        ..Session::default()
    }
}

#[test]
fn test_export_pdf_to_temp_dir() {
    let dir = tempfile::tempdir().unwrap();
    let report = test_report();
    let session = test_session();

    let path = PdfExporter::export_to(&report, &session, dir.path()).unwrap();

    assert!(
        path.starts_with(dir.path()),
        "path {:?} should be under {:?}",
        path,
        dir.path()
    );
    assert!(path.exists(), "PDF file should exist at {:?}", path);
    assert!(
        path.metadata().unwrap().len() > 0,
        "PDF file should be non-empty"
    );

    // Check PDF magic bytes
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..4], b"%PDF", "file should start with PDF magic bytes");
}

#[test]
fn test_export_pdf_to_specific_path() {
    let dir = tempfile::tempdir().unwrap();
    let pdf_path = dir.path().join("output.pdf");
    let report = test_report();
    let session = test_session();

    let path =
        PdfExporter::export_to_path(&report, &session, &pdf_path).unwrap();

    assert_eq!(path, pdf_path);
    assert!(path.exists());
    assert!(path.metadata().unwrap().len() > 0);

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..4], b"%PDF");
}

#[test]
fn test_export_pdf_default_dir() {
    let report = test_report();
    let session = test_session();

    let path = PdfExporter::export(&report, &session).unwrap();

    assert!(path.exists(), "PDF file should exist at {:?}", path);
    assert!(
        path.metadata().unwrap().len() > 0,
        "PDF file should be non-empty"
    );

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..4], b"%PDF");

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_export_pdf_creates_parent_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested").join("subdir");
    let report = test_report();
    let session = test_session();

    let path = PdfExporter::export_to(&report, &session, &nested).unwrap();

    assert!(path.exists());
    assert!(path.metadata().unwrap().len() > 0);
}

fn extract_pdf_text(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    muon::infrastructure::agent_rs::pdf_bytes_to_text(&bytes, 1_000_000).unwrap()
}

// ---- Phase 3 additions ----

#[test]
fn test_export_pdf_has_no_yaml_as_body() {
    let report = ResearchReport {
        title: "YAML Test".into(),
        summary: "Real content here.\n\nMore details.".into(),
        sections: vec![ReportSection {
            heading: "Details".into(),
            body_markdown: "More text.".into(),
        }],
        citations: vec![],
        stats: ReportStats::default(),
    };
    let session = test_session();
    let dir = tempfile::tempdir().unwrap();
    let path = PdfExporter::export_to(&report, &session, dir.path()).unwrap();
    let text = extract_pdf_text(&path);

    // PDF body should NOT start with YAML frontmatter markers or keys
    assert!(!text.starts_with("---"), "PDF body must not begin with YAML frontmatter marker");
    assert!(!text.contains("created_at:"), "PDF body must not contain YAML frontmatter 'created_at:'");
    assert!(!text.contains("sources:"), "PDF body must not contain YAML frontmatter 'sources:'");
    assert!(text.contains("Real content here."), "actual summary text must be present");
}

#[test]
fn test_export_pdf_long_body_long_url_preserved() {
    let long_url = format!(
        "https://example.com/very/long/path/{}/more/stuff/at/the/end",
        "a".repeat(150)
    );
    let report = ResearchReport {
        title: "URL Test".into(),
        summary: "A paragraph with enough text to fill more than one line in the PDF rendering at 96-character wrap width. This is not a URL. "
            .repeat(10)
            .into(),
        sections: vec![],
        citations: vec![Citation {
            reference_number: 1,
            url: long_url.clone(),
            title: "Long URL Article".into(),
            context_snippet: String::new(),
            verification_level: VerificationLevel::Exact,
        }],
        stats: ReportStats::default(),
    };
    let session = test_session();
    let dir = tempfile::tempdir().unwrap();
    let path = PdfExporter::export_to(&report, &session, dir.path()).unwrap();
    let text = extract_pdf_text(&path);

    assert!(text.contains("https://"), "extracted text should contain URL http part");
    assert!(text.contains(&long_url[..25]), "extracted text should contain URL prefix");
    assert!(text.contains(&long_url[long_url.len() - 20..]), "extracted text should contain URL suffix");
    assert!(!text.contains("created_at:"), "PDF body must not contain YAML keys");
}

fn make_report_with_title_and_h1_summary(title: &str, h1: &str, body: &str) -> ResearchReport {
    ResearchReport {
        title: title.into(),
        summary: format!("# {}\n\n{}", h1, body),
        sections: vec![],
        citations: vec![],
        stats: ReportStats::default(),
    }
}

#[test]
fn test_export_pdf_strips_body_h1_when_differs_from_title() {
    let report = make_report_with_title_and_h1_summary(
        "Plan Title",
        "Totally Different H1",
        "Real summary text.",
    );
    let session = test_session();
    let dir = tempfile::tempdir().unwrap();
    let path = PdfExporter::export_to(&report, &session, dir.path()).unwrap();
    let text = extract_pdf_text(&path);

    assert!(
        !text.contains("Totally Different H1"),
        "extracted PDF text must not contain the H1 header line from summary"
    );
    assert!(
        text.contains("Real summary text."),
        "extracted PDF text must contain the actual summary text after the H1"
    );
}

#[test]
fn test_export_pdf_info_metadata_set() {
    let report = make_report_with_title_and_h1_summary(
        "Plan Title",
        "Summary H1",
        "Real summary text.",
    );
    let session = test_session();
    let dir = tempfile::tempdir().unwrap();
    let path = PdfExporter::export_to(&report, &session, dir.path()).unwrap();

    assert!(path.exists(), "PDF file must exist");
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..4], b"%PDF", "file must start with PDF magic bytes");

    // Verify it's a valid parseable PDF via Pdf::from_bytes
    let _pdf = pdf_oxide::api::Pdf::from_bytes(bytes).unwrap();
}
