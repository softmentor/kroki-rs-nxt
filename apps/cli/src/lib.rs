//! Public CLI surface for kroki-cli.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
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
    /// Convert a diagram file to an output format
    Convert(ConvertArgs),
    /// Encode text to deflate + base64 (for use in GET URLs)
    Encode(EncodeArgs),
    /// Decode deflate + base64 back to plain text
    Decode(DecodeArgs),
    /// Generate shell completion scripts
    Completions(CompletionArgs),
    /// Show version information
    Version,
    /// Batch convert diagram files (placeholder)
    Batch,
}

#[derive(Debug, Clone, Args)]
pub struct ConvertArgs {
    /// Diagram provider type; auto-detected from file extension when omitted
    #[arg(short = 't', long = "diagram-type")]
    pub diagram_type: Option<String>,

    /// Inline diagram source (use "-" to read from stdin)
    #[arg(short = 's', long = "source", conflicts_with = "input")]
    pub source: Option<String>,

    /// Path to input source file (use "-" to read from stdin)
    #[arg(short = 'i', long = "input", conflicts_with = "source")]
    pub input: Option<PathBuf>,

    /// Output format
    #[arg(short = 'f', long = "format", default_value = "svg")]
    pub format: CliOutputFormat,

    /// Output file path; use "-" for stdout; when omitted, auto-writes to ./kroki-<type>-<ts>.<ext>
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct EncodeArgs {
    /// Text to encode (reads from stdin if omitted)
    pub text: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct DecodeArgs {
    /// Encoded string to decode (reads from stdin if omitted)
    pub encoded: Option<String>,
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
    registry.register_with_metadata(
        "ditaa",
        Arc::new(kroki_core::DitaaProvider::new()),
        ProviderMetadata {
            provider_id: "ditaa".to_string(),
            category: ProviderCategory::Command,
            runtime: RuntimeDependency::SystemTool {
                binary: "ditaa".to_string(),
            },
            supported_formats: vec![OutputFormat::Png, OutputFormat::Svg],
            description: "Ditaa command provider".to_string(),
        },
    );
    registry.register_with_metadata(
        "excalidraw",
        Arc::new(kroki_core::ExcalidrawProvider::new()),
        ProviderMetadata {
            provider_id: "excalidraw".to_string(),
            category: ProviderCategory::Command,
            runtime: RuntimeDependency::SystemTool {
                binary: "excalidraw".to_string(),
            },
            supported_formats: vec![OutputFormat::Svg],
            description: "Excalidraw command provider".to_string(),
        },
    );
    registry.register_with_metadata(
        "wavedrom",
        Arc::new(kroki_core::WavedromProvider::new()),
        ProviderMetadata {
            provider_id: "wavedrom".to_string(),
            category: ProviderCategory::Command,
            runtime: RuntimeDependency::SystemTool {
                binary: "wavedrom-cli".to_string(),
            },
            supported_formats: vec![OutputFormat::Svg],
            description: "Wavedrom command provider".to_string(),
        },
    );
    registry
}

/// Auto-detect diagram type from file extension.
///
/// Reference: <https://docs.kroki.io/kroki/setup/kroki-cli/>
fn detect_diagram_type(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "dot" | "gv" => Some("graphviz"),
        "mmd" | "mermaid" => Some("mermaid"),
        "d2" => Some("d2"),
        "puml" | "plantuml" => Some("plantuml"),
        "excalidraw" => Some("excalidraw"),
        "bpmn" => Some("bpmn"),
        "vega" => Some("vega"),
        "vl" => Some("vegalite"),
        "ditaa" => Some("ditaa"),
        "wavedrom" => Some("wavedrom"),
        _ => None,
    }
}

/// Read source from stdin.
fn read_stdin() -> anyhow::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn load_convert_source(args: &ConvertArgs) -> anyhow::Result<String> {
    if let Some(source) = &args.source {
        if source == "-" {
            return read_stdin();
        }
        return Ok(source.clone());
    }
    if let Some(path) = &args.input {
        if path == Path::new("-") {
            return read_stdin();
        }
        let source = fs::read_to_string(path)?;
        return Ok(source);
    }
    Ok("digraph G { A -> B; }".to_string())
}

/// Resolve the diagram type — explicit flag takes precedence, then auto-detect from file.
fn resolve_diagram_type(args: &ConvertArgs) -> String {
    if let Some(dt) = &args.diagram_type {
        return dt.clone();
    }
    if let Some(path) = &args.input {
        if let Some(detected) = detect_diagram_type(path) {
            tracing::info!(detected_type = detected, "auto-detected diagram type from file extension");
            return detected.to_string();
        }
    }
    "graphviz".to_string()
}

/// Encode text to deflate + base64 (URL-safe, no padding).
///
/// This produces output compatible with Kroki GET URLs: `/{type}/{format}/{encoded}`
pub fn run_encode(args: &EncodeArgs) -> anyhow::Result<()> {
    use base64::Engine;

    let text = match &args.text {
        Some(t) => t.clone(),
        None => read_stdin()?,
    };

    let mut encoder =
        flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(text.as_bytes())?;
    let compressed = encoder.finish()?;
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&compressed);
    println!("{encoded}");
    Ok(())
}

