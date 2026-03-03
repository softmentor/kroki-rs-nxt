//! Public CLI surface for kroki-cli.

use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Command, CommandFactory, Parser, ValueEnum};
use clap_complete::Generator;
use kroki_adapter_transport::{
    render_diagram, PayloadEncoding, RenderRequestDto, RenderResponseDto,
};
use kroki_core::{
    BpmnProvider, D2Provider, DiagramRegistry, EchoProvider, GraphvizProvider, MermaidProvider,
    OutputFormat, ProviderCategory, ProviderMetadata, RuntimeDependency,
};

#[derive(Parser)]
#[command(
    name = "kroki",
    version,
    about = "Diagram generation platform CLI",
    long_about = "kroki - diagram conversion command line interface\n\n\
SYNOPSIS:\n\
  kroki <command> [options]\n\n\
DESCRIPTION:\n\
  Validates input, selects a provider, and renders diagrams through the same\n\
  core contracts used by the server surface.\n\n\
  Use `kroki convert --help` for conversion options and examples.",
    after_long_help = "EXAMPLES:\n\
  kroki convert --diagram-type graphviz --source 'digraph G { A -> B; }'\n\
  kroki convert --diagram-type d2 --source 'a -> b'\n\
  kroki convert --diagram-type mermaid --source 'graph TD; A-->B;'\n\
  kroki convert --diagram-type bpmn --input ./process.bpmn\n\
  kroki convert -t graphviz -i ./sample.dot -o ./sample.svg\n\
  kroki completions --shell zsh --output ~/.local/share/kroki/completions"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Convert a diagram file
    Convert(ConvertArgs),
    /// Generate shell completion scripts
    Completions(CompletionArgs),
    /// Batch convert diagram files
    Batch,
}

#[derive(Debug, Clone, Args)]
pub struct ConvertArgs {
    /// Diagram provider type (current: graphviz, d2, mermaid, bpmn, echo)
    #[arg(short = 't', long = "diagram-type", default_value = "graphviz")]
    pub diagram_type: String,

    /// Inline diagram source
    #[arg(short = 's', long = "source", conflicts_with = "input")]
    pub source: Option<String>,

    /// Path to input source file
    #[arg(short = 'i', long = "input", conflicts_with = "source")]
    pub input: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long = "format", default_value = "svg")]
    pub format: CliOutputFormat,

    /// Output file path; when omitted, auto-writes to ./kroki-<diagram_type>-<ts>.<ext>
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct CompletionArgs {
    /// Shell type
    #[arg(long = "shell", value_enum)]
    pub shell: CompletionShell,

    /// Output directory (when omitted, writes to stdout)
    #[arg(long = "output")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliOutputFormat {
    Svg,
    Png,
    Webp,
    Pdf,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::Svg => OutputFormat::Svg,
            CliOutputFormat::Png => OutputFormat::Png,
            CliOutputFormat::Webp => OutputFormat::WebP,
            CliOutputFormat::Pdf => OutputFormat::Pdf,
        }
    }
}

impl CliOutputFormat {
    fn file_extension(self) -> &'static str {
        match self {
            CliOutputFormat::Svg => "svg",
            CliOutputFormat::Png => "png",
            CliOutputFormat::Webp => "webp",
            CliOutputFormat::Pdf => "pdf",
        }
    }
}

pub fn command() -> Command {
    Cli::command()
}

fn write_completion_script<G: Generator>(
    generator: G,
    output: Option<&PathBuf>,
) -> anyhow::Result<()> {
    let mut cmd = command();
    match output {
        Some(dir) => {
            let path = clap_complete::generate_to(generator, &mut cmd, "kroki", dir)?;
            tracing::info!(path = %path.display(), "wrote shell completion");
        }
        None => {
            clap_complete::generate(generator, &mut cmd, "kroki", &mut io::stdout());
        }
    }
    Ok(())
}

pub fn run_completions(args: &CompletionArgs) -> anyhow::Result<()> {
    match args.shell {
        CompletionShell::Bash => {
            write_completion_script(clap_complete::shells::Bash, args.output.as_ref())
        }
        CompletionShell::Elvish => {
            write_completion_script(clap_complete::shells::Elvish, args.output.as_ref())
        }
        CompletionShell::Fish => {
            write_completion_script(clap_complete::shells::Fish, args.output.as_ref())
        }
        CompletionShell::PowerShell => {
            write_completion_script(clap_complete::shells::PowerShell, args.output.as_ref())
        }
        CompletionShell::Zsh => {
            write_completion_script(clap_complete::shells::Zsh, args.output.as_ref())
        }
    }
}

