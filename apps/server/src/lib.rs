//! Public server surface for kroki-server.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use axum::{Json, Router};
use kroki_adapter_transport::middleware::auth::auth_middleware;
use kroki_adapter_transport::middleware::circuit_breaker::CircuitBreakerManager;
use kroki_adapter_transport::middleware::rate_limit::{rate_limit_middleware, RateLimiter};
use kroki_adapter_transport::{render_diagram, RenderRequestDto, RenderResponseDto};
use kroki_core::config::Config;
use kroki_core::{
    BpmnProvider, D2Provider, DiagramError, DiagramRegistry, EchoProvider, GraphvizProvider,
    MermaidProvider, ProviderCategory, ProviderMetadata, RuntimeDependency,
};
use metrics_exporter_prometheus::PrometheusHandle;
use tracing::{debug, error, info, warn};

#[derive(Clone)]
struct AppState {
    registry: Arc<DiagramRegistry>,
    policies: Arc<RuntimePolicies>,
}

#[derive(Clone)]
struct AdminState {
    metrics: Option<PrometheusHandle>,
}

#[derive(Clone)]
struct RuntimePolicies {
    max_input_size: usize,
    max_output_size: usize,
    circuit_breaker: Option<CircuitBreakerManager>,
}

pub fn default_bind_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8000))
}

pub fn default_admin_bind_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8081))
}

pub fn app() -> Router {
    app_with_config(&Config::default())
}

pub fn app_with_config(config: &Config) -> Router {
    let mut registry = DiagramRegistry::new();
    registry.register("echo", Arc::new(EchoProvider::new()));
    registry.register_with_metadata(
        "graphviz",
        Arc::new(GraphvizProvider::new()),
        ProviderMetadata {
            provider_id: "graphviz".to_string(),
            category: ProviderCategory::Command,
            runtime: RuntimeDependency::SystemTool {
                binary: "dot".to_string(),
            },
            supported_formats: vec![kroki_core::OutputFormat::Svg],
            description: "Graphviz command provider".to_string(),
        },
    );
    registry.register_with_metadata(
        "d2",
        Arc::new(D2Provider::new()),
        ProviderMetadata {
            provider_id: "d2".to_string(),
            category: ProviderCategory::Command,
            runtime: RuntimeDependency::SystemTool {
                binary: "d2".to_string(),
            },
            supported_formats: vec![kroki_core::OutputFormat::Svg],
            description: "D2 command provider".to_string(),
        },
    );
    registry.register_with_metadata(
        "mermaid",
        Arc::new(MermaidProvider::new()),
        ProviderMetadata {
            provider_id: "mermaid".to_string(),
            category: ProviderCategory::Browser,
            runtime: RuntimeDependency::BrowserEngine,
            supported_formats: vec![kroki_core::OutputFormat::Svg],
            description: "Mermaid browser provider (feature-gated native-browser path)".to_string(),
        },
    );
    registry.register_with_metadata(
        "bpmn",
        Arc::new(BpmnProvider::new()),
        ProviderMetadata {
            provider_id: "bpmn".to_string(),
            category: ProviderCategory::Browser,
            runtime: RuntimeDependency::BrowserEngine,
            supported_formats: vec![kroki_core::OutputFormat::Svg],
            description: "BPMN browser provider (native-browser runtime pending)".to_string(),
        },
    );

    let rate_limiter = if config.server.rate_limit.enabled {
        Some(RateLimiter::new(&config.server.rate_limit))
    } else {
        None
    };
    let circuit_breaker = if config.server.circuit_breaker.enabled {
        Some(CircuitBreakerManager::new(&config.server.circuit_breaker))
    } else {
        None
    };

    let policies = RuntimePolicies {
        max_input_size: config.server.max_input_size,
        max_output_size: config.server.max_output_size,
        circuit_breaker,
    };

    let state = AppState {
        registry: Arc::new(registry),
        policies: Arc::new(policies.clone()),
    };

    Router::new()
        .route(
            "/",
            axum::routing::get(|| async { "kroki-rs-nxt server - bootstrap baseline ready" }),
        )
        .route("/playground", axum::routing::get(playground_handler))
        .route("/capabilities", axum::routing::get(capabilities_handler))
        .route("/render", axum::routing::post(render_handler))
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter,
            rate_limit_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            config.server.auth.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

pub fn admin_app(metrics: Option<PrometheusHandle>) -> Router {
    let state = AdminState { metrics };
    Router::new()
        .route("/health", axum::routing::get(health_handler))
        .route("/metrics", axum::routing::get(metrics_handler))
        .with_state(state)
}

async fn playground_handler() -> Html<String> {
    Html(PLAYGROUND_HTML_TEMPLATE.replace("__KROKI_SHARED_THEME__", SHARED_THEME_CSS))
}

async fn render_handler(
    State(state): State<AppState>,
    Json(request): Json<RenderRequestDto>,
) -> Result<Json<RenderResponseDto>, (StatusCode, String)> {
    if request.source.len() > state.policies.max_input_size {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            serde_json::json!({
                "code": "payload_too_large",
                "message": format!(
                    "input exceeds max_input_size ({} bytes)",
                    state.policies.max_input_size
                )
            })
            .to_string(),
        ));
    }

    if let Some(cb) = state.policies.circuit_breaker.as_ref() {
        if !cb.should_allow(&request.diagram_type) {
            return Err((
                StatusCode::SERVICE_UNAVAILABLE,
                serde_json::json!({
                    "code": "circuit_breaker_open",
                    "message": format!(
                        "provider '{}' is temporarily unavailable due to repeated failures",
                        request.diagram_type
                    )
                })
                .to_string(),
            ));
        }
    }

    debug!(diagram_type = %request.diagram_type, "server received render request");
    let diagram_type = request.diagram_type.clone();
    let response = render_diagram(state.registry.as_ref(), request)
        .await
        .map_err(|err| {
            if let Some(cb) = state.policies.circuit_breaker.as_ref() {
                if should_record_provider_failure(&err) {
                    cb.record_failure(&diagram_type);
                }
            }
            warn!(error = %err, "render request rejected");
            let (status, code) = map_error_status(&err);
            (
                status,
                serde_json::json!({
                    "code": code,
                    "message": err.to_string()
                })
                .to_string(),
            )
        })?;

    if response.data.len() > state.policies.max_output_size {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({
                "code": "output_too_large",
                "message": format!(
                    "generated output exceeds max_output_size ({} bytes)",
                    state.policies.max_output_size
                )
            })
            .to_string(),
        ));
    }

    if let Some(cb) = state.policies.circuit_breaker.as_ref() {
        cb.record_success(&diagram_type);
    }

    info!(content_type = %response.content_type, "render request completed");
    Ok(Json(response))
}

