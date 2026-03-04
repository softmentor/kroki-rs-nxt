---
title: Phase 3 Provider Migration Backlog
label: kroki-rs-nxt.developer-guide.phase3-backlog
---

# Phase 3 Provider Migration Backlog

## Purpose

Define concrete Phase 3 migration batches, dependency ordering, and risk notes after Phase 2 bootstrap closure.

Date baseline: **2026-03-03** | Last updated: **2026-03-04**

## Batch Breakdown

### Batch 3.1: Command Providers Foundation

Status: **Complete**

Scope:

- Graphviz (`dot`)
- D2
- Ditaa
- Excalidraw
- Wavedrom
- Shared command execution wrapper and timeout handling

Completion (2026-03-04):

- Graphviz, D2, Ditaa, Excalidraw, and Wavedrom providers implemented with timeout/error handling.
- All command providers registered with capability metadata in CLI and server registries.
- Providers use per-provider command paths with common timeout/error conventions via `tokio::process::Command`.
- SVG-to-PNG/WebP format conversion wired via `resvg` in transport layer.
- Integration and conformance tests in place.

### Batch 3.2: Browser Providers Foundation

Status: **Complete**

Scope:

- Mermaid (CDP primary + `mmdc` CLI fallback)
- BPMN (CDP)
- Browser execution using `native-browser` feature

Completion (2026-03-04):

- Mermaid provider wired with dual-path: CDP primary via `BrowserManager`, `mmdc` CLI fallback.
- BPMN provider fully implemented with CDP-based rendering.
- Browser pool management with configurable pool size, context TTL recycle, and adaptive failure recycle.
- Browser engine URL configuration threaded through `Config` → `BrowserManager` → providers.
- Font injection via cache-backed `@font-face` harness.
- Feature-gated `native-browser` path with graceful fallback for environments without browser support.

### Batch 3.3: Pipeline and Plugin Providers

Status: **Pipeline Complete / Plugin Planned**

Scope:

- Vega / VegaLite multi-step pipeline
- Plugin provider handshake through `kroki-plugins`

Completion (2026-03-04 — Pipeline):

- Vega provider implemented (`vg2svg` subprocess).
- Vega-Lite provider implemented (`vl2vg` → `vg2svg` two-stage pipeline).
- Both providers registered with capability metadata and integration tests.
- Pipeline error attribution handled via per-stage error messages.

Remaining (Plugin):

- Plugin subprocess protocol baseline and error semantics — deferred to later Phase 3 increment.
- `core/plugins` crate scaffolded but not yet wired.

### Batch 3.4: Transport and Middleware Hardening

Status: **Substantially Complete**

Scope:

- Standard Kroki API endpoints (wire-compatible with original Kroki)
- RFC 7807 Problem Details error responses
- Auth, rate limiting, circuit breaker middleware
- SVG-to-raster format conversion pipeline

Completion (2026-03-04):

- Standard endpoints: `POST /{type}/{format}`, `POST /` (JSON), `GET /{type}/{format}/{encoded}`.
- RFC 7807 `application/problem+json` error responses on all error paths.
- Auth, rate limiting, and circuit breaker middleware fully wired.
- SVG-to-PNG and SVG-to-WebP conversion via `resvg` + `image` in `adapters/transport/src/conversion.rs`.
- Input/output size guardrails enforced.

Remaining:

- Admin dashboard (HTML UI) — gap
- Richer Prometheus metrics (beyond request count and duration) — partial

## Dependency Ordering

1. ~~Batch 3.1 (Command) must land first.~~ **Done.**
2. ~~Batch 3.2 (Browser) depends on 3.1 error and timeout conventions.~~ **Done.**
3. ~~Batch 3.3 (Pipeline) depends on 3.1 execution baseline.~~ **Done (pipeline). Plugin deferred.**
4. ~~Batch 3.4 (Transport hardening) consolidates provider semantics from earlier batches.~~ **Substantially done.**

## Remaining Phase 3 Work

- Plugin system (`core/plugins`) — L effort
- Per-tool config (bin_path, timeout, config overrides) — M effort
- Filesystem cache (`adapters/storage`) — M effort
- Admin dashboard — M effort
- PDF output format — S effort

## Readiness Gates per Batch

- Provider contract tests pass.
- Integration tests pass in `dwf check:pr`.
- Known risk mitigations are documented in execution tracker.
- No regression in baseline CLI/server vertical slice behavior.

## Ownership Model (Initial)

- Core/Provider owner: **Core Platform Lead**
- Adapter/Transport owner: **Runtime Surfaces Lead**
- CI/Test owner: **Developer Productivity Lead**
- Documentation owner: **Documentation Maintainer**

Owner names can be mapped once team staffing is finalized.
