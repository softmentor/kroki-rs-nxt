//! Diagram provider implementations.
//!
//! Provider categories:
//! - Command: wraps CLI tools via subprocess (Graphviz, D2, Ditaa, etc.)
//! - Browser: evaluates JS in headless Chrome (Mermaid, BPMN)
//! - Pipeline: multi-step conversion chains (Vega-Lite → Vega → SVG)
//!
//! Bootstrap baseline provider module; concrete providers are planned for Phase 3.

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

#[cfg(feature = "native-browser")]
use crate::browser::BrowserManager;
use crate::error::{DiagramError, DiagramResult};
use crate::ports::{DiagramProvider, DiagramRequest, DiagramResponse, OutputFormat};
#[cfg(feature = "native-browser")]
use crate::utils::font_manager::FontManager;

#[cfg(feature = "native-browser")]
use std::sync::Arc;
#[cfg(feature = "native-browser")]
use tokio::sync::OnceCell;

/// Minimal bootstrap provider used to validate end-to-end request flow in Phase 2.
///
/// It returns a deterministic text payload and does not execute external tools.
pub struct EchoProvider;

impl EchoProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EchoProvider {
    fn default() -> Self {
        Self::new()
    }
}

const ECHO_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];
const GRAPHVIZ_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];
const D2_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];
const MERMAID_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];
const BPMN_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];

#[cfg(feature = "native-browser")]
fn browser_pool_size() -> usize {
    std::env::var("KROKI_BROWSER_POOL_SIZE")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|size| *size > 0)
        .unwrap_or(4)
}

#[cfg(feature = "native-browser")]
fn browser_context_ttl_requests() -> usize {
    std::env::var("KROKI_BROWSER_CONTEXT_TTL_REQUESTS")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|size| *size > 0)
        .unwrap_or(100)
}

#[cfg(feature = "native-browser")]
async fn browser_manager() -> DiagramResult<Arc<BrowserManager>> {
    static MANAGER: OnceCell<Arc<BrowserManager>> = OnceCell::const_new();
    let pool_size = browser_pool_size();
    let context_ttl = browser_context_ttl_requests();
    MANAGER
        .get_or_try_init(|| async move {
            BrowserManager::start(pool_size, context_ttl)
                .await
                .map(Arc::new)
                .map_err(|err| {
                    DiagramError::Internal(format!("browser manager startup failed: {err}"))
                })
        })
        .await
        .map(Arc::clone)
}

#[async_trait]
impl DiagramProvider for EchoProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "diagram source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "echo".to_string(),
            });
        }

        let payload = format!(
            "<svg><!-- bootstrap-echo:{}:{} --></svg>",
            request.diagram_type, request.source
        );

        Ok(DiagramResponse {
            data: payload.into_bytes(),
            content_type: "image/svg+xml".to_string(),
            duration_ms: 0,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        ECHO_SUPPORTED_FORMATS
    }
}

/// Command-based Graphviz provider for the first real system-tool vertical slice.
pub struct GraphvizProvider {
    dot_binary: String,
    default_timeout_ms: u64,
}

impl GraphvizProvider {
    pub fn new() -> Self {
        Self {
            dot_binary: "dot".to_string(),
            default_timeout_ms: 5_000,
        }
    }

    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            dot_binary: binary.into(),
            default_timeout_ms: 5_000,
        }
    }
}

impl Default for GraphvizProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for GraphvizProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "graphviz source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;
        let source = normalize_escaped_whitespace(&request.source);

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "graphviz".to_string(),
            });
        }

        let timeout_ms = request
            .options
            .timeout_ms
            .unwrap_or(self.default_timeout_ms);
        debug!(
            provider = "graphviz",
            timeout_ms, "starting graphviz command execution"
        );

        let start = std::time::Instant::now();
        let mut child = Command::new(&self.dot_binary)
            .arg("-Tsvg")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(
                        provider = "graphviz",
                        binary = %self.dot_binary,
                        "graphviz binary is not available on host"
                    );
                    DiagramError::ToolNotFound(self.dot_binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(source.as_bytes()).await?;
        }

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(
                    provider = "graphviz",
                    timeout_ms, "graphviz command timed out"
                );
                DiagramError::ExecutionTimeout {
                    tool: self.dot_binary.clone(),
                    timeout_ms,
                }
            })??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!(
                provider = "graphviz",
                stderr = %stderr,
                "graphviz process failed"
            );
            return Err(DiagramError::ProcessFailed(stderr));
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(
            provider = "graphviz",
            duration_ms, "graphviz render completed"
        );

        Ok(DiagramResponse {
            data: output.stdout,
            content_type: "image/svg+xml".to_string(),
            duration_ms,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        GRAPHVIZ_SUPPORTED_FORMATS
    }
}

