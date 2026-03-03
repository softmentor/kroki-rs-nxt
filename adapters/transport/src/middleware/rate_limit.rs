use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use kroki_core::config::RateLimitConfig;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<IpAddr, TokenBucket>>,
    config: RateLimitConfig,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(burst_size: u32, refill_rate: u32) -> Self {
        Self {
            tokens: burst_size as f64,
            last_refill: Instant::now(),
            max_tokens: burst_size as f64,
            refill_rate: refill_rate.max(1) as f64,
        }
    }

    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn retry_after(&self) -> u64 {
        if self.refill_rate <= 0.0 {
            return 1;
        }
        ((1.0 - self.tokens).max(0.0) / self.refill_rate).ceil() as u64
    }
}

impl RateLimiter {
    pub fn new(config: &RateLimitConfig) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            config: config.clone(),
        }
    }

    pub fn check(&self, ip: IpAddr) -> Result<(), u64> {
        let mut entry = self.buckets.entry(ip).or_insert_with(|| {
            TokenBucket::new(self.config.burst_size, self.config.requests_per_second)
        });

        if entry.try_consume() {
            Ok(())
        } else {
            Err(entry.retry_after())
        }
    }
}

pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<Option<RateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let Some(limiter) = limiter else {
        return next.run(request).await;
    };

    let ip = client_ip_from_headers(&request).unwrap_or_else(|| {
        "127.0.0.1"
            .parse::<IpAddr>()
            .expect("loopback parse should always succeed")
    });

    match limiter.check(ip) {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                serde_json::json!({
                    "error": "rate_limit_exceeded",
                    "message": "Too many requests",
                    "retry_after_seconds": retry_after
                })
                .to_string(),
            )
                .into_response();
            response.headers_mut().insert(
                "retry-after",
                HeaderValue::from_str(&retry_after.to_string())
                    .unwrap_or_else(|_| HeaderValue::from_static("1")),
            );
            response
        }
    }
}

fn client_ip_from_headers(request: &Request<Body>) -> Option<IpAddr> {
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(text) = forwarded.to_str() {
            if let Some(first) = text.split(',').next() {
                if let Ok(ip) = first.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    request
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .and_then(|text| text.parse::<IpAddr>().ok())
}