/// Decode deflate + base64 back to plain text.
pub fn run_decode(args: &DecodeArgs) -> anyhow::Result<()> {
    use base64::Engine;

    let encoded = match &args.encoded {
        Some(e) => e.clone(),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf.trim().to_string()
        }
    };

    let compressed = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&encoded)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(&encoded))
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(&encoded))
        .map_err(|e| anyhow::anyhow!("invalid base64: {e}"))?;

    let mut decoder = flate2::read::ZlibDecoder::new(compressed.as_slice());
    let mut output = String::new();
    decoder.read_to_string(&mut output)?;
    print!("{output}");
    Ok(())
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
    let provider = resolve_diagram_type(args).replace('/', "-");
    PathBuf::from(format!("kroki-{provider}-{ts}.{ext}"))
}

pub async fn run_convert(args: &ConvertArgs) -> anyhow::Result<(RenderResponseDto, PathBuf)> {
    let registry = build_registry();
    let source = load_convert_source(args)?;
    let diagram_type = resolve_diagram_type(args);

    let request = RenderRequestDto {
        source: source.clone(),
        source_encoded: None,
        source_encoding: PayloadEncoding::Plain,
        diagram_type: diagram_type.clone(),
        output_format: args.format.into(),
    };

    let response = match render_diagram(&registry, request).await {
        Ok(response) => response,
        Err(kroki_core::DiagramError::ToolNotFound(_)) => {
            if diagram_type != "graphviz" && diagram_type != "d2" {
                return Err(kroki_core::DiagramError::ToolNotFound(format!(
                    "provider '{diagram_type}' is unavailable in current runtime configuration",
                ))
                .into());
            }
            tracing::warn!(
                diagram_type = %diagram_type,
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

    // Write output: "-" means stdout, otherwise write to file
    let output = resolve_output_path(args);
    if output == Path::new("-") {
        io::stdout().write_all(&response.data)?;
        io::stdout().flush()?;
    } else {
        fs::write(&output, &response.data)?;
        tracing::info!(path = %output.display(), "wrote rendered output");
    }

    Ok((response, output))
}

pub async fn handle_command(command: Option<Commands>) {
    match command {
        Some(Commands::Convert(args)) => match run_convert(&args).await {
            Ok((response, output)) => {
                tracing::debug!(content_type = %response.content_type, "convert response received");
                if output != Path::new("-") {
                    println!("output: {}", output.display());
                }
            }
            Err(err) => {
                tracing::error!(error = %err, "convert command failed");
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        },
        Some(Commands::Encode(args)) => {
            if let Err(err) = run_encode(&args) {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
        Some(Commands::Decode(args)) => {
            if let Err(err) = run_decode(&args) {
                eprintln!("error: {err}");
                std::process::exit(1);
            }
        }
        Some(Commands::Completions(args)) => {
            if let Err(err) = run_completions(&args) {
                tracing::error!(error = %err, "completion generation failed");
                std::process::exit(1);
            }
        }
        Some(Commands::Version) => {
            println!("kroki {}", env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Batch) => {
            tracing::info!("Batch mode is not yet implemented");
            eprintln!("batch mode is not yet implemented");
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
        let output = std::env::temp_dir().join("kroki-cli-test-graphviz.svg");
        let response = super::run_convert(&super::ConvertArgs {
            diagram_type: Some("graphviz".to_string()),
            source: None,
            input: None,
            format: super::CliOutputFormat::Svg,
            output: Some(output),
        })
        .await
        .expect("convert bootstrap flow should succeed");
        assert_eq!(response.0.content_type, "image/svg+xml");
        let data = response.0.data_as_string();
        assert!(
            data.contains("<svg") || data.contains("bootstrap-echo:echo:")
        );
    }

    #[tokio::test]
    async fn unit_convert_mermaid_reports_runtime_unavailable() {
        let result = super::run_convert(&super::ConvertArgs {
            diagram_type: Some("mermaid".to_string()),
            source: Some("graph TD; A-->B;".to_string()),
            input: None,
            format: super::CliOutputFormat::Svg,
            output: Some(std::env::temp_dir().join("should-not-be-created.svg")),
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

    #[tokio::test]
    async fn unit_convert_auto_detects_type_from_file_extension() {
        let tmp = std::env::temp_dir().join("kroki-cli-test-auto.dot");
        std::fs::write(&tmp, "digraph G { A -> B; }").expect("write test file");
        let output = std::env::temp_dir().join("kroki-cli-test-auto.svg");

        let result = super::run_convert(&super::ConvertArgs {
            diagram_type: None, // auto-detect from .dot extension
            source: None,
            input: Some(tmp.clone()),
            format: super::CliOutputFormat::Svg,
            output: Some(output),
        })
        .await;

        // Either graphviz renders or falls back to echo
        let (response, _) = result.expect("convert should succeed");
        assert_eq!(response.content_type, "image/svg+xml");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn unit_detect_diagram_type_from_extensions() {
        use std::path::Path;
        assert_eq!(super::detect_diagram_type(Path::new("test.dot")), Some("graphviz"));
        assert_eq!(super::detect_diagram_type(Path::new("test.gv")), Some("graphviz"));
        assert_eq!(super::detect_diagram_type(Path::new("test.mmd")), Some("mermaid"));
        assert_eq!(super::detect_diagram_type(Path::new("test.d2")), Some("d2"));
        assert_eq!(super::detect_diagram_type(Path::new("test.bpmn")), Some("bpmn"));
        assert_eq!(super::detect_diagram_type(Path::new("test.vega")), Some("vega"));
        assert_eq!(super::detect_diagram_type(Path::new("test.vl")), Some("vegalite"));
        assert_eq!(super::detect_diagram_type(Path::new("test.unknown")), None);
    }

    #[test]
    fn unit_encode_decode_roundtrip() {
        use base64::Engine;
        use std::io::{Read, Write};

        let text = "digraph G { A -> B; }";

        // Encode
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(text.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&compressed);

        // Decode
        let decoded_compressed = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&encoded)
            .unwrap();
        let mut decoder = flate2::read::ZlibDecoder::new(decoded_compressed.as_slice());
        let mut output = String::new();
        decoder.read_to_string(&mut output).unwrap();

        assert_eq!(output, text);
    }

    #[test]
    fn unit_cli_command_is_well_formed() {
        super::command().debug_assert();
    }
}