fn normalize_escaped_whitespace(source: &str) -> String {
    source.replace("\\n", "\n").replace("\\t", "\t")
}

/// Command-based D2 provider for Batch 3.1 parity progression.
pub struct D2Provider {
    d2_binary: String,
    default_timeout_ms: u64,
}

impl D2Provider {
    pub fn new() -> Self {
        Self {
            d2_binary: "d2".to_string(),
            default_timeout_ms: 5_000,
        }
    }

    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            d2_binary: binary.into(),
            default_timeout_ms: 5_000,
        }
    }
}

impl Default for D2Provider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for D2Provider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "d2 source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "d2".to_string(),
            });
        }

        let timeout_ms = request
            .options
            .timeout_ms
            .unwrap_or(self.default_timeout_ms);
        let temp_dir = tempfile::tempdir()?;
        let input_path = temp_dir.path().join("diagram.d2");
        let output_path = temp_dir.path().join("diagram.svg");
        std::fs::write(&input_path, request.source.as_bytes())?;

        debug!(provider = "d2", timeout_ms, "starting d2 command execution");
        let start = std::time::Instant::now();
        let child = Command::new(&self.d2_binary)
            .arg(input_path.as_os_str())
            .arg(output_path.as_os_str())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(
                        provider = "d2",
                        binary = %self.d2_binary,
                        "d2 binary is not available on host"
                    );
                    DiagramError::ToolNotFound(self.d2_binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "d2", timeout_ms, "d2 command timed out");
                DiagramError::ExecutionTimeout {
                    tool: self.d2_binary.clone(),
                    timeout_ms,
                }
            })??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!(provider = "d2", stderr = %stderr, "d2 process failed");
            return Err(DiagramError::ProcessFailed(stderr));
        }

        let rendered = std::fs::read(&output_path).map_err(|err| {
            DiagramError::Internal(format!(
                "d2 succeeded but output file '{}' was unreadable: {err}",
                output_path.display()
            ))
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(provider = "d2", duration_ms, "d2 render completed");

        Ok(DiagramResponse {
            data: rendered,
            content_type: "image/svg+xml".to_string(),
            duration_ms,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        D2_SUPPORTED_FORMATS
    }
}

/// Browser-backed Mermaid provider baseline for Batch 3.2 groundwork.
///
/// In the default build profile, Mermaid rendering is disabled and reports a
/// feature-gating error. Enabling `native-browser` unlocks the browser path.
pub struct MermaidProvider {
    #[cfg_attr(not(feature = "native-browser"), allow(dead_code))]
    mmdc_binary: String,
    #[cfg_attr(not(feature = "native-browser"), allow(dead_code))]
    default_timeout_ms: u64,
}

impl MermaidProvider {
    pub fn new() -> Self {
        Self {
            mmdc_binary: "mmdc".to_string(),
            default_timeout_ms: 5_000,
        }
    }

    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            mmdc_binary: binary.into(),
            default_timeout_ms: 5_000,
        }
    }

    #[cfg_attr(not(feature = "native-browser"), allow(dead_code))]
    async fn render_with_mmdc(&self, source: &str, timeout_ms: u64) -> DiagramResult<Vec<u8>> {
        let temp_dir = tempfile::tempdir()?;
        let input_path = temp_dir.path().join("diagram.mmd");
        let output_path = temp_dir.path().join("diagram.svg");
        std::fs::write(&input_path, source.as_bytes())?;

        debug!(
            provider = "mermaid",
            timeout_ms, "starting mmdc command execution"
        );
        let child = Command::new(&self.mmdc_binary)
            .arg("-i")
            .arg(input_path.as_os_str())
            .arg("-o")
            .arg(output_path.as_os_str())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(
                        provider = "mermaid",
                        binary = %self.mmdc_binary,
                        "mermaid cli binary is not available on host"
                    );
                    DiagramError::ToolNotFound(self.mmdc_binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "mermaid", timeout_ms, "mmdc command timed out");
                DiagramError::ExecutionTimeout {
                    tool: self.mmdc_binary.clone(),
                    timeout_ms,
                }
            })??;

        if !output.status.success() {
            let stderr_text = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let message = if stderr_text.is_empty() {
                stdout_text
            } else {
                stderr_text
            };
            error!(provider = "mermaid", error = %message, "mmdc process failed");
            return Err(DiagramError::ProcessFailed(message));
        }

        std::fs::read(&output_path).map_err(|err| {
            DiagramError::Internal(format!(
                "mmdc succeeded but output file '{}' was unreadable: {err}",
                output_path.display()
            ))
        })
    }
}

