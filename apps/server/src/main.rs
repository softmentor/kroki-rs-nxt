//! kroki-server: HTTP API server for the kroki diagram platform.
//!
//! Provides:
//! - REST API for diagram generation (/render, /render/batch)
//! - Admin dashboard with health checks and metrics
//! - Authentication, rate limiting, and circuit breaker middleware

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let addr = kroki_server::default_bind_addr();
    tracing::info!("kroki-server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, kroki_server::app()).await?;

    Ok(())
}
