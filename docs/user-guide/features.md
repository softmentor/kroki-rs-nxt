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

Production-ready HTTP API server with authentication, rate limiting, circuit breaker, and Prometheus metrics.

- **Stack**: Rust + Axum + Tower
- **Binary**: `kroki-server`
- **Location**: `apps/server/`
- **Endpoints**: `/render`, `/render/batch`, `/health`, `/metrics`

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

---

## Diagram Providers

### Command Providers

Wrap CLI tools via subprocess execution:

| Provider | Tool | Formats |
|----------|------|---------|
| Graphviz | `dot` | SVG, PNG, PDF |
| D2 | `d2` | SVG, PNG |
| Ditaa | `ditaa` | SVG, PNG |
| Wavedrom | `wavedrom-cli` | SVG, PNG |
| Excalidraw | `excalidraw` | SVG, PNG |

### Browser Providers

Evaluate JavaScript in headless Chrome via CDP:

| Provider | Engine | Formats |
|----------|--------|---------|
| Mermaid | Mermaid.js | SVG, PNG |
| BPMN | bpmn-js | SVG, PNG |

### Pipeline Providers

Multi-step conversion chains:

| Provider | Pipeline | Formats |
|----------|----------|---------|
| Vega-Lite | vegalite → vega → SVG | SVG, PNG |

### Plugin Providers

User-defined external tools via subprocess protocol. Configure custom diagram tools in `kroki.toml`.

---

## Production Features

- **Authentication**: API key-based with per-key rate limits
- **Rate Limiting**: Token-bucket algorithm (per-IP and per-key)
- **Circuit Breaker**: Per-provider with configurable thresholds
- **Metrics**: Prometheus export with per-provider tracking
- **Caching**: SHA256-keyed filesystem cache
- **Observability**: Structured logging via `tracing`

---

## Shared Core via Wasm

TypeScript surfaces (Desktop UI, Web App, VS Code) access the Rust domain logic through WebAssembly bindings generated from `core/sdk-ts`. This ensures business logic is written once and shared across all surfaces.
