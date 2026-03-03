use kroki_core::DiagramRegistry;
use std::path::PathBuf;

#[test]
fn registry_starts_empty() {
    let registry = DiagramRegistry::new();
    assert!(registry.known_types().is_empty());
}

#[test]
fn fixture_diagram_types_exists_for_future_registry_tests() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("diagram-types.txt");
    assert!(fixture.exists());
}
