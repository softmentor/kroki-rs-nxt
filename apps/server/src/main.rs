//! kroki-server: HTTP API server for the kroki diagram platform.
//!
//! Provides:
//! - REST API for diagram generation (/render, /render/batch)
//! - Admin dashboard with health checks and metrics
//! - Authentication, rate limiting, and circuit breaker middleware

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use kroki_core::config::Config;
use tracing_subscriber::fmt::time::OffsetTime;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ServerMode {
    Dev,
    Prod,
}

#[derive(Debug, Parser)]
#[command(
    name = "kroki-server",
    version,
    about = "kroki HTTP server",
    long_about = "kroki-server starts the HTTP API and playground UI.\n\n\
Modes:\n\
  dev  - localhost binding for local development\n\
  prod - all-interface binding for container/production runtime"
)]
struct ServerCli {
    /// Optional path to runtime configuration file (kroki.toml)
    #[arg(long)]
    config: Option<PathBuf>,
    /// Enable debug logging regardless of RUST_LOG
    #[arg(long, default_value_t = false)]
    debug: bool,
    /// Runtime mode: dev or prod
    #[arg(long, value_enum)]
    mode: Option<ServerMode>,
    /// Bind IP address override
    #[arg(long)]
    host: Option<IpAddr>,
    /// Listen port override
    #[arg(long)]
    port: Option<u16>,
    /// Full bind address override (takes precedence over host/port)
    #[arg(long)]
    bind: Option<SocketAddr>,
    /// Admin host override
    #[arg(long)]
    admin_host: Option<IpAddr>,
    /// Admin port override
    #[arg(long)]
    admin_port: Option<u16>,
    /// Full admin bind address override
    #[arg(long)]
    admin_bind: Option<SocketAddr>,
}

fn read_mode(cli_mode: Option<ServerMode>) -> ServerMode {
    if let Some(mode) = cli_mode {
        return mode;
    }
    match std::env::var("KROKI_SERVER_MODE")
        .unwrap_or_else(|_| "dev".to_string())
        .to_lowercase()
        .as_str()
    {
        "prod" | "production" => ServerMode::Prod,
        _ => ServerMode::Dev,
    }
}

fn read_bind_addr(
    cli: &ServerCli,
    mode: ServerMode,
    config: &Config,
) -> anyhow::Result<SocketAddr> {
    if let Some(bind) = cli.bind {
        return Ok(bind);
    }

    if let Ok(bind) = std::env::var("KROKI_SERVER_BIND") {
        return bind
            .parse::<SocketAddr>()
            .map_err(|err| anyhow::anyhow!("invalid KROKI_SERVER_BIND '{bind}': {err}"));
    }

    let host = cli
        .host
        .or_else(|| {
            std::env::var("KROKI_SERVER_HOST")
                .ok()
                .and_then(|v| v.parse::<IpAddr>().ok())
        })
        .unwrap_or_else(|| {
            config
                .server
                .host
                .parse::<IpAddr>()
                .unwrap_or_else(|_| match mode {
                    ServerMode::Dev => [127, 0, 0, 1].into(),
                    ServerMode::Prod => [0, 0, 0, 0].into(),
                })
        });

    let port = cli
        .port
        .or_else(|| {
            std::env::var("KROKI_SERVER_PORT")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
        })
        .unwrap_or(config.server.port);

    Ok(SocketAddr::from((host, port)))
}

fn read_admin_bind_addr(
    cli: &ServerCli,
    mode: ServerMode,
    config: &Config,
) -> anyhow::Result<SocketAddr> {
    if let Some(bind) = cli.admin_bind {
        return Ok(bind);
    }

    if let Ok(bind) = std::env::var("KROKI_SERVER_ADMIN_BIND") {
        return bind
            .parse::<SocketAddr>()
            .map_err(|err| anyhow::anyhow!("invalid KROKI_SERVER_ADMIN_BIND '{bind}': {err}"));
    }

    let host = cli
        .admin_host
        .or_else(|| {
            std::env::var("KROKI_SERVER_ADMIN_HOST")
                .ok()
                .and_then(|v| v.parse::<IpAddr>().ok())
        })
        .unwrap_or_else(|| {
            config
                .server
                .host
                .parse::<IpAddr>()
                .unwrap_or_else(|_| match mode {
                    ServerMode::Dev => [127, 0, 0, 1].into(),
                    ServerMode::Prod => [0, 0, 0, 0].into(),
                })
        });

    let port = cli
        .admin_port
        .or_else(|| {
            std::env::var("KROKI_SERVER_ADMIN_PORT")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
        })
        .unwrap_or(config.server.admin_port);

    Ok(SocketAddr::from((host, port)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = ServerCli::parse();
    let config = Config::load(cli.config.clone())?;
    let env_filter = if cli.debug {
        tracing_subscriber::EnvFilter::new("debug")
    } else {
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(config.server.log_level.clone()))
    };
    let timer_format =
        time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let timer = OffsetTime::new(offset, timer_format);
    tracing_subscriber::fmt()
        .with_timer(timer)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(env_filter)
        .init();

    let mode = read_mode(cli.mode);
    let addr = read_bind_addr(&cli, mode, &config)?;
    let admin_addr = read_admin_bind_addr(&cli, mode, &config)?;
    let metrics_handle = if config.server.metrics.enabled {
        Some(
            metrics_exporter_prometheus::PrometheusBuilder::new()
                .install_recorder()
                .map_err(|err| anyhow::anyhow!("failed to install prometheus recorder: {err}"))?,
        )
    } else {
        None
    };
    metrics::counter!("kroki_server_starts_total", "mode" => format!("{mode:?}")).increment(1);

    tracing::info!(
        surface = "server",
        version = env!("CARGO_PKG_VERSION"),
        mode = ?mode,
        "starting surface"
    );

    tracing::info!(listen_ports = %format!("[{}, {}]", addr.port(), admin_addr.port()), "server listen ports");
    tracing::info!("kroki-server starting on {}", addr);
    tracing::info!(
        admin_health = %format!("http://{}/health", admin_addr),
        admin_metrics = %format!("http://{}/metrics", admin_addr),
        "admin endpoints configured"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let admin_listener = tokio::net::TcpListener::bind(admin_addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!("Admin listening on {}", admin_addr);
    let public = axum::serve(listener, kroki_server::app_with_config(&config));
    let admin = axum::serve(admin_listener, kroki_server::admin_app(metrics_handle));
    tokio::try_join!(public, admin)?;

    Ok(())
}
