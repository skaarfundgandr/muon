use std::path::{Path, PathBuf};

use pdf_oxide::api::PdfBuilder;

use crate::application::services::{soft_wrap_markdown_for_pdf, strip_leading_h1};
use crate::domain::error::MuonError;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::Session;

fn build_pdf_markdown(report: &ResearchReport, _session: &Session) -> String {
    let mut content = String::new();
    content.push_str(&strip_leading_h1(&report.summary));
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

fn map_pdf_error(e: impl std::fmt::Display) -> MuonError {
    MuonError::Io(std::io::Error::other(format!("PDF export: {e}")))
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

        let body = soft_wrap_markdown_for_pdf(&build_pdf_markdown(report, session), 96);
        let mut pdf = PdfBuilder::new()
            .title(report.title.as_str())
            .subject(session.query.as_str())
            .from_markdown(&body)
            .map_err(map_pdf_error)?;
        pdf.save(path).map_err(map_pdf_error)?;

        Ok(path.to_path_buf())
    }
}
