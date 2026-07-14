use std::path::{Path, PathBuf};

use pdf_oxide::api::Pdf;

use crate::domain::error::MuonError;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::Session;

fn map_pdf_error(e: impl std::fmt::Display) -> MuonError {
    MuonError::Io(std::io::Error::other(format!("PDF export: {e}")))
}

fn build_markdown(report: &ResearchReport, session: &Session) -> String {
    let mut content = String::from("---\n");
    content.push_str(&format!("title: {}\n", report.title));
    content.push_str(&format!("query: {}\n", session.query));
    content.push_str(&format!(
        "created_at: {}\n",
        session.created_at.to_rfc3339()
    ));
    content.push_str(&format!("sources: {}\n", report.citations.len()));
    content.push_str("---\n\n");

    content.push_str(&report.summary);
    content.push_str("\n\n");

    for section in &report.sections {
        content.push_str(&format!("## {}\n\n", section.heading));
        content.push_str(&section.body_markdown);
        content.push_str("\n\n");
    }

    if !report.citations.is_empty() {
        content.push_str("## References\n\n");
        for citation in &report.citations {
            content.push_str(&format!(
                "{}. {} — {}\n",
                citation.reference_number, citation.title, citation.url
            ));
        }
    }

    content
}

pub struct PdfExporter;

impl PdfExporter {
    pub fn default_export_dir() -> Result<PathBuf, MuonError> {
        let base = dirs::data_dir().ok_or_else(|| {
            MuonError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "data directory not found",
            ))
        })?;
        Ok(base.join("muon").join("exports"))
    }

    pub fn export(report: &ResearchReport, session: &Session) -> Result<PathBuf, MuonError> {
        let dir = Self::default_export_dir()?;
        Self::export_to(report, session, &dir)
    }

    pub fn export_to(
        report: &ResearchReport,
        session: &Session,
        dir: &Path,
    ) -> Result<PathBuf, MuonError> {
        let path = dir.join(format!("{}.pdf", session.id));
        Self::export_to_path(report, session, &path)
    }

    pub fn export_to_path(
        report: &ResearchReport,
        session: &Session,
        path: &Path,
    ) -> Result<PathBuf, MuonError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let markdown = build_markdown(report, session);

        let mut pdf = Pdf::from_markdown(&markdown).map_err(map_pdf_error)?;
        pdf.save(path).map_err(map_pdf_error)?;

        Ok(path.to_path_buf())
    }
}
