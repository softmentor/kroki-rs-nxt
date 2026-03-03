//! Public CLI surface for kroki-cli.

use std::sync::Arc;

use clap::{Command, CommandFactory, Parser};
use kroki_adapter_transport::{render_diagram, RenderRequestDto, RenderResponseDto};
use kroki_core::{DiagramRegistry, EchoProvider, OutputFormat};

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

pub async fn run_convert_bootstrap() -> anyhow::Result<RenderResponseDto> {
    let mut registry = DiagramRegistry::new();
    registry.register("echo", Arc::new(EchoProvider::new()));

    let request = RenderRequestDto {
        source: "A -> B".to_string(),
        diagram_type: "echo".to_string(),
        output_format: OutputFormat::Svg,
    };

    let response = render_diagram(&registry, request).await?;
    Ok(response)
}

pub async fn handle_command(command: Option<Commands>) {
    match command {
        Some(Commands::Serve) => {
            tracing::info!("Server mode bootstrap baseline");
        }
        Some(Commands::Convert) => match run_convert_bootstrap().await {
            Ok(response) => {
                tracing::info!(
                    "Convert mode bootstrap vertical slice complete: content_type={}",
                    response.content_type
                );
            }
            Err(err) => {
                tracing::error!("Convert bootstrap flow failed: {err}");
            }
        },
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
    #[tokio::test]
    async fn unit_convert_bootstrap_returns_svg_contract() {
        let response = super::run_convert_bootstrap()
            .await
            .expect("convert bootstrap flow should succeed");
        assert_eq!(response.content_type, "image/svg+xml");
        assert!(response.data.contains("bootstrap-echo:echo:A -> B"));
    }

    #[test]
    fn unit_cli_command_is_well_formed() {
        super::command().debug_assert();
    }
}
