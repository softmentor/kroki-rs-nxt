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
async fn browser_manager() -> DiagramResult<Arc<BrowserManager>> {
    static MANAGER: OnceCell<Arc<BrowserManager>> = OnceCell::const_new();
    MANAGER
        .get_or_try_init(|| async move {
            let config = crate::config::Config::load(None).unwrap_or_default().browser;
            BrowserManager::start(config.pool_size, config.context_ttl_requests, &config.engine_urls)
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
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><!-- bootstrap-echo:{}:{} --></svg>"#,
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
            let start = std::time::Instant::now();
            let font_css = None; // BPMN doesn't use standard font injection right now
            match browser_manager().await {
                Ok(manager) => match manager
                    .evaluate("bpmn", &request.source, "svg", font_css)
                    .await
                {
                    Ok(rendered) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        info!(
                            provider = "bpmn",
                            backend = "headless_chrome",
                            duration_ms,
                            "bpmn render completed"
                        );
                        Ok(DiagramResponse {
                            data: rendered,
                            content_type: "image/svg+xml".to_string(),
                            duration_ms,
                        })
                    }
                    Err(err) => {
                        error!(
                            provider = "bpmn",
                            backend = "headless_chrome",
                            error = %err,
                            "native browser render failed"
                        );
                        Err(err)
                    }
                },
                Err(err) => {
                    error!(
                        provider = "bpmn",
                        backend = "headless_chrome",
                        error = %err,
                        "browser manager unavailable"
                    );
                    Err(DiagramError::Internal(format!("browser disconnected: {err}")))
                }
            }
        }
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        BPMN_SUPPORTED_FORMATS
    }
}

const DITAA_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Png, OutputFormat::Svg];
const EXCALIDRAW_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];
const WAVEDROM_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];

/// Command-based Ditaa provider for Batch 3.1 parity progression.
pub struct DitaaProvider {
    binary: String,
    default_timeout_ms: u64,
}

impl DitaaProvider {
    pub fn new() -> Self {
        Self {
            binary: "ditaa".to_string(),
            default_timeout_ms: 5_000,
        }
    }

    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            default_timeout_ms: 5_000,
        }
    }
}

impl Default for DitaaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for DitaaProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "ditaa source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg && request.output_format != OutputFormat::Png {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "ditaa".to_string(),
            });
        }

        let timeout_ms = request.options.timeout_ms.unwrap_or(self.default_timeout_ms);
        let temp_dir = tempfile::tempdir()?;
        let input_path = temp_dir.path().join("diagram.ditaa");
        let ext = if request.output_format == OutputFormat::Svg { "svg" } else { "png" };
        let output_path = temp_dir.path().join(format!("diagram.{ext}"));
        std::fs::write(&input_path, request.source.as_bytes())?;

        debug!(provider = "ditaa", timeout_ms, "starting ditaa command execution");
        let start = std::time::Instant::now();
        let mut child_cmd = Command::new(&self.binary);
        
        if request.output_format == OutputFormat::Svg {
            child_cmd.arg("--svg");
        }
        
        child_cmd.arg(input_path.as_os_str()).arg(output_path.as_os_str());

        let child = child_cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(provider = "ditaa", binary = %self.binary, "ditaa binary is not available on host");
                    DiagramError::ToolNotFound(self.binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "ditaa", timeout_ms, "ditaa command timed out");
                DiagramError::ExecutionTimeout { tool: self.binary.clone(), timeout_ms }
            })??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!(provider = "ditaa", stderr = %stderr, "ditaa process failed");
            return Err(DiagramError::ProcessFailed(stderr));
        }

        let rendered = std::fs::read(&output_path).map_err(|err| {
            DiagramError::Internal(format!(
                "ditaa succeeded but output file '{}' was unreadable: {err}",
                output_path.display()
            ))
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(provider = "ditaa", duration_ms, "ditaa render completed");

        let content_type = if request.output_format == OutputFormat::Svg {
            "image/svg+xml".to_string()
        } else {
            "image/png".to_string()
        };

        Ok(DiagramResponse {
            data: rendered,
            content_type,
            duration_ms,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        DITAA_SUPPORTED_FORMATS
    }
}

/// Command-based Excalidraw provider for Batch 3.1 parity progression.
pub struct ExcalidrawProvider {
    binary: String,
    default_timeout_ms: u64,
}

impl ExcalidrawProvider {
    pub fn new() -> Self {
        Self {
            binary: "excalidraw".to_string(),
            default_timeout_ms: 5_000,
        }
    }

    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            default_timeout_ms: 5_000,
        }
    }
}

