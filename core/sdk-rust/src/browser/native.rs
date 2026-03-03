use std::ffi::OsStr;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use headless_chrome::{Browser, LaunchOptions, Tab};
use tempfile::{Builder, NamedTempFile};
use tokio::sync::{RwLock, Semaphore};

use crate::browser::backend::BrowserBackend;
use crate::error::{DiagramError, DiagramResult};

const CHROME_ARGS: &[&str] = &[
    "--no-sandbox",
    "--disable-setuid-sandbox",
    "--disable-dev-shm-usage",
    "--disable-gpu",
    "--disable-web-security",
    "--disable-software-rasterizer",
    "--disable-features=IsolateOrigins,site-per-process",
    "--font-render-hinting=none",
    "--allow-file-access-from-files",
];

const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
fn adaptive_recycle_failure_threshold() -> usize {
    std::env::var("KROKI_BROWSER_ADAPTIVE_RECYCLE_FAILURES")
        .ok()
        .and_then(|raw| raw.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(3)
}

/// Native browser backend based on headless_chrome (CDP).
pub struct NativeBackend {
    browser: Arc<RwLock<Browser>>,
    harness_url: String,
    semaphore: Arc<Semaphore>,
    _harness_file: NamedTempFile,
    context_ttl_requests: usize,
    request_count: AtomicUsize,
    consecutive_failures: AtomicUsize,
    restarting: AtomicBool,
}

impl NativeBackend {
    pub async fn new(pool_size: usize, context_ttl_requests: usize) -> Result<Self, String> {
        let (harness_url, harness_file) = Self::build_harness()?;
        let browser = Self::spawn_browser().await?;
        Ok(Self {
            browser: Arc::new(RwLock::new(browser)),
            harness_url,
            semaphore: Arc::new(Semaphore::new(pool_size.max(1))),
            _harness_file: harness_file,
            context_ttl_requests: context_ttl_requests.max(1),
            request_count: AtomicUsize::new(0),
            consecutive_failures: AtomicUsize::new(0),
            restarting: AtomicBool::new(false),
        })
    }

    fn build_harness() -> Result<(String, NamedTempFile), String> {
        let mut temp_file = Builder::new()
            .suffix(".html")
            .tempfile()
            .map_err(|e| format!("failed to create harness file: {e}"))?;

        let html = r#"<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <style id="kroki-fonts"></style>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
</head>
<body>
  <div id="container"></div>
  <script>
    window.krokiRenderMermaid = async (source) => {
      if (!window.mermaid) throw new Error("Mermaid runtime unavailable");
      window.mermaid.initialize({ startOnLoad: false, securityLevel: "loose" });
      const id = "kroki" + Math.floor(Math.random() * 1e9);
      const { svg } = await window.mermaid.render(id, source);
      return svg;
    };
  </script>
</body>
</html>"#;

        temp_file
            .write_all(html.as_bytes())
            .map_err(|e| format!("failed to write harness: {e}"))?;

        let path = temp_file
            .path()
            .to_str()
            .ok_or_else(|| "failed to convert harness path".to_string())?;
        Ok((format!("file://{path}"), temp_file))
    }

    fn launch_options() -> LaunchOptions<'static> {
        let args: Vec<&'static OsStr> = CHROME_ARGS.iter().map(OsStr::new).collect();
        LaunchOptions {
            args,
            idle_browser_timeout: DEFAULT_IDLE_TIMEOUT,
            ..Default::default()
        }
    }

    async fn spawn_browser() -> Result<Browser, String> {
        let options = Self::launch_options();
        tokio::task::spawn_blocking(move || Browser::new(options))
            .await
            .map_err(|e| format!("browser spawn join failed: {e}"))?
            .map_err(|e| e.to_string())
    }

    async fn restart_browser(&self) -> Result<(), String> {
        let new_browser = Self::spawn_browser().await?;
        let mut guard = self.browser.write().await;
        *guard = new_browser;
        Ok(())
    }

    fn should_ttl_recycle(&self) -> bool {
        let count = self.request_count.fetch_add(1, Ordering::Relaxed) + 1;
        count >= self.context_ttl_requests
    }

    fn should_failure_recycle(&self) -> bool {
        self.consecutive_failures.load(Ordering::Relaxed) >= adaptive_recycle_failure_threshold()
    }

    async fn maybe_recycle(&self) {
        if !(self.should_ttl_recycle() || self.should_failure_recycle()) {
            return;
        }
        if self
            .restarting
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return;
        }

        if let Err(err) = self.restart_browser().await {
            tracing::error!(error = %err, "native browser recycle failed");
        } else {
            tracing::warn!("native browser recycled");
            self.request_count.store(0, Ordering::Relaxed);
            self.consecutive_failures.store(0, Ordering::Relaxed);
        }

        self.restarting.store(false, Ordering::Release);
    }

    async fn acquire_browser(&self) -> Browser {
        let guard = self.browser.read().await;
        guard.clone()
    }

    async fn do_render(
        &self,
        tab: &Tab,
        diagram_type: &str,
        source: &str,
        _format: &str,
        font_css: Option<&str>,
    ) -> DiagramResult<Vec<u8>> {
        tab.navigate_to(&self.harness_url)
            .map_err(|e| DiagramError::ProcessFailed(format!("navigation failed: {e}")))?;
        tab.wait_for_element("#container")
            .map_err(|e| DiagramError::ProcessFailed(format!("harness load timeout: {e}")))?;

        if let Some(css) = font_css {
            let escaped = serde_json::to_string(css).map_err(|e| {
                DiagramError::Internal(format!("font css serialization failed: {e}"))
            })?;
            let inject = format!(
                "const style=document.getElementById('kroki-fonts'); if(style){{style.innerHTML={escaped};}}"
            );
            tab.evaluate(&inject, false)
                .map_err(|e| DiagramError::ProcessFailed(format!("font injection failed: {e}")))?;
        }

        match diagram_type {
            "mermaid" => {
                let render_expr = format!(
                    "window.krokiRenderMermaid({})",
                    serde_json::to_string(source).unwrap_or_else(|_| "\"\"".to_string())
                );
                let rendered = tab.evaluate(&render_expr, true).map_err(|e| {
                    DiagramError::ProcessFailed(format!("render execution failed: {e}"))
                })?;

                let svg = rendered
                    .value
                    .and_then(|v| v.as_str().map(ToString::to_string))
                    .ok_or_else(|| {
                        DiagramError::ProcessFailed(
                            "render returned null or non-string".to_string(),
                        )
                    })?;
                Ok(svg.into_bytes())
            }
            "bpmn" => Err(DiagramError::Internal(
                "bpmn browser runtime not wired yet".to_string(),
            )),
            _ => Err(DiagramError::UnsupportedFormat {
                provider: diagram_type.to_string(),
                format: _format.to_string(),
            }),
        }
    }
}

