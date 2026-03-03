//! Core services: registry, orchestration.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::{DiagramError, DiagramResult};
use crate::ports::{DiagramProvider, DiagramRequest, DiagramResponse};

/// Central registry for diagram provider discovery and lookup.
pub struct DiagramRegistry {
    providers: HashMap<String, Arc<dyn DiagramProvider>>,
}

impl DiagramRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider under the given name.
    pub fn register(&mut self, name: &str, provider: Arc<dyn DiagramProvider>) {
        self.providers.insert(name.to_lowercase(), provider);
    }

    /// Look up a provider by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn DiagramProvider>> {
        self.providers.get(&name.to_lowercase()).cloned()
    }

    /// Return all registered provider names.
    pub fn known_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.providers.keys().cloned().collect();
        types.sort();
        types
    }
}

impl Default for DiagramRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a diagram by resolving the provider from registry and invoking it.
pub async fn render_with_registry(
    registry: &DiagramRegistry,
    request: &DiagramRequest,
) -> DiagramResult<DiagramResponse> {
    let provider = registry
        .get(&request.diagram_type)
        .ok_or_else(|| DiagramError::ToolNotFound(request.diagram_type.clone()))?;

    provider.validate(&request.source)?;
    provider.generate(request).await
}
