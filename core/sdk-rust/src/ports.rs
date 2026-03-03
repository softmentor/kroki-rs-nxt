//! Port definitions (traits) for the diagram domain.
//!
//! Ports define the interfaces that adapters implement.

use async_trait::async_trait;

use crate::error::DiagramResult;

/// Supported output formats for diagram generation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum OutputFormat {
    Svg,
    Png,
    WebP,
    Pdf,
}

/// Request to generate a diagram.
#[derive(Debug, Clone)]
pub struct DiagramRequest {
    pub source: String,
    pub diagram_type: String,
    pub output_format: OutputFormat,
    pub options: DiagramOptions,
}

/// Optional parameters for diagram generation.
#[derive(Debug, Clone, Default)]
pub struct DiagramOptions {
    pub font_urls: Vec<String>,
    pub timeout_ms: Option<u64>,
}

/// Response from diagram generation.
#[derive(Debug)]
pub struct DiagramResponse {
    pub data: Vec<u8>,
    pub content_type: String,
    pub duration_ms: u64,
}

/// The central port: every diagram type implements this trait.
#[async_trait]
pub trait DiagramProvider: Send + Sync {
    /// Validate the diagram source before generation.
    fn validate(&self, source: &str) -> DiagramResult<()>;

    /// Generate a diagram from the given request.
    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse>;

    /// Return the list of output formats this provider supports.
    fn supported_formats(&self) -> &[OutputFormat];
}
