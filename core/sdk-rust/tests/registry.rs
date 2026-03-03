use std::sync::Arc;

use kroki_core::{
    DiagramRegistry, EchoProvider, OutputFormat, ProviderCategory, ProviderMetadata,
    RuntimeDependency,
};
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

#[test]
fn register_adds_bootstrap_metadata_by_default() {
    let mut registry = DiagramRegistry::new();
    registry.register("echo", Arc::new(EchoProvider::new()));

    let metadata = registry
        .metadata("echo")
        .expect("metadata should exist for registered provider");
    assert_eq!(metadata.provider_id, "echo");
    assert_eq!(metadata.category, ProviderCategory::Stub);
    assert_eq!(metadata.runtime, RuntimeDependency::None);
    assert_eq!(metadata.supported_formats, vec![OutputFormat::Svg]);
}

#[test]
fn register_with_metadata_preserves_explicit_contract() {
    let mut registry = DiagramRegistry::new();
    let metadata = ProviderMetadata {
        provider_id: "graphviz".to_string(),
        category: ProviderCategory::Command,
        runtime: RuntimeDependency::SystemTool {
            binary: "dot".to_string(),
        },
        supported_formats: vec![OutputFormat::Svg],
        description: "Graphviz command provider".to_string(),
    };
    registry.register_with_metadata("graphviz", Arc::new(EchoProvider::new()), metadata);

    let graphviz = registry
        .metadata("graphviz")
        .expect("graphviz metadata should be available");
    assert_eq!(graphviz.category, ProviderCategory::Command);
    assert_eq!(
        graphviz.runtime,
        RuntimeDependency::SystemTool {
            binary: "dot".to_string()
        }
    );
}
