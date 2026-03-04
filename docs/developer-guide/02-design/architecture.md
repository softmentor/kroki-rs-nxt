---
title: Architecture Overview
label: kroki-rs-nxt.developer-guide.architecture
---

# Architecture Overview

## Design Philosophy

kroki-rs-nxt follows the **Hexagonal Architecture** (Ports & Adapters) pattern. This separates pure domain logic from infrastructure concerns, enabling multiple interaction surfaces to share the same core engine.

```mermaid
flowchart TB
    subgraph L3["Apps (Surfaces)"]
        CLI["apps/cli"]
        SRV["apps/server"]
        DESK["apps/desktop (planned)"]
        WEB["apps/web-app (planned)"]
        VSX["apps/vscode-ext (planned)"]
        MYST["apps/myst-plugin (planned)"]
    end

    subgraph L2["Adapters"]
        TR["adapters/transport"]
        ST["adapters/storage"]
    end

    subgraph L1["Core"]
        CORE["core/sdk-rust"]
        PLUG["core/plugins"]
        TSSDK["core/sdk-ts (planned expansion)"]
    end

    CLI --> TR
    SRV --> TR
    DESK --> TR
    WEB --> TR
    VSX --> TR
    MYST --> TR
    TR --> CORE
    ST --> CORE
    TR --> PLUG
    TSSDK --> CORE
```

### Dependency Direction Rule

**`apps → adapters → core`**

Core MUST NEVER depend on an App or Adapter. Adapters MUST NEVER depend on an App.

---

## Architecture State (Current vs Target)

### Current Repository State (Phase 3 — v0.1.0-alpha.1)

- `core/sdk-rust` has all 9 production providers implemented (Graphviz, D2, Ditaa, Excalidraw, Wavedrom, Mermaid, BPMN, Vega, Vega-Lite) plus Echo bootstrap stub.
- `core/sdk-rust` includes browser subsystem (`browser/manager.rs`, `browser/native.rs`, `browser/backend.rs`) and font management.
- `adapters/transport` provides HTTP DTO mapping, format conversion (SVG to PNG/WebP via `resvg`), and middleware (auth, rate limiting, circuit breaker).
- `adapters/storage` is scaffolded (cache adapter not yet implemented).
- `apps/cli` is feature-complete for Phase 3: `convert`, `encode`, `decode`, `completions`, `version`, file auto-detection, stdin/stdout piping.
- `apps/server` implements standard Kroki API endpoints, RFC 7807 error responses, and all middleware.
- `apps/desktop`, `apps/web-app`, and `apps/vscode-ext` have bootstrap baseline packages ready for Phase 4.

### Target State (Phases 4-5)

- Plugin system implementation (`core/plugins`).
- Filesystem caching via `adapters/storage`.
- Per-tool configuration (bin_path, timeout, config overrides).
- Additional surfaces (Desktop, Web, VS Code) activated and consuming shared core logic through `core/sdk-ts`.

---

## Layers

### Responsibility Summary

| Layer | Owns | Does Not Own |
|-------|------|--------------|
| Apps | User entry points, app lifecycle, surface UX/API endpoints | Core business rules, provider internals |
| Adapters | IO boundaries, DTO mapping, protocol translation, middleware | Domain policy decisions |
| Core | Domain contracts, provider orchestration, business semantics | Transport concerns, app runtime concerns |

### Core (`core/`)

Pure domain logic with zero infrastructure dependencies. Defines interface traits (ports) that adapters implement.

| Crate | Purpose |
|-------|---------|
| `core/sdk-rust` (`kroki-core`) | Primary business logic: traits, domain models, providers, config |
| `core/plugins` (`kroki-plugins`) | Plugin discovery, loading, and lifecycle management |
| `core/sdk-ts` | Wasm/FFI bindings exposing Rust domain logic to TypeScript surfaces |

### Adapters (`adapters/`)

Concrete implementations of core traits for specific technologies.

| Crate | Purpose |
|-------|---------|
| `adapters/storage` (`kroki-adapter-storage`) | Filesystem cache (SHA256-keyed), future DB backends |
| `adapters/transport` (`kroki-adapter-transport`) | HTTP handlers (Axum), IPC for Tauri, CLI dispatch |

### Apps (`apps/`)

Entry points that compose Core and Adapters into runnable applications.

| App | Stack | Description |
|-----|-------|-------------|
| `apps/cli` (`kroki-cli`) | Rust (Ratatui TUI) | Interactive terminal UI for diagram conversion |
| `apps/server` (`kroki-server`) | Rust (Axum + Lit playground route) | Standard Kroki API (`/{type}/{format}`, `/`, `/{type}/{format}/{encoded}`), legacy `/render`, `/capabilities`, `/playground` + admin API (`/health`, `/metrics`) with dev/prod runtime modes and RFC 7807 error responses |
| `apps/desktop` | Tauri (Rust + Lit/TS) | Native desktop app with embedded web UI (planned) |
| `apps/vscode-ext` | TypeScript | VS Code extension for in-editor diagram preview (planned) |
| `apps/web-app` | Lit + TypeScript | Standalone web dashboard (planned) |
| `apps/myst-plugin` | TypeScript | MyST plugin surface for documentation-native rendering workflows (planned) |

