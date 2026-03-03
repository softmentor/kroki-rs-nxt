//! Public server surface for kroki-server.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router};
use kroki_adapter_transport::{render_diagram, RenderRequestDto, RenderResponseDto};
use kroki_core::{DiagramRegistry, EchoProvider};

#[derive(Clone)]
struct AppState {
    registry: Arc<DiagramRegistry>,
}

pub fn default_bind_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8000))
}

pub fn app() -> Router {
    let mut registry = DiagramRegistry::new();
    registry.register("echo", Arc::new(EchoProvider::new()));

    let state = AppState {
        registry: Arc::new(registry),
    };

    Router::new()
        .route(
            "/",
            axum::routing::get(|| async { "kroki-rs-nxt server — bootstrap baseline ready" }),
        )
        .route("/render", axum::routing::post(render_handler))
        .with_state(state)
}

async fn render_handler(
    State(state): State<AppState>,
    Json(request): Json<RenderRequestDto>,
) -> Result<Json<RenderResponseDto>, (StatusCode, String)> {
    let response = render_diagram(state.registry.as_ref(), request)
        .await
        .map_err(|err| (StatusCode::BAD_REQUEST, err.to_string()))?;

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[test]
    fn unit_default_bind_addr_is_localhost_8000() {
        let addr = super::default_bind_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:8000");
    }

    #[tokio::test]
    async fn integration_render_route_executes_vertical_slice() {
        let app = super::app();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"A -> B","diagram_type":"echo","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
