use std::sync::Arc;

use anyhow::{anyhow, Result};

use crate::browser::backend::BrowserBackend;
use crate::error::DiagramResult;

/// Unified browser manager with pooled execution.
#[derive(Clone)]
pub struct BrowserManager {
    backend: Arc<dyn BrowserBackend>,
}

impl BrowserManager {
    /// Starts the preferred backend.
    /// In `native-browser` builds this is a headless_chrome/CDP backend.
    pub async fn start(pool_size: usize, context_ttl_requests: usize) -> Result<Self> {
        #[cfg(feature = "native-browser")]
        {
            let backend =
                crate::browser::native::NativeBackend::new(pool_size, context_ttl_requests)
                    .await
                    .map_err(|err| anyhow!("native browser backend startup failed: {err}"))?;

            Ok(Self {
                backend: Arc::new(backend),
            })
        }

        #[cfg(not(feature = "native-browser"))]
        {
            let _ = (pool_size, context_ttl_requests);
            Err(anyhow!(
                "browser rendering is disabled in this build. Rebuild with --features native-browser"
            ))
        }
    }

    pub async fn evaluate(
        &self,
        diagram_type: &str,
        source: &str,
        format: &str,
        font_css: Option<&str>,
    ) -> DiagramResult<Vec<u8>> {
        self.backend
            .render(diagram_type, source, format, font_css)
            .await
    }

    pub async fn get_pool_health(&self) -> Result<serde_json::Value> {
        Ok(self.backend.health().await)
    }
}