### Shared (`shared/`)

Cross-stack resources used by multiple surfaces.

| Directory | Purpose |
|-----------|---------|
| `shared/design-system` | Shared Lit web components and CSS design tokens |
| `shared/scripts` | Global CI/CD and build scripts |

### Component and Interface View

```mermaid
flowchart LR
    CLI["kroki-cli"]
    SERVER["kroki-server"]
    TRANSPORT["kroki-adapter-transport"]
    CONV["Format Conversion<br/>(resvg + image)"]
    STORAGE["kroki-adapter-storage"]
    REGISTRY["DiagramRegistry"]
    PROVIDER["DiagramProvider"]
    CMD["CommandProviders<br/>(Graphviz, D2, Ditaa,<br/>Excalidraw, Wavedrom)"]
    BROWSER["BrowserProviders<br/>(Mermaid, BPMN)"]
    PIPELINE["PipelineProviders<br/>(Vega, Vega-Lite)"]

    CLI -->|"RenderRequestDto"| TRANSPORT
    SERVER -->|"RenderRequestDto"| TRANSPORT
    TRANSPORT -->|"DiagramRequest<br/>(always SVG)"| REGISTRY
    REGISTRY -->|"resolve by diagram_type"| PROVIDER
    CMD -.implements.-> PROVIDER
    BROWSER -.implements.-> PROVIDER
    PIPELINE -.implements.-> PROVIDER
    REGISTRY -->|"DiagramResponse<br/>(SVG bytes)"| TRANSPORT
    TRANSPORT -->|"SVG → PNG/WebP"| CONV
    CONV -->|"RenderResponseDto<br/>(target format)"| TRANSPORT
    TRANSPORT --> CLI
    TRANSPORT --> SERVER
    STORAGE -.planned cache boundary.-> REGISTRY
```

Transport request contract supports two source paths:
- plain source via `source`
- encoded source via `source_encoded` + `source_encoding` (`plain`, `base64`, `base64_deflate`)

Format conversion pipeline: providers always generate SVG. The transport layer converts to the requested target format (PNG/WebP) post-render using `resvg` for rasterisation and the `image` crate for encoding. This is implemented in `adapters/transport/src/conversion.rs`.

---

## Core Domain Model

The domain model is extracted from kroki-rs and refined for hexagonal architecture boundaries.

### DiagramProvider (Port)

The central abstraction. Every diagram type implements this trait.

```rust
#[async_trait]
pub trait DiagramProvider: Send + Sync {
    /// Validate the diagram source before generation.
    fn validate(&self, source: &str) -> DiagramResult<()>;

    /// Generate a diagram from source in the specified format.
    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse>;

    /// Return the list of output formats this provider supports.
    fn supported_formats(&self) -> &[OutputFormat];
}
```

### Provider Categories

| Category | Description | Examples |
|----------|-------------|---------|
| **Command** | Wraps CLI tools via subprocess | Graphviz (dot), D2, Ditaa, Wavedrom, Excalidraw |
| **Browser** | Evaluates JS in headless Chrome (CDP) | Mermaid, BPMN |
| **Pipeline** | Multi-step conversion chains | Vega-Lite → Vega → SVG |
| **Plugin** | External subprocess via plugin protocol | User-defined custom tools |

### DiagramRegistry

Central registry for provider discovery and lookup.

```rust
pub struct DiagramRegistry {
    providers: HashMap<String, Arc<dyn DiagramProvider>>,
}

impl DiagramRegistry {
    pub fn register(&mut self, name: &str, provider: Arc<dyn DiagramProvider>);
    pub fn get(&self, name: &str) -> Option<Arc<dyn DiagramProvider>>;
    pub fn known_types(&self) -> Vec<String>;
}
```

### CapabilityRegistry

Provider metadata contract registry for discovery and planning:

```rust
pub struct ProviderMetadata {
    pub provider_id: String,
    pub category: ProviderCategory,
    pub runtime: RuntimeDependency,
    pub supported_formats: Vec<OutputFormat>,
    pub description: String,
}

pub struct CapabilityRegistry { /* provider metadata map */ }
```

Used for:
- capability introspection endpoints (for example `/capabilities`)
- provider migration planning by category/runtime dependency
- enforcing explicit metadata contracts as new providers land

### Domain Types

```rust
pub struct DiagramRequest {
    pub source: String,
    pub diagram_type: String,
    pub output_format: OutputFormat,
    pub options: DiagramOptions,
}

pub struct DiagramResponse {
    pub data: Vec<u8>,
    pub content_type: String,
    pub duration_ms: u64,
}

pub enum OutputFormat {
    Svg,
    Png,
    WebP,
    Pdf,
}

pub enum DiagramError {
    ValidationFailed(String),
    ToolNotFound(String),
    ExecutionTimeout { tool: String, timeout_ms: u64 },
    ProcessFailed(String),
    UnsupportedFormat { format: String, provider: String },
    Io(std::io::Error),
    Internal(String),
}
```

