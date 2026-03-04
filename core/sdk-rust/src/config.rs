//! Configuration model for the kroki platform.
//!
//! Runtime configuration loaded from `kroki.toml` with environment variable overrides.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub browser: BrowserConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_admin_port")]
    pub admin_port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default = "default_max_input_size")]
    pub max_input_size: usize,
    #[serde(default = "default_max_output_size")]
    pub max_output_size: usize,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            admin_port: default_admin_port(),
            host: default_host(),
            log_level: default_log_level(),
            timeout_ms: default_timeout_ms(),
            max_input_size: default_max_input_size(),
            max_output_size: default_max_output_size(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub api_keys: Vec<ApiKeyEntry>,
    #[serde(default = "default_auth_header")]
    pub header_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyEntry {
    pub key: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub rate_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_requests_per_second")]
    pub requests_per_second: u32,
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: default_requests_per_second(),
            burst_size: default_burst_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    #[serde(default = "default_reset_timeout_secs")]
    pub reset_timeout_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            failure_threshold: default_failure_threshold(),
            reset_timeout_secs: default_reset_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,
    #[serde(default = "default_metrics_export_endpoint")]
    pub export_endpoint: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            export_endpoint: default_metrics_export_endpoint(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
    #[serde(default = "default_context_ttl_requests")]
    pub context_ttl_requests: usize,
    #[serde(default = "default_engine_urls")]
    pub engine_urls: HashMap<String, String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            pool_size: default_pool_size(),
            context_ttl_requests: default_context_ttl_requests(),
            engine_urls: default_engine_urls(),
        }
    }
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> anyhow::Result<Self> {
        let path = match path {
            Some(explicit) => Some(explicit),
            None => match env::var("KROKI_CONFIG") {
                Ok(from_env) => Some(PathBuf::from(from_env)),
                Err(_) => Self::default_config_path(),
            },
        };

        let mut config = if let Some(path) = path {
            if path.exists() {
                let bytes = std::fs::read_to_string(&path)?;
                toml::from_str::<Config>(&bytes)?
            } else {
                Config::default()
            }
        } else {
            Config::default()
        };

        config.apply_env_overrides();
        Ok(config)
    }

    fn default_config_path() -> Option<PathBuf> {
        let candidate = Path::new("kroki.toml");
        if candidate.exists() {
            Some(candidate.to_path_buf())
        } else {
            None
        }
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(v) = env::var("KROKI_SERVER_HOST") {
            self.server.host = v;
        }
        if let Ok(v) = env::var("KROKI_SERVER_PORT") {
            if let Ok(parsed) = v.parse::<u16>() {
                self.server.port = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_ADMIN_PORT") {
            if let Ok(parsed) = v.parse::<u16>() {
                self.server.admin_port = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_TIMEOUT_MS") {
            if let Ok(parsed) = v.parse::<u64>() {
                self.server.timeout_ms = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_MAX_INPUT_SIZE") {
            if let Ok(parsed) = v.parse::<usize>() {
                self.server.max_input_size = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_MAX_OUTPUT_SIZE") {
            if let Ok(parsed) = v.parse::<usize>() {
                self.server.max_output_size = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_AUTH_ENABLED") {
            if let Ok(parsed) = v.parse::<bool>() {
                self.server.auth.enabled = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_AUTH_HEADER") {
            self.server.auth.header_name = v;
        }
        if let Ok(v) = env::var("KROKI_SERVER_RATE_LIMIT_ENABLED") {
            if let Ok(parsed) = v.parse::<bool>() {
                self.server.rate_limit.enabled = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_RATE_LIMIT_RPS") {
            if let Ok(parsed) = v.parse::<u32>() {
                self.server.rate_limit.requests_per_second = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_RATE_LIMIT_BURST") {
            if let Ok(parsed) = v.parse::<u32>() {
                self.server.rate_limit.burst_size = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_CIRCUIT_BREAKER_ENABLED") {
            if let Ok(parsed) = v.parse::<bool>() {
                self.server.circuit_breaker.enabled = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_CIRCUIT_BREAKER_THRESHOLD") {
            if let Ok(parsed) = v.parse::<u32>() {
                self.server.circuit_breaker.failure_threshold = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_SERVER_CIRCUIT_BREAKER_RESET_SECS") {
            if let Ok(parsed) = v.parse::<u64>() {
                self.server.circuit_breaker.reset_timeout_secs = parsed;
            }
        }
        if let Ok(v) = env::var("KROKI_BROWSER_POOL_SIZE") {
            if let Ok(parsed) = v.parse::<usize>() {
                self.browser.pool_size = parsed.max(1);
            }
        }
        if let Ok(v) = env::var("KROKI_BROWSER_CONTEXT_TTL_REQUESTS") {
            if let Ok(parsed) = v.parse::<usize>() {
                self.browser.context_ttl_requests = parsed.max(1);
            }
        }
    }
}

fn default_port() -> u16 {
    8000
}
fn default_admin_port() -> u16 {
    8081
}
fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_timeout_ms() -> u64 {
    5_000
}
fn default_max_input_size() -> usize {
    1_048_576
}
fn default_max_output_size() -> usize {
    52_428_800
}
fn default_auth_header() -> String {
    "X-API-Key".to_string()
}
fn default_requests_per_second() -> u32 {
    100
}
fn default_burst_size() -> u32 {
    10
}
fn default_failure_threshold() -> u32 {
    5
}
fn default_reset_timeout_secs() -> u64 {
    60
}
fn default_metrics_enabled() -> bool {
    true
}
fn default_metrics_export_endpoint() -> bool {
    true
}
fn default_pool_size() -> usize {
    4
}
fn default_context_ttl_requests() -> usize {
    100
}
fn default_engine_urls() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("mermaid".to_string(), "https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js".to_string());
    map.insert("bpmn".to_string(), "https://unpkg.com/bpmn-js@17/dist/bpmn-viewer.development.js".to_string());
    map
}
