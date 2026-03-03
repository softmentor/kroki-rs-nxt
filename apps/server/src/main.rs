//! kroki-server: HTTP API server for the kroki diagram platform.
//!
//! Provides:
//! - REST API for diagram generation (/render, /render/batch)
//! - Admin dashboard with health checks and metrics
//! - Authentication, rate limiting, and circuit breaker middleware

use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("kroki-server starting on {}", addr);

    // Placeholder: server setup will be implemented in Phase 3
    let app = axum::Router::new().route(
        "/",
        axum::routing::get(|| async { "kroki-rs-nxt server — not yet implemented" }),
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