For the frozen Phase 2 baseline contract and change-control rules, see:
- [Core Contract Boundaries (v0.1.0-alpha.0)](#kroki-rs-nxt.developer-guide.core-contracts)

---

## Cross-Cutting Concerns

### Configuration

- **Runtime config** (`kroki.toml`): server settings, tool paths, timeouts, auth, rate limits
- **Build config** (`devflow.toml`): workflow orchestration, CI targets, extensions
- **Environment overrides**: `KROKI_<TOOL>_BIN`, `KROKI_<TOOL>_TIMEOUT`, etc.

### Observability

- **Structured logging**: `tracing` with configurable log levels and env-filter
- **Metrics**: `metrics` crate with Prometheus exporter
- **Per-provider tracking**: request count, duration, payload size, error types

### Browser Pool

- Headless Chrome via CDP (feature-gated: `native-browser`)
- Connection pool with configurable size
- Context reuse with TTL for memory management
- Health metrics: active contexts, idle slots

### Caching

- **Key**: SHA256 hash of (diagram_type, format, source, options)
- **Storage**: filesystem (configurable directory), extensible to other backends
- **Strategy**: check cache → generate → store result

---

## Wasm/FFI Bridge (`core/sdk-ts`)

TypeScript surfaces (desktop frontend, web-app, vscode-ext, myst-plugin) access core domain logic through Wasm bindings generated from `core/sdk-rust`.

```mermaid
flowchart LR
    RUST["core/sdk-rust (Rust)"]
    WASM["core/sdk-ts (TypeScript/Wasm)"]
    DAPP["apps/desktop"]
    WAPP["apps/web-app"]
    VSC["apps/vscode-ext"]
    MYSTAPP["apps/myst-plugin"]

    RUST -->|"wasm-pack build"| WASM
    WASM --> DAPP
    WASM --> WAPP
    WASM --> VSC
    WASM --> MYSTAPP
```

This ensures business logic is written once in Rust and shared across surfaces.

---

## Runtime Flow Diagrams

### CLI Convert Flow

```mermaid
sequenceDiagram
    participant U as "Developer/User"
    participant C as "apps/cli"
    participant T as "adapters/transport"
    participant R as "DiagramRegistry"
    participant P as "Provider (e.g. GraphvizProvider)"
    participant Conv as "Format Conversion (resvg)"

    U->>C: Run `kroki convert -t graphviz -f png`
    C->>T: Build `RenderRequestDto` and call `render_diagram`
    T->>T: Override output_format to SVG
    T->>R: Convert to `DiagramRequest` and call `render_with_registry`
    R->>R: Resolve provider by `diagram_type`
    R->>P: `validate(source)`
    P-->>R: Ok
    R->>P: `generate(request)`
    P-->>R: `DiagramResponse` (SVG payload)
    R-->>T: `DiagramResponse`
    T->>Conv: `convert_svg(svg_data, PNG)`
    Conv-->>T: PNG bytes
    T-->>C: `RenderResponseDto` (PNG)
    C-->>U: Write output file
```

### Server Render Flow (`POST /{type}/{format}`)

```mermaid
sequenceDiagram
    participant Client as "HTTP Client"
    participant S as "apps/server"
    participant T as "adapters/transport"
    participant R as "DiagramRegistry"
    participant P as "Provider"
    participant Conv as "Format Conversion"

    Client->>S: POST /graphviz/svg (text/plain body)
    S->>T: `render_diagram(registry, RenderRequestDto)`
    T->>T: Override format to SVG for provider
    T->>R: `render_with_registry(DiagramRequest)`
    R->>P: validate + generate
    P-->>R: `DiagramResponse` (SVG)
    R-->>T: `DiagramResponse`
    T->>Conv: `convert_svg()` if target != SVG
    Conv-->>T: Converted bytes
    T-->>S: `RenderResponseDto`
    S-->>Client: 200 + binary response (image/svg+xml or image/png)
```

Error responses use RFC 7807 Problem Details (`application/problem+json`).

---

## Architectural Decision Records

Key decisions are documented in [Design Decisions (ADRs)](#kroki-rs-nxt.developer-guide.adr.index):

| ADR | Decision |
|-----|----------|
| [ADR-001](#kroki-rs-nxt.adr.0001) | Hexagonal architecture with apps/adapters/core layering |
| [ADR-002](#kroki-rs-nxt.adr.0002) | Single monorepo for all surfaces (split only when evidence justifies) |
| [ADR-003](#kroki-rs-nxt.adr.0003) | devflow v0.2.0 as workflow orchestration from day one |
| [ADR-004](#kroki-rs-nxt.adr.0004) | Wasm bridge for Rust-to-TypeScript shared logic |
| [ADR-005](#kroki-rs-nxt.adr.0005) | Ratatui for CLI TUI (upgrade from clap-only) |
| [ADR-006](#kroki-rs-nxt.adr.0006) | Test structure and taxonomy for maintainable multi-surface development |
