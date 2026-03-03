use std::fs;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn resource_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources")
        .join(name)
}

#[test]
fn default_bind_address_is_localhost_8000() {
    let addr = kroki_server::default_bind_addr();
    assert_eq!(addr.to_string(), "127.0.0.1:8000");
}

#[test]
fn test_fixture_config_is_present() {
    let cfg = fs::read_to_string(fixture_path("server-test-config.toml"))
        .expect("server fixture config should exist");
    assert!(cfg.contains("[server]"));
    assert!(cfg.contains("port = 8000"));
}

#[test]
fn test_resource_expected_root_response_is_present() {
    let expected = fs::read_to_string(resource_path("expected-root-response.txt"))
        .expect("server expected-response resource should exist");
    assert!(expected.contains("kroki-rs-nxt server"));
}