fn should_record_provider_failure(err: &DiagramError) -> bool {
    !matches!(
        err,
        DiagramError::ValidationFailed(_) | DiagramError::UnsupportedFormat { .. }
    )
}

fn map_error_status(err: &DiagramError) -> (StatusCode, &'static str) {
    match err {
        DiagramError::ValidationFailed(_) => (StatusCode::BAD_REQUEST, "validation_failed"),
        DiagramError::UnsupportedFormat { .. } => {
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, "unsupported_format")
        }
        DiagramError::ToolNotFound(_) => (StatusCode::SERVICE_UNAVAILABLE, "tool_unavailable"),
        DiagramError::ExecutionTimeout { .. } => (StatusCode::GATEWAY_TIMEOUT, "execution_timeout"),
        DiagramError::ProcessFailed(_) => (StatusCode::UNPROCESSABLE_ENTITY, "process_failed"),
        DiagramError::Io(_) | DiagramError::Internal(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
        }
    }
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "kroki-server-admin",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn metrics_handler(State(state): State<AdminState>) -> Result<String, (StatusCode, String)> {
    let handle = state.metrics.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "metrics recorder is not initialized".to_string(),
        )
    })?;
    Ok(handle.render())
}

const SHARED_THEME_CSS: &str = include_str!("../../../shared/design-system/src/theme.css");