fn build_registry() -> DiagramRegistry {
    let mut registry = DiagramRegistry::new();
    registry.register("echo", Arc::new(EchoProvider::new()));
    registry.register_with_metadata(
        "graphviz",
        Arc::new(GraphvizProvider::new()),
        ProviderMetadata {
            provider_id: "graphviz".to_string(),
            category: ProviderCategory::Command,
            runtime: RuntimeDependency::SystemTool {
                binary: "dot".to_string(),
            },
            supported_formats: vec![OutputFormat::Svg],
            description: "Graphviz command provider".to_string(),
        },
    );
    registry.register_with_metadata(
        "d2",
        Arc::new(D2Provider::new()),
        ProviderMetadata {
            provider_id: "d2".to_string(),
            category: ProviderCategory::Command,
            runtime: RuntimeDependency::SystemTool {
                binary: "d2".to_string(),
            },
            supported_formats: vec![OutputFormat::Svg],
            description: "D2 command provider".to_string(),
        },
    );
    registry.register_with_metadata(
        "mermaid",
        Arc::new(MermaidProvider::new()),
        ProviderMetadata {
            provider_id: "mermaid".to_string(),
            category: ProviderCategory::Browser,
            runtime: RuntimeDependency::BrowserEngine,
            supported_formats: vec![OutputFormat::Svg],
            description: "Mermaid browser provider (feature-gated native-browser path)".to_string(),
        },
    );
    registry.register_with_metadata(
        "bpmn",
        Arc::new(BpmnProvider::new()),
        ProviderMetadata {
            provider_id: "bpmn".to_string(),
            category: ProviderCategory::Browser,
            runtime: RuntimeDependency::BrowserEngine,
            supported_formats: vec![OutputFormat::Svg],
            description: "BPMN browser provider (native-browser runtime pending)".to_string(),
        },
    );
    registry
}

fn load_convert_source(args: &ConvertArgs) -> anyhow::Result<String> {
    if let Some(source) = &args.source {
        return Ok(source.clone());
    }
    if let Some(path) = &args.input {
        let source = fs::read_to_string(path)?;
        return Ok(source);
    }
    Ok("digraph G { A -> B; }".to_string())
}

fn resolve_output_path(args: &ConvertArgs) -> PathBuf {
    if let Some(path) = &args.output {
        return path.clone();
    }
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ext = args.format.file_extension();
    let provider = args.diagram_type.replace('/', "-");
    PathBuf::from(format!("kroki-{provider}-{ts}.{ext}"))
}

pub async fn run_convert(args: &ConvertArgs) -> anyhow::Result<(RenderResponseDto, PathBuf)> {
    let registry = build_registry();
    let source = load_convert_source(args)?;

    let request = RenderRequestDto {
        source: source.clone(),
        source_encoded: None,
        source_encoding: PayloadEncoding::Plain,
        diagram_type: args.diagram_type.clone(),
        output_format: args.format.into(),
    };

    let response = match render_diagram(&registry, request).await {
        Ok(response) => response,
        Err(kroki_core::DiagramError::ToolNotFound(_)) => {
            if args.diagram_type != "graphviz" && args.diagram_type != "d2" {
                return Err(kroki_core::DiagramError::ToolNotFound(format!(
                    "provider '{}' is unavailable in current runtime configuration",
                    args.diagram_type
                ))
                .into());
            }
            tracing::warn!(
                diagram_type = %args.diagram_type,
                "provider binary not found; falling back to echo provider"
            );
            render_diagram(
                &registry,
                RenderRequestDto {
                    source: source.clone(),
                    source_encoded: None,
                    source_encoding: PayloadEncoding::Plain,
                    diagram_type: "echo".to_string(),
                    output_format: OutputFormat::Svg,
                },
            )
            .await?
        }
        Err(err) => return Err(err.into()),
    };

    let output = resolve_output_path(args);
    fs::write(&output, response.data.as_bytes())?;
    tracing::info!(path = %output.display(), "wrote rendered output");

    Ok((response, output))
}

pub async fn handle_command(command: Option<Commands>) {
    match command {
        Some(Commands::Convert(args)) => match run_convert(&args).await {
            Ok((response, output)) => {
                tracing::debug!(content_type = %response.content_type, "convert response received");
                tracing::info!(
                    "Convert mode bootstrap vertical slice complete: content_type={}",
                    response.content_type
                );
                println!("output: {}", output.display());
            }
            Err(err) => {
                tracing::error!(error = %err, "convert command failed");
                tracing::error!("Convert bootstrap flow failed: {err}");
            }
        },
        Some(Commands::Completions(args)) => {
            if let Err(err) = run_completions(&args) {
                tracing::error!(error = %err, "completion generation failed");
            }
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
    use std::path::PathBuf;

    #[tokio::test]
    async fn unit_convert_bootstrap_returns_svg_contract() {
        let output = std::env::temp_dir().join("kroki-cli-test-graphviz.svg");
        let response = super::run_convert(&super::ConvertArgs {
            diagram_type: "graphviz".to_string(),
            source: None,
            input: None,
            format: super::CliOutputFormat::Svg,
            output: Some(output),
        })
        .await
        .expect("convert bootstrap flow should succeed");
        assert_eq!(response.0.content_type, "image/svg+xml");
        assert!(
            response.0.data.contains("<svg") || response.0.data.contains("bootstrap-echo:echo:")
        );
    }

    #[tokio::test]
    async fn unit_convert_mermaid_reports_runtime_unavailable() {
        let result = super::run_convert(&super::ConvertArgs {
            diagram_type: "mermaid".to_string(),
            source: Some("graph TD; A-->B;".to_string()),
            input: None,
            format: super::CliOutputFormat::Svg,
            output: Some(PathBuf::from("should-not-be-created.svg")),
        })
        .await;
        if which::which("mmdc").is_ok() {
            let ok = result.expect("mermaid should render when mmdc is installed");
            assert_eq!(ok.0.content_type, "image/svg+xml");
        } else {
            let err = result.expect_err("mermaid should report runtime unavailability");
            assert!(
                err.to_string()
                    .contains("provider 'mermaid' is unavailable"),
                "error should explain runtime unavailability"
            );
        }
    }

    #[test]
    fn unit_cli_command_is_well_formed() {
        super::command().debug_assert();
    }
}
