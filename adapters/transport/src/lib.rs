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

pub mod middleware;

use kroki_core::{
    render_with_registry, DiagramError, DiagramOptions, DiagramRegistry, DiagramRequest,
    DiagramResponse, DiagramResult, OutputFormat,
};
use std::io::Read;
use tracing::{debug, error, info};

/// Transport-facing render request DTO.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct RenderRequestDto {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub source_encoded: Option<String>,
    #[serde(default)]
    pub source_encoding: PayloadEncoding,
    pub diagram_type: String,
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
}

/// Supported payload encoding modes for transport requests.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PayloadEncoding {
    #[default]
    Plain,
    Base64,
    Base64Deflate,
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

impl RenderRequestDto {
    fn decode_source(&self) -> DiagramResult<String> {
        if !self.source.is_empty() {
            return Ok(self.source.clone());
        }

        let encoded = self.source_encoded.clone().ok_or_else(|| {
            DiagramError::ValidationFailed(
                "either 'source' or 'source_encoded' must be provided".to_string(),
            )
        })?;

        match self.source_encoding {
            PayloadEncoding::Plain => Ok(encoded),
            PayloadEncoding::Base64 => decode_base64_source(&encoded),
            PayloadEncoding::Base64Deflate => decode_base64_deflate_source(&encoded),
        }
    }

    fn into_diagram_request(self) -> DiagramResult<DiagramRequest> {
        Ok(DiagramRequest {
            source: self.decode_source()?,
            diagram_type: self.diagram_type,
            output_format: self.output_format,
            options: DiagramOptions::default(),
        })
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
    let diagram_type = request.diagram_type.clone();
    let request = request.into_diagram_request()?;
    debug!(diagram_type = %diagram_type, "transport received render request");

    let started = std::time::Instant::now();
    let response = match render_with_registry(registry, &request).await {
        Ok(response) => response,
        Err(err) => {
            metrics::counter!(
                "kroki_transport_render_requests_total",
                "diagram_type" => diagram_type.clone(),
                "status" => "error"
            )
            .increment(1);
            error!(
                diagram_type = %diagram_type,
                error = %err,
                "transport render flow failed"
            );
            return Err(err);
        }
    };

    metrics::counter!(
        "kroki_transport_render_requests_total",
        "diagram_type" => diagram_type.clone(),
        "status" => "ok"
    )
    .increment(1);
    metrics::histogram!(
        "kroki_transport_render_duration_ms",
        "diagram_type" => diagram_type.clone()
    )
    .record(started.elapsed().as_millis() as f64);

    info!(
        diagram_type = %diagram_type,
        duration_ms = started.elapsed().as_millis() as u64,
        "transport render flow completed"
    );
    Ok(response.into())
}

fn decode_base64_source(encoded: &str) -> DiagramResult<String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(encoded))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded))
        .map_err(|err| DiagramError::ValidationFailed(format!("invalid base64 payload: {err}")))?;
    String::from_utf8(bytes)
        .map_err(|err| DiagramError::ValidationFailed(format!("invalid utf-8 payload: {err}")))
}

fn decode_base64_deflate_source(encoded: &str) -> DiagramResult<String> {
    use base64::Engine;
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(encoded))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded))
        .map_err(|err| DiagramError::ValidationFailed(format!("invalid base64 payload: {err}")))?;
    let mut decoder = flate2::read::ZlibDecoder::new(compressed.as_slice());
    let mut output = String::new();
    decoder
        .read_to_string(&mut output)
        .map_err(|err| DiagramError::ValidationFailed(format!("invalid zlib payload: {err}")))?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::Arc;

    use super::{render_diagram, PayloadEncoding, RenderRequestDto};
    use kroki_core::{DiagramRegistry, EchoProvider, OutputFormat};

    #[tokio::test]
    async fn unit_transport_adapter_executes_render_flow() {
        let mut registry = DiagramRegistry::new();
        registry.register("echo", Arc::new(EchoProvider::new()));

        let request = RenderRequestDto {
            source: "A -> B".to_string(),
            source_encoded: None,
            source_encoding: PayloadEncoding::Plain,
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

    #[tokio::test]
    async fn unit_transport_adapter_accepts_base64_source() {
        use base64::Engine;

        let mut registry = DiagramRegistry::new();
        registry.register("echo", Arc::new(EchoProvider::new()));
        let encoded = base64::engine::general_purpose::STANDARD.encode("A -> B");
        let request = RenderRequestDto {
            source: String::new(),
            source_encoded: Some(encoded),
            source_encoding: PayloadEncoding::Base64,
            diagram_type: "echo".to_string(),
            output_format: OutputFormat::Svg,
        };

        let response = render_diagram(&registry, request)
            .await
            .expect("render flow should succeed");
        assert!(response.data.contains("bootstrap-echo:echo:A -> B"));
    }

    #[tokio::test]
    async fn unit_transport_adapter_accepts_base64_deflate_source() {
        use base64::Engine;

        let mut registry = DiagramRegistry::new();
        registry.register("echo", Arc::new(EchoProvider::new()));

        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder
            .write_all("A -> B".as_bytes())
            .expect("zlib encoding should succeed");
        let compressed = encoder.finish().expect("zlib finish should succeed");
        let encoded = base64::engine::general_purpose::STANDARD.encode(compressed);

        let request = RenderRequestDto {
            source: String::new(),
            source_encoded: Some(encoded),
            source_encoding: PayloadEncoding::Base64Deflate,
            diagram_type: "echo".to_string(),
            output_format: OutputFormat::Svg,
        };

        let response = render_diagram(&registry, request)
            .await
            .expect("render flow should succeed");
        assert!(response.data.contains("bootstrap-echo:echo:A -> B"));
    }
}