const PLAYGROUND_HTML_TEMPLATE: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <title>kroki Playground</title>
    <style>
      __KROKI_SHARED_THEME__
      * {
        box-sizing: border-box;
      }
      body {
        margin: 0;
        min-height: 100vh;
        color: var(--text-main);
        font-family: "Space Grotesk", "Segoe UI", sans-serif;
        background:
          radial-gradient(1200px 640px at 10% -20%, rgba(255, 179, 15, 0.16), transparent),
          radial-gradient(800px 540px at 90% -10%, rgba(255, 111, 97, 0.16), transparent),
          var(--bg-deep);
      }
      .shell {
        max-width: 1440px;
        margin: 18px auto;
        padding: 14px;
      }
      .topbar {
        position: sticky;
        top: 12px;
        z-index: 40;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 12px 14px;
        backdrop-filter: var(--glass-surface);
        background: rgba(17, 14, 39, 0.78);
        border: 1px solid rgba(255, 179, 15, 0.22);
        border-radius: 999px;
        margin-bottom: 12px;
      }
      .brand {
        display: flex;
        align-items: center;
        gap: 10px;
      }
      .brand-dot {
        width: 12px;
        height: 12px;
        border-radius: 999px;
        background: var(--accent-primary);
        box-shadow: var(--glow);
      }
      .brand h1 {
        margin: 0;
        font-size: 1rem;
        letter-spacing: 0.02em;
      }
      .brand p {
        margin: 0;
        color: var(--text-dim);
        font-size: 0.82rem;
      }
      .topbar-actions {
        display: flex;
        align-items: center;
        gap: 10px;
      }
      .layout {
        display: grid;
        gap: 12px;
        grid-template-columns: 280px minmax(420px, 1fr) minmax(340px, 0.85fr);
        min-height: calc(100vh - 110px);
      }
      .panel {
        border: 1px solid transparent;
        border-image: var(--border-glow) 1;
        border-radius: 18px;
        padding: 14px;
        background: var(--bg-card);
        backdrop-filter: var(--glass-surface);
      }
      .panel-title {
        margin: 0 0 10px;
        font-size: 0.95rem;
        color: var(--accent-secondary);
        letter-spacing: 0.02em;
      }
      .example-list {
        display: grid;
        gap: 8px;
      }
      .example-item {
        width: 100%;
        text-align: left;
        border: 1px solid rgba(255, 179, 15, 0.26);
        border-radius: 12px;
        background: rgba(10, 10, 25, 0.42);
        color: var(--text-main);
        padding: 10px;
        cursor: pointer;
        transition: transform 140ms ease, filter 140ms ease, border-color 140ms ease;
      }
      .example-item:hover {
        transform: translateY(-2px);
        filter: brightness(1.08);
      }
      .example-item.active {
        border-color: var(--accent-primary);
        box-shadow: 0 0 0 1px rgba(255, 111, 97, 0.24);
      }
      .example-item strong {
        display: block;
        margin-bottom: 3px;
      }
      .example-item small {
        color: var(--text-dim);
      }
      .controls {
        display: grid;
        grid-template-columns: 1.2fr 1fr 1fr auto;
        gap: 10px;
        margin-bottom: 10px;
      }
      label {
        display: grid;
        gap: 4px;
        font-size: 0.78rem;
        color: var(--text-dim);
      }
      select,
      textarea {
        width: 100%;
        border: 1px solid rgba(255, 179, 15, 0.28);
        border-radius: 12px;
        background: rgba(11, 11, 27, 0.72);
        color: var(--text-main);
        padding: 10px 12px;
      }
      textarea {
        min-height: calc(100vh - 285px);
        resize: vertical;
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-size: 0.9rem;
        line-height: 1.45;
      }
      .toolbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;
        margin-top: 10px;
      }
      .actions {
        display: flex;
        gap: 8px;
      }
      button {
        border: 0;
        border-radius: 999px;
        padding: 10px 14px;
        cursor: pointer;
        color: #170f10;
        font-weight: 700;
        transition: transform 140ms ease, filter 140ms ease;
      }
      button:hover {
        transform: translateY(-2px);
        filter: brightness(1.05);
      }
      .btn-primary {
        background: var(--accent-primary);
        box-shadow: var(--glow);
      }
      .btn-ghost {
        background: transparent;
        color: var(--accent-secondary);
        border: 1px solid var(--accent-secondary);
      }
      .btn-theme {
        background: transparent;
        color: var(--text-main);
        border: 1px solid rgba(255, 179, 15, 0.35);
      }
      .pill {
        border-radius: 999px;
        border: 1px solid rgba(255, 179, 15, 0.28);
        padding: 4px 9px;
        font-size: 0.75rem;
        color: var(--text-dim);
      }
      .status {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 10px;
      }
      .dot {
        width: 8px;
        height: 8px;
        border-radius: 999px;
        background: #2dbf6c;
      }
      .dot.warn {
        background: #ffb30f;
      }
      .dot.err {
        background: var(--accent-primary);
      }
      .preview-wrap {
        background: rgba(10, 10, 24, 0.7);
        border: 1px solid rgba(255, 179, 15, 0.2);
        border-radius: 12px;
        min-height: 360px;
        max-height: calc(100vh - 260px);
        overflow: auto;
        padding: 12px;
      }
      .preview-wrap svg {
        max-width: 100%;
        height: auto;
      }
      .meta {
        margin-top: 10px;
        white-space: pre-wrap;
        overflow-wrap: anywhere;
        color: var(--text-dim);
        font-size: 0.82rem;
      }
      .hint {
        margin-top: 8px;
        color: var(--text-dim);
        font-size: 0.82rem;
      }
      .resp {
        white-space: pre-wrap;
        overflow-wrap: anywhere;
        font-size: 0.82rem;
        color: var(--text-dim);
        margin-top: 10px;
      }
      @media (max-width: 1220px) {
        .layout {
          grid-template-columns: 1fr;
          min-height: auto;
        }
        textarea {
          min-height: 280px;
        }
        .preview-wrap {
          max-height: 420px;
        }
      }
      @media (max-width: 760px) {
        .controls {
          grid-template-columns: 1fr 1fr;
        }
      }
    </style>
    <script type="module">
      import { LitElement, css, html } from 'https://unpkg.com/lit@3/index.js?module';

      const EXAMPLES = [
        {
          id: 'graphviz-flow',
          title: 'Graphviz Flow',
          diagramType: 'graphviz',
          source: 'digraph G {\\n  rankdir=LR;\\n  Client -> Gateway -> Renderer;\\n  Renderer -> Cache;\\n  Cache -> Client [style=dashed];\\n}',
          description: 'DOT flow with directional links'
        },
        {
          id: 'd2-service-map',
          title: 'D2 Service Map',
          diagramType: 'd2',
          source: 'api: API\\nqueue: Queue\\nworker: Worker\\nstore: Storage\\napi -> queue -> worker -> store\\nworker -> api: status',
          description: 'D2 dependency graph'
        },
        {
          id: 'mermaid-seq',
          title: 'Mermaid Sequence',
          diagramType: 'mermaid',
          source: 'sequenceDiagram\\n  participant U as User\\n  participant K as kroki-server\\n  participant P as Provider\\n  U->>K: POST /render\\n  K->>P: execute provider\\n  P-->>K: SVG bytes\\n  K-->>U: JSON payload',
          description: 'Mermaid sequence diagram'
        },
        {
          id: 'echo-debug',
          title: 'Echo Debug',
          diagramType: 'echo',
          source: 'A -> B',
          description: 'Bootstrap provider for request debugging'
        },
        {
          id: 'bpmn-min',
          title: 'BPMN Skeleton',
          diagramType: 'bpmn',
          source: '<?xml version=\"1.0\" encoding=\"UTF-8\"?>\\n<definitions id=\"Defs\" xmlns=\"http://www.omg.org/spec/BPMN/20100524/MODEL\"></definitions>',
          description: 'Current bpmn baseline (runtime pending)'
        }
      ];

      const EXAMPLE_MAP = Object.fromEntries(EXAMPLES.map((x) => [x.id, x]));

      class KrokiPlayground extends LitElement {
        static properties = {
          selectedExample: { type: String },
          diagramType: { type: String },
          outputFormat: { type: String },
          source: { type: String },
          encoded: { type: Boolean },
          autoRender: { type: Boolean },
          theme: { type: String },
          statusText: { type: String },
          statusClass: { type: String },
          responseDump: { type: String },
          responseMeta: { type: String },
          previewSvg: { type: String },
          renderTick: { type: Number },
        };

        static styles = css`:host { display: block; }`;

        constructor() {
          super();
          this.selectedExample = EXAMPLES[0].id;
          this.diagramType = EXAMPLES[0].diagramType;
          this.outputFormat = 'Svg';
          this.source = EXAMPLES[0].source;
          this.encoded = false;
          this.autoRender = false;
          this.theme = localStorage.getItem('kroki.theme') || 'dark';
          this.statusText = 'Idle';
          this.statusClass = '';
          this.responseDump = 'No response yet.';
          this.responseMeta = 'Render metadata will appear here.';
          this.previewSvg = '';
          this.renderTick = 0;
          document.documentElement.dataset.theme = this.theme;
        }

        selectExample(exampleId) {
          const match = EXAMPLE_MAP[exampleId];
          if (!match) return;
          this.selectedExample = exampleId;
          this.diagramType = match.diagramType;
          this.source = match.source;
          if (this.autoRender) this.scheduleRender();
        }

        toggleTheme() {
          this.theme = this.theme === 'dark' ? 'light' : 'dark';
          document.documentElement.dataset.theme = this.theme;
          localStorage.setItem('kroki.theme', this.theme);
        }

        setStatus(text, cls = '') {
          this.statusText = text;
          this.statusClass = cls;
        }

        scheduleRender() {
          this.renderTick += 1;
          const thisTick = this.renderTick;
          setTimeout(() => {
            if (this.renderTick === thisTick) this.renderNow();
          }, 260);
        }

        async renderNow() {
          const body = {
            source: this.encoded ? '' : this.source,
            source_encoded: this.encoded ? btoa(this.source) : null,
            source_encoding: this.encoded ? 'base64' : 'plain',
            diagram_type: this.diagramType,
            output_format: this.outputFormat,
          };
          this.setStatus('Rendering...', 'warn');
          try {
            const res = await fetch('/render', {
              method: 'POST',
              headers: { 'content-type': 'application/json' },
              body: JSON.stringify(body),
            });
            const raw = await res.text();
            let parsed = null;
            try {
              parsed = JSON.parse(raw);
            } catch (_) {
              parsed = null;
            }

            if (!res.ok) {
              this.previewSvg = '';
              this.responseMeta = `status=${res.status}\\ncontent-type=${res.headers.get('content-type') || 'unknown'}`;
              this.responseDump = parsed ? JSON.stringify(parsed, null, 2) : raw;
              this.setStatus(`Error (${res.status})`, 'err');
              return;
            }

            const payload = parsed || {};
            this.responseMeta = [
              `status=${res.status}`,
              `content-type=${payload.content_type || 'unknown'}`,
              `duration_ms=${payload.duration_ms ?? 'n/a'}`,
              `diagram_type=${this.diagramType}`,
              `output_format=${this.outputFormat}`,
              `payload_mode=${this.encoded ? 'base64' : 'plain'}`
            ].join('\\n');
            this.responseDump = JSON.stringify(payload, null, 2);

            if (payload.content_type === 'image/svg+xml' && typeof payload.data === 'string' && payload.data.includes('<svg')) {
              this.previewSvg = payload.data;
              this.setStatus('Rendered', '');
            } else {
              this.previewSvg = '';
              this.setStatus('Rendered (preview unavailable for this format)', 'warn');
            }
          } catch (error) {
            this.previewSvg = '';
            this.responseMeta = 'request failed';
            this.responseDump = String(error);
            this.setStatus('Network error', 'err');
          }
        }

        render() {
          return html`
            <div class="shell">
              <header class="topbar">
                <div class="brand">
                  <span class="brand-dot"></span>
                  <div>
                    <h1>kroki Playground</h1>
                    <p>Editor-style layout for rapid render iteration</p>
                  </div>
                </div>
                <div class="topbar-actions">
                  <span class="pill">/render</span>
                  <span class="pill">/capabilities</span>
                  <span class="pill">admin: /health /metrics</span>
                  <button class="btn-theme" @click=${this.toggleTheme}>
                    ${this.theme === 'dark' ? 'Light Theme' : 'Dark Theme'}
                  </button>
                </div>
              </header>

              <section class="layout">
                <aside class="panel">
                  <h2 class="panel-title">Examples</h2>
                  <div class="example-list">
                    ${EXAMPLES.map((ex) => html`
                      <button
                        class="example-item ${this.selectedExample === ex.id ? 'active' : ''}"
                        @click=${() => this.selectExample(ex.id)}
                      >
                        <strong>${ex.title}</strong>
                        <small>${ex.description}</small>
                      </button>
                    `)}
                  </div>
                  <p class="hint">Select an example to prefill the editor, then tweak and render.</p>
                </aside>

                <main class="panel">
                  <h2 class="panel-title">Editor</h2>
                  <div class="controls">
                    <label>
                      Diagram Type
                      <select @change=${(e) => {
                        this.diagramType = e.target.value;
                        if (this.autoRender) this.scheduleRender();
                      }}>
                        <option value="graphviz" ?selected=${this.diagramType === 'graphviz'}>graphviz</option>
                        <option value="d2" ?selected=${this.diagramType === 'd2'}>d2</option>
                        <option value="mermaid" ?selected=${this.diagramType === 'mermaid'}>mermaid</option>
                        <option value="bpmn" ?selected=${this.diagramType === 'bpmn'}>bpmn</option>
                        <option value="echo" ?selected=${this.diagramType === 'echo'}>echo</option>
                      </select>
                    </label>
                    <label>
                      Output
                      <select @change=${(e) => {
                        this.outputFormat = e.target.value;
                        if (this.autoRender) this.scheduleRender();
                      }}>
                        <option value="Svg">Svg</option>
                        <option value="Png">Png</option>
                        <option value="WebP">WebP</option>
                        <option value="Pdf">Pdf</option>
                      </select>
                    </label>
                    <label>
                      Payload
                      <select @change=${(e) => {
                        this.encoded = e.target.value === 'base64';
                        if (this.autoRender) this.scheduleRender();
                      }}>
                        <option value="plain" ?selected=${!this.encoded}>plain</option>
                        <option value="base64" ?selected=${this.encoded}>base64</option>
                      </select>
                    </label>
                    <label>
                      Auto
                      <select @change=${(e) => this.autoRender = e.target.value === 'on'}>
                        <option value="off" ?selected=${!this.autoRender}>off</option>
                        <option value="on" ?selected=${this.autoRender}>on</option>
                      </select>
                    </label>
                  </div>
                  <textarea
                    .value=${this.source}
                    @input=${(e) => {
                      this.source = e.target.value;
                      if (this.autoRender) this.scheduleRender();
                    }}
                  ></textarea>
                  <div class="toolbar">
                    <div class="actions">
                      <button class="btn-primary" @click=${this.renderNow}>Render</button>
                      <button class="btn-ghost" @click=${() => this.selectExample(this.selectedExample)}>Reset</button>
                    </div>
                    <span class="pill">${this.diagramType}</span>
                  </div>
                  <p class="hint">Tip: enable Auto mode for near-live updates while editing.</p>
                </main>

                <section class="panel">
                  <h2 class="panel-title">Preview</h2>
                  <div class="status">
                    <span class="dot ${this.statusClass}"></span>
                    <strong>${this.statusText}</strong>
                  </div>
                  <div class="preview-wrap">
                    ${this.previewSvg
                      ? html`<div .innerHTML=${this.previewSvg}></div>`
                      : html`<p class="hint">No render preview yet. Click <strong>Render</strong>.</p>`}
                  </div>
                  <pre class="meta">${this.responseMeta}</pre>
                  <pre class="resp">${this.responseDump}</pre>
                </section>
              </section>
            </div>
          `;
        }
      }
      customElements.define('kroki-playground', KrokiPlayground);
    </script>
  </head>
  <body>
    <kroki-playground></kroki-playground>
  </body>
