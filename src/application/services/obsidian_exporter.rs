use std::path::{Path, PathBuf};

use crate::domain::error::MuonError;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::session::Session;

pub struct ObsidianExporter;

impl ObsidianExporter {
    pub fn export(
        report: &ResearchReport,
        _session: &Session,
        vault_path: &Path,
    ) -> Result<PathBuf, MuonError> {
        let slug = slugify(&report.title);
        let dir = vault_path.join("Muon");
        std::fs::create_dir_all(&dir)?;

        let mut content = String::new();
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

        let file_path = dir.join(format!("{slug}.md"));
        std::fs::write(&file_path, content)?;

        let moc_path = dir.join("MOC.md");
        let moc_entry = format!("- [[{}]] {}\n", slug, report.title);
        if moc_path.exists() {
            let existing = std::fs::read_to_string(&moc_path)?;
            if !existing.contains(&format!("[[{slug}]]")) {
                let mut updated = existing;
                if !updated.ends_with('\n') {
                    updated.push('\n');
                }
                updated.push_str(&moc_entry);
                std::fs::write(&moc_path, updated)?;
            }
        } else {
            std::fs::write(&moc_path, moc_entry)?;
        }

        Ok(file_path)
    }
}

pub fn slugify(title: &str) -> String {
    let slug: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();

    let collapsed: String = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if collapsed.len() > 60 {
        collapsed[..60].to_string()
    } else {
        collapsed
    }
}
