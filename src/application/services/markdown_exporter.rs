use std::path::{Path, PathBuf};

use serde::Serialize;

use noyalib::compat::serde_yaml;

use crate::application::services::strip_leading_h1;
use crate::domain::error::MuonError;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::Session;

#[derive(Serialize)]
struct FrontMatter<'a> {
    title: &'a str,
    query: &'a str,
    created_at: String,
    sources: usize,
}

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

        let fm = FrontMatter {
            title: &report.title,
            query: &session.query,
            created_at: session.created_at.to_rfc3339(),
            sources: report.citations.len(),
        };
        let fm_yaml = serde_yaml::to_string(&fm)
            .map_err(|e| MuonError::Io(std::io::Error::other(format!("frontmatter yaml: {e}"))))?;
        let mut content = format!("---\n{fm_yaml}---\n\n");

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

        std::fs::write(&path, content)?;
        Ok(path)
    }
}
