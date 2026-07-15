use std::path::Path;

use tokio::sync::mpsc;

use crate::application::bridge::BridgeChannels;
use crate::application::pipeline::PipelineState;
use crate::application::pipeline_runner::run_pipeline;
use crate::application::services::{ExportFormat, ObsidianExporter, PdfExporter};
use crate::domain::error::{MuonError, Result};
use crate::domain::models::report::ResearchReport;
use crate::domain::models::{Session, SessionId, SessionStatus};
use crate::domain::traits::session_store::{SessionStore, SessionSummary};
use crate::infrastructure::context::InfrastructureContext;
use crate::infrastructure::storage::{DieselSessionStore, init_pool};

fn render_markdown(report: &ResearchReport, query: &str) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("title: {}\n", report.title));
    out.push_str(&format!("query: {query}\n"));
    out.push_str(&format!("sources: {}\n", report.citations.len()));
    out.push_str("---\n\n");
    out.push_str(&report.summary);
    out.push('\n');
    for section in &report.sections {
        out.push_str(&format!("\n## {}\n\n", section.heading));
        out.push_str(&section.body_markdown);
    }
    if !report.citations.is_empty() {
        out.push_str("\n## References\n\n");
        for c in &report.citations {
            out.push_str(&format!(
                "{}. {} — {}\n",
                c.reference_number, c.title, c.url
            ));
        }
    }
    out
}

fn render_obsidian(report: &ResearchReport) -> String {
    let mut out = String::new();
    out.push_str(&report.summary);
    out.push('\n');
    for section in &report.sections {
        out.push_str(&format!("\n## {}\n\n", section.heading));
        out.push_str(&section.body_markdown);
    }
    if !report.citations.is_empty() {
        out.push_str("\n## References\n\n");
        for c in &report.citations {
            out.push_str(&format!(
                "{}. {} — {}\n",
                c.reference_number, c.title, c.url
            ));
        }
    }
    out
}

pub async fn run_headless(query: &str, output: Option<&Path>) -> Result<()> {
    let config = crate::infrastructure::config::load();
    let obs = crate::infrastructure::observability::Observability::init(
        "muon",
        &config.observability,
    )?;

    let result = async {
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = BridgeChannels::new(tx);

        let infra = InfrastructureContext::new_live(&config, &bridge).await?;

        let mut state = PipelineState::default();
        let deps = crate::application::deps::PipelineDeps::from_infra(&infra);
        let report = run_pipeline(query, &mut state, &config, &deps, &bridge, None).await?;

        let content = render_markdown(&report, query);

        if let Some(path) = output {
            std::fs::write(path, &content)?;
            eprintln!("Report written to {}", path.display());
        } else {
            print!("{content}");
        }

        Ok(())
    }
    .await;

    obs.shutdown().await?;
    result
}

fn session_from_summary(summary: &SessionSummary) -> Session {
    Session {
        id: summary.id,
        query: summary.query.clone(),
        status: SessionStatus::Complete,
        ..Session::default()
    }
}

pub async fn export_session(
    session_id: &str,
    format: ExportFormat,
    output: Option<&Path>,
) -> Result<()> {
    let config = crate::infrastructure::config::load();
    let pool = init_pool(&config.advanced.session_db_path).await?;

    let sid: SessionId = session_id.parse().map_err(|e: uuid::Error| {
        MuonError::Session(format!("invalid session ID: {session_id}: {e}"))
    })?;

    let session_store = DieselSessionStore::new(pool);

    let summary = session_store
        .get(sid)
        .await?
        .ok_or_else(|| MuonError::Session(format!("session not found: {session_id}")))?;

    let report = session_store
        .get_report(sid)
        .await?
        .ok_or_else(|| MuonError::Session(format!("report not found for session: {session_id}")))?;

    let session = session_from_summary(&summary);

    let content = match format {
        ExportFormat::Markdown => render_markdown(&report, &summary.query),
        ExportFormat::Obsidian => {
            let vault_str = if !config.obsidian.vault_path.is_empty() {
                crate::infrastructure::util::expand_tilde(&config.obsidian.vault_path)
                    .to_string_lossy()
                    .into_owned()
            } else {
                std::env::var("MUON_OBSIDIAN_VAULT").map_err(|_| {
                    MuonError::Config(
                        "Obsidian vault not configured: set [obsidian] vault_path in config.toml or MUON_OBSIDIAN_VAULT env"
                            .into(),
                    )
                })?
            };
            let vault_path = std::path::PathBuf::from(vault_str);
            ObsidianExporter::export(&report, &session, &vault_path)?;
            render_obsidian(&report)
        }
        ExportFormat::Pdf => {
            let pdf_path = match output {
                Some(out) => PdfExporter::export_to_path(&report, &session, out)?,
                None => PdfExporter::export(&report, &session)?,
            };
            eprintln!("PDF export written to {}", pdf_path.display());
            return Ok(());
        }
    };

    if let Some(path) = output {
        std::fs::write(path, &content)?;
        eprintln!("Export written to {}", path.display());
    } else {
        print!("{content}");
    }

    Ok(())
}
