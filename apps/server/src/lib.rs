//! Public server surface for kroki-server.

use std::net::SocketAddr;

pub fn default_bind_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8000))
}

pub fn app() -> axum::Router {
    // Bootstrap baseline server setup; feature-complete routing is planned for Phase 3.
    axum::Router::new().route(
        "/",
        axum::routing::get(|| async { "kroki-rs-nxt server — bootstrap baseline ready" }),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_default_bind_addr_is_localhost_8000() {
        let addr = super::default_bind_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:8000");
    }
}
