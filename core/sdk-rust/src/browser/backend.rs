use async_trait::async_trait;

use crate::error::DiagramResult;

/// Abstract interface for browser-backed rendering providers.
#[async_trait]
pub trait BrowserBackend: Send + Sync {
    async fn render(
        &self,
        diagram_type: &str,
        source: &str,
        format: &str,
        font_css: Option<&str>,
    ) -> DiagramResult<Vec<u8>>;

    async fn health(&self) -> serde_json::Value;
}
