#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use uuid::Uuid;

use muon::application::services::PdfExporter;
use muon::domain::models::report::{ReportSection, ResearchReport};
use muon::domain::models::session::{ReportStats, Session, SessionStatus};
use muon::domain::models::pipeline::PipelineStage;

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
