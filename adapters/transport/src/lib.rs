//! kroki-adapter-transport: Transport layer implementations.
//!
//! Provides:
//! - HTTP handlers and middleware (Axum) for the server surface
//! - DTOs and request/response mapping
//! - Authentication, rate limiting, circuit breaker middleware
//! - Metrics and Prometheus export
//! - Future: IPC handlers for Tauri, CLI dispatch
//!
//! Bootstrap baseline transport adapter crate; feature-complete implementation is planned for Phase 3 (Batch 4).

#[cfg(test)]
mod tests {
    #[test]
    fn unit_transport_adapter_crate_name_is_stable() {
        let crate_name = env!("CARGO_PKG_NAME");
        assert_eq!(crate_name, "kroki-adapter-transport");
    }
}
