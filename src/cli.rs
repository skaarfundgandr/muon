use std::path::Path;

use tokio::sync::mpsc;

use crate::application::bridge::BridgeChannels;
use crate::application::pipeline::PipelineState;
use crate::application::pipeline_runner::run_pipeline;
use crate::application::services::{ExportFormat, ObsidianExporter};
use crate::config::MuonConfig;
use crate::domain::models::report::ResearchReport;
use crate::domain::models::{Session, SessionId, SessionStatus};
use crate::domain::traits::session_store::{SessionStore, SessionSummary};
use crate::domain::error::{MuonError, Result};
use crate::infrastructure::context::InfrastructureContext;
use crate::infrastructure::storage::{init_pool, DieselSessionStore, ReportStore};

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
            out.push_str(&format!("{}. {} — {}\n", c.reference_number, c.title, c.url));
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
            out.push_str(&format!("{}. {} — {}\n", c.reference_number, c.title, c.url));
        }
    }
    out
}

pub async fn run_headless(query: &str, mock: bool, output: Option<&Path>) -> Result<()> {
    let obs = crate::infrastructure::observability::Observability::init("muon")?;

    let result = async {
        let config = MuonConfig::load();
        let (tx, _rx) = mpsc::unbounded_channel();
        let bridge = BridgeChannels::new(tx);

        let infra = if mock {
            #[cfg(any(test, feature = "mock"))]
            {
                InfrastructureContext::mock()
            }
            #[cfg(not(any(test, feature = "mock")))]
            {
                return Err(MuonError::Config(
                    "mock mode requires building with the 'mock' feature".to_string(),
                ));
            }
        } else {
            InfrastructureContext::new_live(&config, &bridge).await?
        };

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
    let config = MuonConfig::load();
    let pool = init_pool(&config.advanced.session_db_path).await?;

    let sid: SessionId = session_id
        .parse()
        .map_err(|e: uuid::Error| MuonError::Session(format!("invalid session ID: {session_id}: {e}")))?;

    let report_store = ReportStore::new(pool.clone());
    let session_store = DieselSessionStore::new(pool);

    let summary = session_store
        .get(sid)
        .await?
        .ok_or_else(|| MuonError::Session(format!("session not found: {session_id}")))?;

    let report = report_store
        .get_for_session(session_id)
        .await?
        .ok_or_else(|| MuonError::Session(format!("report not found for session: {session_id}")))?;

    let session = session_from_summary(&summary);

    let content = match format {
        ExportFormat::Markdown => render_markdown(&report, &summary.query),
        ExportFormat::Obsidian => {
            let vault = std::env::var("MUON_OBSIDIAN_VAULT")
                .map_err(|_| MuonError::Config("MUON_OBSIDIAN_VAULT not set".to_string()))?;
            let vault_path = std::path::PathBuf::from(vault);
            ObsidianExporter::export(&report, &session, &vault_path)?;
            render_obsidian(&report)
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