impl Default for ExcalidrawProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for ExcalidrawProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "excalidraw source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "excalidraw".to_string(),
            });
        }

        let timeout_ms = request.options.timeout_ms.unwrap_or(self.default_timeout_ms);
        debug!(provider = "excalidraw", timeout_ms, "starting excalidraw command execution");

        let start = std::time::Instant::now();
        let mut child = Command::new(&self.binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(provider = "excalidraw", binary = %self.binary, "excalidraw binary is not available on host");
                    DiagramError::ToolNotFound(self.binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(request.source.as_bytes()).await?;
        }

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "excalidraw", timeout_ms, "excalidraw command timed out");
                DiagramError::ExecutionTimeout { tool: self.binary.clone(), timeout_ms }
            })??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!(provider = "excalidraw", stderr = %stderr, "excalidraw process failed");
            return Err(DiagramError::ProcessFailed(stderr));
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(provider = "excalidraw", duration_ms, "excalidraw render completed");

        Ok(DiagramResponse {
            data: output.stdout,
            content_type: "image/svg+xml".to_string(),
            duration_ms,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        EXCALIDRAW_SUPPORTED_FORMATS
    }
}


/// Command-based Wavedrom provider for Batch 3.1 parity progression.
pub struct WavedromProvider {
    binary: String,
    default_timeout_ms: u64,
}

impl WavedromProvider {
    pub fn new() -> Self {
        Self {
            binary: "wavedrom-cli".to_string(),
            default_timeout_ms: 5_000,
        }
    }

    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            default_timeout_ms: 5_000,
        }
    }
}

impl Default for WavedromProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for WavedromProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "wavedrom source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "wavedrom".to_string(),
            });
        }

        let timeout_ms = request.options.timeout_ms.unwrap_or(self.default_timeout_ms);
        let temp_dir = tempfile::tempdir()?;
        let input_path = temp_dir.path().join("diagram.json");
        let output_path = temp_dir.path().join("diagram.svg");
        std::fs::write(&input_path, request.source.as_bytes())?;

        debug!(provider = "wavedrom", timeout_ms, "starting wavedrom command execution");
        let start = std::time::Instant::now();
        let child = Command::new(&self.binary)
            .arg("-i")
            .arg(input_path.as_os_str())
            .arg("-s")
            .arg(output_path.as_os_str())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(provider = "wavedrom", binary = %self.binary, "wavedrom binary is not available on host");
                    DiagramError::ToolNotFound(self.binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "wavedrom", timeout_ms, "wavedrom command timed out");
                DiagramError::ExecutionTimeout { tool: self.binary.clone(), timeout_ms }
            })??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!(provider = "wavedrom", stderr = %stderr, "wavedrom process failed");
            return Err(DiagramError::ProcessFailed(stderr));
        }

        let rendered = std::fs::read(&output_path).map_err(|err| {
            DiagramError::Internal(format!(
                "wavedrom succeeded but output file '{}' was unreadable: {err}",
                output_path.display()
            ))
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(provider = "wavedrom", duration_ms, "wavedrom render completed");

        Ok(DiagramResponse {
            data: rendered,
            content_type: "image/svg+xml".to_string(),
            duration_ms,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        WAVEDROM_SUPPORTED_FORMATS
    }
}

const VEGA_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];
const VEGALITE_SUPPORTED_FORMATS: &[OutputFormat] = &[OutputFormat::Svg];

/// Pipeline-based Vega provider mapping `vg2svg`.
pub struct VegaProvider {
    binary: String,
    default_timeout_ms: u64,
}

impl VegaProvider {
    pub fn new() -> Self {
        Self {
            binary: "vg2svg".to_string(),
            default_timeout_ms: 10_000,
        }
    }

    pub fn with_binary(binary: impl Into<String>) -> Self {
        Self {
            binary: binary.into(),
            default_timeout_ms: 10_000,
        }
    }
}

impl Default for VegaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for VegaProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "vega source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "vega".to_string(),
            });
        }

        let timeout_ms = request.options.timeout_ms.unwrap_or(self.default_timeout_ms);
        debug!(provider = "vega", timeout_ms, "starting vg2svg execution");
        let start = std::time::Instant::now();

        let mut child = Command::new(&self.binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(provider = "vega", binary = %self.binary, "vg2svg binary is not available on host");
                    DiagramError::ToolNotFound(self.binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(request.source.as_bytes()).await?;
        }

        let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "vega", timeout_ms, "vega command timed out");
                DiagramError::ExecutionTimeout { tool: self.binary.clone(), timeout_ms }
            })??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            error!(provider = "vega", stderr = %stderr, "vega process failed");
            return Err(DiagramError::ProcessFailed(stderr));
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(provider = "vega", duration_ms, "vega render completed");

        Ok(DiagramResponse {
            data: output.stdout,
            content_type: request.output_format.content_type().to_string(),
            duration_ms,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        VEGA_SUPPORTED_FORMATS
    }
}

