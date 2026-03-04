//! Port definitions (traits) for the diagram domain.
//!
//! Ports define the interfaces that adapters implement.

use std::fmt;
use std::str::FromStr;

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

impl OutputFormat {
    /// Return the MIME content type for this format.
    pub fn content_type(&self) -> &'static str {
        match self {
            OutputFormat::Svg => "image/svg+xml",
            OutputFormat::Png => "image/png",
            OutputFormat::WebP => "image/webp",
            OutputFormat::Pdf => "application/pdf",
        }
    }

    /// Return the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Svg => "svg",
            OutputFormat::Png => "png",
            OutputFormat::WebP => "webp",
            OutputFormat::Pdf => "pdf",
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.extension())
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "svg" => Ok(OutputFormat::Svg),
            "png" => Ok(OutputFormat::Png),
            "webp" => Ok(OutputFormat::WebP),
            "pdf" => Ok(OutputFormat::Pdf),
            other => Err(format!("unsupported output format: '{other}'")),
        }
    }
}

/// Request to generate a diagram.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DiagramRequest {
    pub source: String,
    pub diagram_type: String,
    pub output_format: OutputFormat,
    pub options: DiagramOptions,
}

/// Optional parameters for diagram generation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DiagramOptions {
    pub font_urls: Vec<String>,
    pub timeout_ms: Option<u64>,
}

/// Response from diagram generation.
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
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
