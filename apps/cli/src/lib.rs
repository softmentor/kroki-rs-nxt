//! Public CLI surface for kroki-cli.

use clap::{Command, CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "kroki", about = "Diagram generation platform")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Start the diagram server
    Serve,
    /// Convert a diagram file
    Convert,
    /// Batch convert diagram files
    Batch,
}

pub fn command() -> Command {
    Cli::command()
}

pub fn handle_command(command: Option<Commands>) {
    match command {
        Some(Commands::Serve) => {
            tracing::info!("Server mode bootstrap baseline");
        }
        Some(Commands::Convert) => {
            tracing::info!("Convert mode bootstrap baseline");
        }
        Some(Commands::Batch) => {
            tracing::info!("Batch mode bootstrap baseline");
        }
        None => {
            tracing::info!("kroki-rs-nxt — run with --help for usage");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_cli_command_is_well_formed() {
        super::command().debug_assert();
    }
}