</html>"#;

async fn capabilities_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProviderMetadata>>, (StatusCode, String)> {
    let capabilities = state
        .registry
        .all_metadata()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    debug!(count = capabilities.len(), "capabilities requested");
    if capabilities.is_empty() {
        error!("capability registry is empty");
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "no providers".to_string(),
        ));
    }
    Ok(Json(capabilities))
}

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use kroki_core::config::{ApiKeyEntry, Config};
    use metrics_exporter_prometheus::PrometheusBuilder;
    use tower::util::ServiceExt;

    #[test]
    fn unit_default_bind_addr_is_localhost_8000() {
        let addr = super::default_bind_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:8000");
    }

    #[test]
    fn unit_default_admin_bind_addr_is_localhost_8081() {
        let addr = super::default_admin_bind_addr();
        assert_eq!(addr.to_string(), "127.0.0.1:8081");
    }

    #[tokio::test]
    async fn integration_render_route_executes_vertical_slice() {
        let app = super::app();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"A -> B","diagram_type":"echo","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn integration_capabilities_route_returns_registered_providers() {
        let app = super::app();
        let request = Request::builder()
            .method("GET")
            .uri("/capabilities")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 1024 * 1024)
            .await
            .expect("capabilities body should be readable");
        let payload = String::from_utf8(bytes.to_vec()).expect("payload should be valid utf-8");
        assert!(
            payload.contains("\"provider_id\":\"graphviz\""),
            "capabilities should include graphviz metadata"
        );
        assert!(
            payload.contains("\"provider_id\":\"d2\""),
            "capabilities should include d2 metadata"
        );
        assert!(
            payload.contains("\"provider_id\":\"mermaid\""),
            "capabilities should include mermaid metadata"
        );
        assert!(
            payload.contains("\"provider_id\":\"bpmn\""),
            "capabilities should include bpmn metadata"
        );
    }

    #[tokio::test]
    async fn integration_playground_route_returns_html() {
        let app = super::app();
        let request = Request::builder()
            .method("GET")
            .uri("/playground")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn integration_render_route_accepts_base64_payload() {
        use base64::Engine;

        let app = super::app();
        let payload = base64::engine::general_purpose::STANDARD.encode("A -> B");
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                "{{\"source\":\"\",\"source_encoded\":\"{payload}\",\"source_encoding\":\"base64\",\"diagram_type\":\"echo\",\"output_format\":\"Svg\"}}"
            )))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn integration_admin_health_route_returns_ok() {
        let app = super::admin_app(None);
        let request = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn integration_admin_metrics_route_returns_text() {
        let handle = PrometheusBuilder::new().install_recorder().ok();
        let app = super::admin_app(handle);
        let request = Request::builder()
            .method("GET")
            .uri("/metrics")
            .body(Body::empty())
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert!(
            response.status() == StatusCode::OK
                || response.status() == StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn integration_render_route_graphviz_when_available() {
        if which::which("dot").is_err() {
            return;
        }

        let app = super::app();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"digraph G { A -> B; }","diagram_type":"graphviz","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn integration_render_route_d2_when_available() {
        if which::which("d2").is_err() {
            return;
        }

        let app = super::app();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"a -> b","diagram_type":"d2","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn integration_render_route_mermaid_status_depends_on_runtime_tooling() {
        let app = super::app();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"graph TD; A-->B;","diagram_type":"mermaid","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        if which::which("mmdc").is_ok() {
            assert_eq!(response.status(), StatusCode::OK);
        } else {
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[tokio::test]
    async fn integration_render_route_bpmn_returns_not_implemented_status() {
        let app = super::app();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"<?xml version=\"1.0\"?><definitions></definitions>","diagram_type":"bpmn","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn integration_render_route_maps_validation_failure_to_400() {
        let app = super::app();
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"   ","diagram_type":"echo","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn integration_auth_middleware_rejects_request_without_key() {
        let mut config = Config::default();
        config.server.auth.enabled = true;
        config.server.auth.api_keys = vec![ApiKeyEntry {
            key: "test-key".to_string(),
            label: "test".to_string(),
            rate_limit: None,
        }];

        let app = super::app_with_config(&config);
        let request = Request::builder()
            .method("POST")
            .uri("/render")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"source":"A -> B","diagram_type":"echo","output_format":"Svg"}"#,
            ))
            .expect("request should be valid");

        let response = app
            .oneshot(request)
            .await
            .expect("app should handle request");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn integration_rate_limit_middleware_rejects_excess_requests() {
        let mut config = Config::default();
        config.server.rate_limit.enabled = true;
        config.server.rate_limit.requests_per_second = 1;
        config.server.rate_limit.burst_size = 1;

        let app = super::app_with_config(&config);
        let req = || {
            Request::builder()
                .method("POST")
                .uri("/render")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"source":"A -> B","diagram_type":"echo","output_format":"Svg"}"#,
                ))
                .expect("request should be valid")
        };

        let first = app
            .clone()
            .oneshot(req())
            .await
            .expect("first request should complete");
        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .clone()
            .oneshot(req())
            .await
            .expect("second request should complete");
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn integration_circuit_breaker_opens_after_provider_failure() {
        let mut config = Config::default();
        config.server.circuit_breaker.enabled = true;
        config.server.circuit_breaker.failure_threshold = 1;
        config.server.circuit_breaker.reset_timeout_secs = 60;

        let app = super::app_with_config(&config);
        let req = || {
            Request::builder()
                .method("POST")
                .uri("/render")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"source":"<?xml version=\"1.0\"?><definitions></definitions>","diagram_type":"bpmn","output_format":"Svg"}"#,
                ))
                .expect("request should be valid")
        };

        let first = app
            .clone()
            .oneshot(req())
            .await
            .expect("first request should complete");
        assert_eq!(first.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let second = app
            .clone()
            .oneshot(req())
            .await
            .expect("second request should complete");
        assert_eq!(second.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
