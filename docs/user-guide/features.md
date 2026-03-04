---
title: Features
label: kroki-rs-nxt.user-guide.features
---

# Features

## Multi-Surface Architecture

kroki-rs-nxt is built as a hexagonal architecture where a single Rust core powers multiple interaction surfaces:

### CLI (Ratatui TUI)

Interactive terminal UI for diagram generation. Supports single conversion, batch processing, and an interactive mode with live preview.

- **Stack**: Rust + Ratatui + Clap
- **Binary**: `kroki`
- **Location**: `apps/cli/`

### Server (Axum HTTP API)

HTTP API server with authentication, rate limiting, circuit breaker, and Prometheus metrics baseline.

- **Stack**: Rust + Axum + Tower
- **Binary**: `kroki-server`
- **Location**: `apps/server/`
- **Standard Kroki Endpoints**: `POST /{type}/{format}`, `POST /` (JSON), `GET /{type}/{format}/{encoded}`
- **Legacy Endpoint**: `POST /render` (JSON — retained for backward compatibility)
- **Discovery**: `/capabilities`, `/playground`
- **Admin Endpoints** (admin port): `/health`, `/metrics`
- **Error Format**: RFC 7807 Problem Details (`application/problem+json`)

### Desktop (Tauri)

Native desktop application combining a Rust backend with a Lit/TypeScript frontend.

- **Stack**: Tauri + Rust + Lit/TS
- **Location**: `apps/desktop/`
- **Status**: Phase 4

### Web Dashboard (Lit + TypeScript)

Browser-based diagram editing and preview dashboard.

- **Stack**: Lit + TypeScript + Wasm
- **Location**: `apps/web-app/`
- **Status**: Phase 4

### VS Code Extension

In-editor diagram preview and generation directly within VS Code.

- **Stack**: TypeScript
- **Location**: `apps/vscode-ext/`
- **Status**: Phase 4

### MyST Plugin

Documentation-native plugin surface for MyST-based workflows and live diagram rendering inside authoring pipelines.

- **Stack**: TypeScript
- **Location**: `apps/myst-plugin/`
- **Status**: Planned

---

## Diagram Providers

All 9 production providers are implemented and active. Providers generate SVG natively; the transport layer converts to PNG/WebP post-render via `resvg`.

### Command Providers

Wrap CLI tools via subprocess execution:

| Provider | Tool | Formats |
|----------|------|---------|
| Graphviz | `dot` | SVG, PNG, WebP |
| D2 | `d2` | SVG, PNG, WebP |
| Ditaa | `ditaa` | PNG, SVG, WebP |
| Wavedrom | `wavedrom-cli` | SVG, PNG, WebP |
| Excalidraw | `excalidraw` | SVG, PNG, WebP |

### Browser Providers

Evaluate JavaScript in headless Chrome via CDP (with `mmdc` CLI fallback for Mermaid):

| Provider | Engine | Formats |
|----------|--------|---------|
| Mermaid | Mermaid.js (CDP primary, `mmdc` fallback) | SVG, PNG, WebP |
| BPMN | bpmn-js (CDP) | SVG, PNG, WebP |

### Pipeline Providers

Multi-step conversion chains:

| Provider | Pipeline | Formats |
|----------|----------|---------|
| Vega | `vg2svg` | SVG, PNG, WebP |
| Vega-Lite | `vl2vg` → `vg2svg` | SVG, PNG, WebP |

### Output Format Conversion

Providers always generate SVG. The transport layer handles format conversion post-render:

- **SVG** — returned directly (no conversion)
- **PNG** — SVG rasterised via `resvg`, encoded with `image` crate
- **WebP** — SVG rasterised via `resvg`, encoded as lossless WebP
- **PDF** — declared in `OutputFormat` enum, not yet wired (future work)

Safety limits: maximum rasterisation dimensions are 8192x8192 pixels.

### Plugin Providers

User-defined external tools via subprocess protocol. Configure custom diagram tools in `kroki.toml`. (Planned — `core/plugins` crate is scaffolded.)

---

## Production Features

- **Authentication**: API key-based gate (`server.auth`)
- **Rate Limiting**: Token-bucket algorithm (global per-IP limiter)
- **Circuit Breaker**: Per-provider with configurable thresholds
- **Metrics**: Prometheus export with per-provider tracking
- **Caching**: SHA256-keyed filesystem cache (planned — `adapters/storage` scaffolded)
- **Observability**: Structured logging via `tracing`

---

## Shared Core via Wasm

TypeScript surfaces (Desktop UI, Web App, VS Code, MyST Plugin) access the Rust domain logic through WebAssembly bindings generated from `core/sdk-ts`. This ensures business logic is written once and shared across all surfaces.
