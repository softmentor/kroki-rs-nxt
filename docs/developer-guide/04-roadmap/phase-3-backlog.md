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

## Cross-Cutting Critical Tasks (Phase 3)

These tasks lock architecture-critical concerns before broad multi-surface expansion.

| ID | Task | Owner | Status | Target Batch | Acceptance Criteria |
|----|------|-------|--------|--------------|---------------------|
| P3-X01 | Runtime contract versioning policy for `runtime-wasm` | Core/Provider owner | Planned | 3.3 | Contract compatibility policy documented; breaking-change process and semver rules added; contract tests cover version negotiation path. |
| P3-X02 | Cross-host cancellation semantics (web + tauri) | Adapter/Transport owner | Planned | 3.3 | In-flight render cancellation behavior defined and implemented; stale responses dropped by revision id; integration tests cover overlap/cancel race. |
| P3-X03 | Capability metadata cache freshness/invalidation strategy | Core/Provider owner | Planned | 3.3 | TTL + invalidation triggers implemented; stale capability scenarios tested; docs updated for refresh behavior. |
| P3-X04 | Host adapter security boundary hardening | Runtime Surfaces Lead | Planned | 3.4 | Host adapter allowlist model defined; file/command bridge restrictions enforced; negative tests for unsafe calls pass. |
| P3-X05 | Provider-class latency budgets and CI smoke thresholds | CI/Test owner | Planned | 3.4 | SLO targets documented per provider class; smoke checks emit budget metrics; CI gate fails on sustained threshold breach. |
| P3-X06 | Accessibility baseline for playground components | Runtime Surfaces Lead | Planned | 3.4 | Keyboard navigation, labels, and minimum contrast checks in place; accessibility checks integrated into UI test pipeline. |
| P3-X07 | Observability parity taxonomy across surfaces | Adapter/Transport owner | Planned | 3.4 | Shared metric/log event names documented and implemented for CLI, server, web, and desktop flows; parity checklist added to reviews. |
| P3-X08 | Shared test-matrix ownership and scope mapping | CI/Test owner | In Progress | 3.4 | Package-level and cross-surface integration ownership table added; `dwf check:pr` maps to required layers with explicit gaps tracked. |
| P3-X09 | Unified fixture/sample corpus governance | Documentation owner | Planned | 3.3 | Canonical sample set established and referenced by tests/docs/playground presets; fixture update policy documented. |
| P3-X10 | Failure-mode UX contract for missing deps/timeouts/tool errors | Runtime Surfaces Lead | Planned | 3.4 | Consistent user-facing error presentation and fallback behavior defined for all surfaces; snapshot tests cover key failure classes. |

### Status Legend

- **Planned**: not started, scoped with owner and acceptance criteria
- **In Progress**: active implementation or documentation work
- **Complete**: accepted with tests/docs and execution log entry

## Phase 3 Gate Checklist (Mapped to P3-X)

Use this checklist for formal Phase 3 closure review.

| Gate ID | Gate Criterion | Required Task IDs | Required Evidence | Gate Status |
|---------|----------------|-------------------|-------------------|-------------|
| G3-01 | Runtime boundary is stable and versioned | P3-X01, P3-X02, P3-X03 | Updated runtime contract docs, versioning policy, and integration tests for cancel/stale handling | Pending |
| G3-02 | Server/runtime platform is secure and observable | P3-X04, P3-X05, P3-X07 | Security boundary tests, latency budget reports, and shared telemetry taxonomy in docs/code | Pending |
| G3-03 | UI platform quality baseline is production-ready | P3-X06, P3-X10 | Accessibility checks, failure-mode snapshots, and user-facing error/UX contract docs | Pending |
| G3-04 | Developer quality system is reproducible | P3-X08, P3-X09 | Test ownership matrix, fixture governance policy, and `dwf check:pr` coverage mapping | In Progress |
| G3-05 | Capability migration parity is complete for v0.1.0 scope | Batch 3.1, 3.2, 3.3, 3.4 + remaining Phase 3 work list | Provider parity dashboard, conformance test results, and execution closure note in `v010.md` | Pending |

### Gate Decision Record Template

- Date:
- Decision: `Go` or `No-Go`
- Reviewers:
- Evidence Links:
- Follow-up Actions:

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