impl Default for MermaidProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for MermaidProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "mermaid source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "mermaid".to_string(),
            });
        }

        #[cfg(not(feature = "native-browser"))]
        {
            return Err(DiagramError::ToolNotFound(
                "native-browser feature disabled for mermaid provider".to_string(),
            ));
        }

        #[cfg(feature = "native-browser")]
        {
            let timeout_ms = request
                .options
                .timeout_ms
                .unwrap_or(self.default_timeout_ms);
            let start = std::time::Instant::now();
            let prefer_native_browser = self.mmdc_binary == "mmdc";
            if prefer_native_browser {
                let font_css = match FontManager::new() {
                    Ok(manager) => match manager.prepare_font_css(&request.options.font_urls).await
                    {
                        Ok(css) => css,
                        Err(err) => {
                            warn!(provider = "mermaid", error = %err, "font preparation failed");
                            None
                        }
                    },
                    Err(err) => {
                        warn!(
                            provider = "mermaid",
                            error = %err,
                            "font manager initialization failed"
                        );
                        None
                    }
                };

                match browser_manager().await {
                    Ok(manager) => match manager
                        .evaluate("mermaid", &request.source, "svg", font_css.as_deref())
                        .await
                    {
                        Ok(rendered) => {
                            let duration_ms = start.elapsed().as_millis() as u64;
                            info!(
                                provider = "mermaid",
                                backend = "headless_chrome",
                                duration_ms,
                                "mermaid render completed"
                            );
                            return Ok(DiagramResponse {
                                data: rendered,
                                content_type: "image/svg+xml".to_string(),
                                duration_ms,
                            });
                        }
                        Err(err) => {
                            warn!(
                                provider = "mermaid",
                                backend = "headless_chrome",
                                error = %err,
                                "native browser render failed; falling back to mmdc"
                            );
                        }
                    },
                    Err(err) => {
                        warn!(
                            provider = "mermaid",
                            backend = "headless_chrome",
                            error = %err,
                            "browser manager unavailable; falling back to mmdc"
                        );
                    }
                }
            }

            let rendered = self.render_with_mmdc(&request.source, timeout_ms).await?;
            let duration_ms = start.elapsed().as_millis() as u64;
            info!(
                provider = "mermaid",
                backend = "mmdc",
                duration_ms,
                "mermaid render completed"
            );
            Ok(DiagramResponse {
                data: rendered,
                content_type: "image/svg+xml".to_string(),
                duration_ms,
            })
        }
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        MERMAID_SUPPORTED_FORMATS
    }
}

/// Browser-backed BPMN provider baseline.
///
/// The runtime command wiring is intentionally deferred to a follow-up slice.
pub struct BpmnProvider;

impl BpmnProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BpmnProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for BpmnProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "bpmn source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "bpmn".to_string(),
            });
        }

        #[cfg(not(feature = "native-browser"))]
        {
            return Err(DiagramError::ToolNotFound(
                "native-browser feature disabled for bpmn provider".to_string(),
            ));
        }

        #[cfg(feature = "native-browser")]
        {
            return Err(DiagramError::Internal(
                "bpmn browser runtime not wired yet".to_string(),
            ));
        }
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        BPMN_SUPPORTED_FORMATS
    }
}
