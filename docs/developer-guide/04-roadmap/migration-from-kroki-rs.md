---
title: Migration from kroki-rs
label: kroki-rs-nxt.developer-guide.migration-from-kroki-rs
---
# Migration from kroki-rs

## Overview

kroki-rs-nxt is a **complete rewrite** of kroki-rs (v0.0.8). It is not backward compatible and makes no attempt to maintain API or binary compatibility with the original.

The rewrite:
- Splits a single-crate project into a multi-crate hexagonal workspace
- Expands from a single surface (CLI + server) to multiple surfaces (CLI TUI, server, desktop, web, VS Code)
- Introduces formal ports-and-adapters boundaries
- Uses devflow for workflow orchestration instead of ad-hoc Makefile targets

kroki-rs will remain in **maintenance mode** (bug fixes and critical releases only) during the transition.

---

## What Changes

| Aspect | kroki-rs | kroki-rs-nxt |
|--------|----------|-------------|
| **Structure** | Single crate | Multi-crate Cargo + pnpm workspace |
| **Architecture** | Monolithic modules | Hexagonal (ports & adapters) |
| **CLI** | Basic clap CLI | Ratatui TUI with interactive mode |
| **Server** | Axum HTTP API | Same, but as separate binary crate |
| **Desktop** | None | Tauri app (Rust + Lit) |
| **Web UI** | None | Lit + TypeScript dashboard |
| **VS Code** | None | TypeScript extension |
| **Plugin system** | PluginProvider in core | Dedicated `kroki-plugins` crate |
| **Build workflow** | Manual Makefile | devflow (`dwf`) orchestration |
| **CI** | Ad-hoc GitHub Actions | Generated via `dwf ci:generate` |

---

## What Carries Over

The following patterns and domain knowledge from kroki-rs are leveraged in the new architecture:

- **DiagramProvider trait** — the core abstraction remains; refined with `DiagramRequest`/`DiagramResponse` types
- **Provider categories** — Command, Browser, Pipeline, Plugin patterns are preserved
- **DiagramRegistry** — provider registration and lookup pattern
- **Configuration model** — TOML-based config with environment variable overrides
- **Server middleware patterns** — auth, rate limiting, circuit breaker, metrics
- **Browser pool management** — CDP-based headless Chrome with connection pooling
- **Caching strategy** — SHA256-keyed filesystem cache
- **Utility functions** — Base64/Zlib decoding, SVG-to-WebP conversion, font management

---

## Module Mapping

Where kroki-rs code maps to in kroki-rs-nxt:

