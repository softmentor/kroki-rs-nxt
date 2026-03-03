//! kroki-adapter-transport: Transport layer implementations.
//!
//! Provides:
//! - HTTP handlers and middleware (Axum) for the server surface
//! - DTOs and request/response mapping
//! - Authentication, rate limiting, circuit breaker middleware
//! - Metrics and Prometheus export
//! - Future: IPC handlers for Tauri, CLI dispatch
//!
//! This crate now contains the Phase 2 bootstrap vertical slice mapping:
//! request DTO -> core request -> provider invocation -> transport response DTO.

use kroki_core::{
    render_with_registry, DiagramOptions, DiagramRegistry, DiagramRequest, DiagramResponse,
    DiagramResult, OutputFormat,
};

/// Transport-facing render request DTO.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RenderRequestDto {
    pub source: String,
    pub diagram_type: String,
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
}

/// Transport-facing render response DTO.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RenderResponseDto {
    pub data: String,
    pub content_type: String,
    pub duration_ms: u64,
}

fn default_output_format() -> OutputFormat {
    OutputFormat::Svg
}

impl From<RenderRequestDto> for DiagramRequest {
    fn from(value: RenderRequestDto) -> Self {
        Self {
            source: value.source,
            diagram_type: value.diagram_type,
            output_format: value.output_format,
            options: DiagramOptions::default(),
        }
    }
}

impl From<DiagramResponse> for RenderResponseDto {
    fn from(value: DiagramResponse) -> Self {
        Self {
            data: String::from_utf8_lossy(&value.data).to_string(),
            content_type: value.content_type,
            duration_ms: value.duration_ms,
        }
    }
}

/// Execute render orchestration through the core registry and return a transport DTO.
pub async fn render_diagram(
    registry: &DiagramRegistry,
    request: RenderRequestDto,
) -> DiagramResult<RenderResponseDto> {
    let response = render_with_registry(registry, &request.into()).await?;
    Ok(response.into())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{render_diagram, RenderRequestDto};
    use kroki_core::{DiagramRegistry, EchoProvider, OutputFormat};

    #[tokio::test]
    async fn unit_transport_adapter_executes_render_flow() {
        let mut registry = DiagramRegistry::new();
        registry.register("echo", Arc::new(EchoProvider::new()));

        let request = RenderRequestDto {
            source: "A -> B".to_string(),
            diagram_type: "echo".to_string(),
            output_format: OutputFormat::Svg,
        };

        let response = render_diagram(&registry, request)
            .await
            .expect("render flow should succeed");

        assert_eq!(response.content_type, "image/svg+xml");
        assert!(response.data.contains("bootstrap-echo:echo:A -> B"));
    }

    #[test]
    fn unit_transport_adapter_crate_name_is_stable() {
        let crate_name = env!("CARGO_PKG_NAME");
        assert_eq!(crate_name, "kroki-adapter-transport");
    }
}
