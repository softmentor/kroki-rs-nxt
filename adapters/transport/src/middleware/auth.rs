use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use kroki_core::config::AuthConfig;

pub async fn auth_middleware(
    axum::extract::State(config): axum::extract::State<AuthConfig>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !config.enabled {
        return next.run(request).await;
    }

    let header_name = &config.header_name;
    let key = request
        .headers()
        .get(header_name)
        .and_then(|v| v.to_str().ok());

    match key {
        Some(token) if config.api_keys.iter().any(|entry| entry.key == token) => {
            next.run(request).await
        }
        Some(_) => (
            StatusCode::UNAUTHORIZED,
            serde_json::json!({
                "error": "unauthorized",
                "message": "Invalid API key"
            })
            .to_string(),
        )
            .into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            serde_json::json!({
                "error": "unauthorized",
                "message": format!("Missing API key header '{}'", header_name)
            })
            .to_string(),
        )
            .into_response(),
    }
}
