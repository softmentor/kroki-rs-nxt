//! kroki-core: Core domain logic for the kroki diagram platform.
//!
//! This crate contains pure business logic with zero infrastructure dependencies:
//! - `DiagramProvider` trait (the central port)
//! - `DiagramRegistry` for provider discovery and lookup
//! - Domain types: `DiagramRequest`, `DiagramResponse`, `DiagramError`
//! - Provider implementations: Command, Browser, Pipeline
//! - Configuration model
//! - Utility functions (decode, image conversion, font management)

pub mod browser;
pub mod capabilities;
pub mod config;
pub mod error;
pub mod ports;
pub mod providers;
pub mod services;
pub mod utils;

pub use browser::BrowserManager;
pub use capabilities::{CapabilityRegistry, ProviderCategory, ProviderMetadata, RuntimeDependency};
pub use error::{DiagramError, DiagramResult};
pub use ports::{DiagramOptions, DiagramProvider, DiagramRequest, DiagramResponse, OutputFormat};
pub use providers::{
    BpmnProvider, D2Provider, DitaaProvider, EchoProvider, ExcalidrawProvider, GraphvizProvider,
    MermaidProvider, WavedromProvider,
};
pub use services::{render_with_registry, DiagramRegistry};

#[cfg(test)]
mod tests {
    use super::DiagramRegistry;

    #[test]
    fn unit_registry_starts_empty() {
        let registry = DiagramRegistry::new();
        assert!(registry.known_types().is_empty());
    }
}
