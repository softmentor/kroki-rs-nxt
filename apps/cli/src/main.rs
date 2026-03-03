//! kroki-cli: Interactive terminal interface for the kroki diagram platform.
//!
//! Supports:
//! - Single diagram conversion
//! - Batch conversion
//! - Interactive TUI mode (Ratatui) — Phase 3

use clap::Parser;
use kroki_cli::Cli;
use tracing_subscriber::fmt::time::OffsetTime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let timer_format =
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let timer = OffsetTime::new(offset, timer_format);
    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(env_filter)
        .init();

    tracing::info!(
        surface = "cli",
        version = env!("CARGO_PKG_VERSION"),
        "starting surface"
    );

    let cli = Cli::parse();
    kroki_cli::handle_command(cli.command).await;

    Ok(())
}
