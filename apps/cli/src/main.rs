//! kroki-cli: Interactive terminal interface for the kroki diagram platform.
//!
//! Supports:
//! - Single diagram conversion
//! - Batch conversion
//! - Interactive TUI mode (Ratatui) — Phase 3
//! - Server startup delegation

use clap::Parser;
use kroki_cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    kroki_cli::handle_command(cli.command).await;

    Ok(())
}