/// Pipeline-based Vega-Lite provider mapping `vl2vg` directly into `vg2svg`.
pub struct VegaLiteProvider {
    vl_binary: String,
    vg_binary: String,
    default_timeout_ms: u64,
}

impl VegaLiteProvider {
    pub fn new() -> Self {
        Self {
            vl_binary: "vl2vg".to_string(),
            vg_binary: "vg2svg".to_string(),
            default_timeout_ms: 10_000,
        }
    }

    pub fn with_binaries(vl_binary: impl Into<String>, vg_binary: impl Into<String>) -> Self {
        Self {
            vl_binary: vl_binary.into(),
            vg_binary: vg_binary.into(),
            default_timeout_ms: 10_000,
        }
    }
}

impl Default for VegaLiteProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DiagramProvider for VegaLiteProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> {
        if source.trim().is_empty() {
            return Err(DiagramError::ValidationFailed(
                "vegalite source must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> {
        self.validate(&request.source)?;

        if request.output_format != OutputFormat::Svg {
            return Err(DiagramError::UnsupportedFormat {
                format: format!("{:?}", request.output_format),
                provider: "vegalite".to_string(),
            });
        }

        let timeout_ms = request.options.timeout_ms.unwrap_or(self.default_timeout_ms);
        debug!(provider = "vegalite", timeout_ms, "starting vl2vg execution");
        let start = std::time::Instant::now();

        // Stage 1: vl2vg
        let mut vl_child = Command::new(&self.vl_binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(provider = "vegalite", binary = %self.vl_binary, "vl2vg binary is not available on host");
                    DiagramError::ToolNotFound(self.vl_binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        if let Some(stdin) = &mut vl_child.stdin {
            stdin.write_all(request.source.as_bytes()).await?;
        }

        let vl_output = timeout(Duration::from_millis(timeout_ms), vl_child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "vegalite", timeout_ms, stage="vl2vg", "vegalite stage 1 command timed out");
                DiagramError::ExecutionTimeout { tool: self.vl_binary.clone(), timeout_ms }
            })??;

        if !vl_output.status.success() {
            let stderr = String::from_utf8_lossy(&vl_output.stderr).trim().to_string();
            error!(provider = "vegalite", stderr = %stderr, "vegalite vl2vg process failed");
            return Err(DiagramError::ProcessFailed(format!("vl2vg failed: {stderr}")));
        }

        // Stage 2: vg2svg
        debug!(provider = "vegalite", timeout_ms, "starting vg2svg execution (stage 2)");
        let mut vg_child = Command::new(&self.vg_binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::NotFound {
                    warn!(provider = "vegalite", binary = %self.vg_binary, "vg2svg binary is not available on host");
                    DiagramError::ToolNotFound(self.vg_binary.clone())
                } else {
                    DiagramError::Io(err)
                }
            })?;

        if let Some(stdin) = &mut vg_child.stdin {
            stdin.write_all(&vl_output.stdout).await?;
        }

        let vg_output = timeout(Duration::from_millis(timeout_ms), vg_child.wait_with_output())
            .await
            .map_err(|_| {
                warn!(provider = "vegalite", timeout_ms, stage="vg2svg", "vegalite stage 2 command timed out");
                DiagramError::ExecutionTimeout { tool: self.vg_binary.clone(), timeout_ms }
            })??;

        if !vg_output.status.success() {
            let stderr = String::from_utf8_lossy(&vg_output.stderr).trim().to_string();
            error!(provider = "vegalite", stderr = %stderr, "vegalite vg2svg process failed");
            return Err(DiagramError::ProcessFailed(format!("vg2svg failed: {stderr}")));
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(provider = "vegalite", duration_ms, "vegalite render pipeline completed");

        Ok(DiagramResponse {
            data: vg_output.stdout,
            content_type: request.output_format.content_type().to_string(),
            duration_ms,
        })
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        VEGALITE_SUPPORTED_FORMATS
    }
}
