use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::domain::error::MuonError;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::Session;

use super::markdown_exporter::MarkdownExporter;
use super::obsidian_exporter::ObsidianExporter;
use super::pdf_exporter::PdfExporter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Obsidian,
    Pdf,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Markdown => write!(f, "markdown"),
            ExportFormat::Obsidian => write!(f, "obsidian"),
            ExportFormat::Pdf => write!(f, "pdf"),
        }
    }
}

impl FromStr for ExportFormat {
    type Err = MuonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" => Ok(ExportFormat::Markdown),
            "obsidian" => Ok(ExportFormat::Obsidian),
            "pdf" => Ok(ExportFormat::Pdf),
            other => Err(MuonError::Config(format!("unknown export format: {other}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportRequest<'a> {
    pub report: &'a ResearchReport,
    pub session: &'a Session,
    pub format: ExportFormat,
    pub obsidian_vault: Option<&'a Path>,
    pub markdown_dir: Option<&'a Path>,
}

pub struct ExportService;

impl ExportService {
    pub fn export(req: ExportRequest<'_>) -> Result<PathBuf, MuonError> {
        match req.format {
            ExportFormat::Markdown => match req.markdown_dir {
                Some(dir) => MarkdownExporter::export_to(req.report, req.session, dir),
                None => MarkdownExporter::export(req.report, req.session),
            },
            ExportFormat::Obsidian => {
                let vault = req
                    .obsidian_vault
                    .ok_or_else(|| MuonError::Config("obsidian_vault missing".into()))?;
                ObsidianExporter::export(req.report, req.session, vault)
            }
            ExportFormat::Pdf => match req.markdown_dir {
                Some(dir) => PdfExporter::export_to(req.report, req.session, dir),
                None => PdfExporter::export(req.report, req.session),
            },
        }
    }
}