| kroki-rs Location | kroki-rs-nxt Location | Crate |
|---|---|---|
| `src/diagrams/mod.rs` (DiagramProvider trait) | `core/sdk-rust/src/ports/diagram.rs` | `kroki-core` |
| `src/diagrams/error.rs` | `core/sdk-rust/src/error.rs` | `kroki-core` |
| `src/diagrams/registry.rs` | `core/sdk-rust/src/services/registry.rs` | `kroki-core` |
| `src/diagrams/providers/cmd.rs` | `core/sdk-rust/src/providers/command.rs` | `kroki-core` |
| `src/diagrams/providers/mermaid.rs` | `core/sdk-rust/src/providers/mermaid.rs` | `kroki-core` |
| `src/diagrams/providers/bpmn.rs` | `core/sdk-rust/src/providers/bpmn.rs` | `kroki-core` |
| `src/diagrams/providers/d2.rs` | `core/sdk-rust/src/providers/d2.rs` | `kroki-core` |
| `src/diagrams/providers/vega.rs` | `core/sdk-rust/src/providers/vega.rs` | `kroki-core` |
| `src/diagrams/providers/wavedrom.rs` | `core/sdk-rust/src/providers/wavedrom.rs` | `kroki-core` |
| `src/diagrams/providers/ditaa.rs` | `core/sdk-rust/src/providers/ditaa.rs` | `kroki-core` |
| `src/diagrams/providers/excalidraw.rs` | `core/sdk-rust/src/providers/excalidraw.rs` | `kroki-core` |
| `src/diagrams/providers/plugin.rs` | `core/plugins/src/provider.rs` | `kroki-plugins` |
| `src/browser/manager.rs` | `core/sdk-rust/src/browser/manager.rs` | `kroki-core` |
| `src/browser/backend.rs` | `core/sdk-rust/src/browser/backend.rs` | `kroki-core` |
| `src/browser/native.rs` | `core/sdk-rust/src/browser/native.rs` | `kroki-core` |
| `src/config/mod.rs` | `core/sdk-rust/src/config/mod.rs` | `kroki-core` |
| `src/server/mod.rs` | `adapters/transport/src/http/server.rs` | `kroki-adapter-transport` |
| `src/server/handlers.rs` | `adapters/transport/src/http/handlers.rs` | `kroki-adapter-transport` |
| `src/server/admin.rs` | `adapters/transport/src/http/admin.rs` | `kroki-adapter-transport` |
| `src/server/metrics.rs` | `adapters/transport/src/http/metrics.rs` | `kroki-adapter-transport` |
| `src/server/middleware/auth.rs` | `adapters/transport/src/middleware/auth.rs` | `kroki-adapter-transport` |
| `src/server/middleware/rate_limit.rs` | `adapters/transport/src/middleware/rate_limit.rs` | `kroki-adapter-transport` |
| `src/server/middleware/circuit_breaker.rs` | `adapters/transport/src/middleware/circuit_breaker.rs` | `kroki-adapter-transport` |
| `src/interface/dtos.rs` | `adapters/transport/src/dtos.rs` | `kroki-adapter-transport` |
| `src/interface/mapping.rs` | `adapters/transport/src/mapping.rs` | `kroki-adapter-transport` |
| `src/interface/errors.rs` | `adapters/transport/src/errors.rs` | `kroki-adapter-transport` |
| `src/cli/mod.rs` | `apps/cli/src/` | `kroki-cli` |
| `src/main.rs` | Split: `apps/cli/src/main.rs` + `apps/server/src/main.rs` | `kroki-cli`, `kroki-server` |
| `src/utils/mod.rs` (decode) | `core/sdk-rust/src/utils/decode.rs` | `kroki-core` |
| `src/utils/image_converter.rs` | `core/sdk-rust/src/utils/image.rs` | `kroki-core` |
| `src/utils/font_manager.rs` | `core/sdk-rust/src/utils/fonts.rs` | `kroki-core` |
| `src/capabilities.rs` | `core/sdk-rust/src/capabilities.rs` | `kroki-core` |

---

## Key Dependency Changes

| Dependency | kroki-rs | kroki-rs-nxt | Notes |
|-----------|----------|-------------|-------|
| `clap` | CLI parsing | Still used in `kroki-cli` | Supplemented by Ratatui for TUI |
| `ratatui` | Not used | New in `kroki-cli` | Interactive terminal UI |
| `axum` | In single crate | In `kroki-adapter-transport` | Isolated to adapter layer |
| `headless_chrome` | Optional feature | Optional in `kroki-core` | Feature-gated `native-browser` |
| `tauri` | Not used | New in `apps/desktop` | Phase 4 |
| `wasm-bindgen` | Not used | New in `core/sdk-ts` | Phase 4 |

---

## Migration Approach

Code is **not** copy-pasted from kroki-rs. Instead:

1. Study the kroki-rs implementation for each module
2. Design the kroki-rs-nxt interface (trait/struct signatures) following hexagonal patterns
3. Implement fresh, leveraging the domain knowledge and algorithms from kroki-rs
4. Write contract tests that verify the new implementation produces equivalent outputs
5. Use conformance tests to validate parity where applicable

This ensures the new codebase is clean, well-structured, and free of accumulated technical debt from the original.