#[async_trait]
impl BrowserBackend for NativeBackend {
    async fn render(
        &self,
        diagram_type: &str,
        source: &str,
        format: &str,
        font_css: Option<&str>,
    ) -> DiagramResult<Vec<u8>> {
        self.maybe_recycle().await;

        let _permit =
            self.semaphore.acquire().await.map_err(|_| {
                DiagramError::Internal("native browser semaphore closed".to_string())
            })?;

        let browser = self.acquire_browser().await;
        let tab = browser
            .new_tab()
            .map_err(|e| DiagramError::ProcessFailed(format!("failed to create tab: {e}")))?;

        let result = self
            .do_render(tab.as_ref(), diagram_type, source, format, font_css)
            .await;

        let _ = tab.close(false);
        if result.is_ok() {
            self.consecutive_failures.store(0, Ordering::Relaxed);
        } else {
            self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    async fn health(&self) -> serde_json::Value {
        let browser = self.acquire_browser().await;
        let tabs_count = browser
            .get_tabs()
            .lock()
            .map(|tabs| tabs.len())
            .unwrap_or(0);
        serde_json::json!({
            "status": "ok",
            "backend": "headless_chrome",
            "tabs": tabs_count,
            "harness_url": self.harness_url,
            "concurrency_permits_available": self.semaphore.available_permits(),
            "consecutive_failures": self.consecutive_failures.load(Ordering::Relaxed),
            "adaptive_recycle_failure_threshold": adaptive_recycle_failure_threshold(),
            "context_ttl_requests": self.context_ttl_requests,
        })
    }
}
