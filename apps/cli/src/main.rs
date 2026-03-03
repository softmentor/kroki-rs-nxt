//! kroki-cli: Interactive terminal interface for the kroki diagram platform.
//!
//! Supports:
//! - Single diagram conversion
//! - Batch conversion
//! - Interactive TUI mode (Ratatui) — Phase 3
//! - Server startup delegation

use clap::Parser;

#[derive(Parser)]
#[command(name = "kroki", about = "Diagram generation platform")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Start the diagram server
    Serve,
    /// Convert a diagram file
    Convert,
    /// Batch convert diagram files
    Batch,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Serve) => {
            tracing::info!("Server mode not yet implemented");
        }
        Some(Commands::Convert) => {
            tracing::info!("Convert mode not yet implemented");
        }
        Some(Commands::Batch) => {
            tracing::info!("Batch mode not yet implemented");
        }
        None => {
            tracing::info!("kroki-rs-nxt — run with --help for usage");
        }
    }

    Ok(())
}
