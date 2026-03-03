//! Diagram provider implementations.
//!
//! Provider categories:
//! - Command: wraps CLI tools via subprocess (Graphviz, D2, Ditaa, etc.)
//! - Browser: evaluates JS in headless Chrome (Mermaid, BPMN)
//! - Pipeline: multi-step conversion chains (Vega-Lite → Vega → SVG)
//!
//! Bootstrap baseline provider module; concrete providers are planned for Phase 3.

use async_trait::async_trait;

use crate::error::{DiagramError, DiagramResult};
use crate::ports::{DiagramProvider, DiagramRequest, DiagramResponse, OutputFormat};

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
