//! Capability registry and provider metadata contracts.

use std::collections::HashMap;

use crate::ports::OutputFormat;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProviderCategory {
    Command,
    Browser,
    Pipeline,
    Plugin,
    Stub,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RuntimeDependency {
    None,
    SystemTool { binary: String },
    BrowserEngine,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProviderMetadata {
    pub provider_id: String,
    pub category: ProviderCategory,
    pub runtime: RuntimeDependency,
    pub supported_formats: Vec<OutputFormat>,
    pub description: String,
}

impl ProviderMetadata {
    pub fn bootstrap(name: &str, supported_formats: Vec<OutputFormat>) -> Self {
        Self {
            provider_id: name.to_lowercase(),
            category: ProviderCategory::Stub,
            runtime: RuntimeDependency::None,
            supported_formats,
            description: "Bootstrap provider metadata".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    capabilities: HashMap<String, ProviderMetadata>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    pub fn register(&mut self, metadata: ProviderMetadata) {
        self.capabilities
            .insert(metadata.provider_id.to_lowercase(), metadata);
    }

    pub fn get(&self, provider_id: &str) -> Option<&ProviderMetadata> {
        self.capabilities.get(&provider_id.to_lowercase())
    }

    pub fn all(&self) -> Vec<&ProviderMetadata> {
        let mut values: Vec<&ProviderMetadata> = self.capabilities.values().collect();
        values.sort_by(|a, b| a.provider_id.cmp(&b.provider_id));
        values
    }
}
