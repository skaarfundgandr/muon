use std::path::PathBuf;

use clap::{Parser, Subcommand};
use muon::application::services::ExportFormat;

#[derive(Parser)]
#[command(name = "muon", version, about = "Terminal-based deep research agent")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a research query
    Run {
        /// The research query
        query: String,
        /// Run without TUI; print report to stdout
        #[arg(long)]
        headless: bool,
        /// Use mock infrastructure (no API calls). Default unless MUON_LIVE=1
        #[arg(long)]
        mock: bool,
        /// Write report to file instead of stdout (headless only)
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// Export a previous session's report
    Export {
        /// Session ID (UUID)
        session: String,
        /// Output format: markdown | obsidian
        format: ExportFormat,
        /// Output file path
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// Launch the TUI (default if no subcommand)
    Tui,
}

#[tokio::main]
async fn main() -> muon::error::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Tui) | None => muon::app::run().await,
        Some(Commands::Run {
            query,
            headless,
            mock,
            output,
        }) => {
            if headless {
                muon::cli::run_headless(&query, mock, output.as_deref()).await
            } else {
                muon::app::run().await
            }
        }
        Some(Commands::Export {
            session,
            format,
            output,
        }) => muon::cli::export_session(&session, format, output.as_deref()).await,
    }
}
