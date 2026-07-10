use std::path::{Path, PathBuf};

use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::Session;
use crate::domain::error::MuonError;

pub struct MarkdownExporter;

impl MarkdownExporter {
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
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.md", session.id));

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

        std::fs::write(&path, content)?;
        Ok(path)
    }
}
