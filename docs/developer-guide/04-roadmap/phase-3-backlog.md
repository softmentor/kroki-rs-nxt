---
title: Phase 3 Provider Migration Backlog
label: kroki-rs-nxt.developer-guide.phase3-backlog
---

# Phase 3 Provider Migration Backlog

## Purpose

Define concrete Phase 3 migration batches, dependency ordering, and risk notes after Phase 2 bootstrap closure.

Date baseline: **2026-03-03**

## Batch Breakdown

### Batch 3.1: Command Providers Foundation

Scope:

- Graphviz (`dot`)
- D2
- Shared command execution wrapper and timeout handling

Progress update (2026-03-03):

- Graphviz provider landed.
- D2 provider landed.
- Shared command execution abstraction is still pending; providers currently use per-provider command paths with common timeout/error conventions.

Deliverables:

- Command provider base implementation in `kroki-core`
- Registry wiring and capability metadata for command providers
- Conformance tests for deterministic SVG output contracts

Key dependencies:

- Frozen request/response/error contracts (`core-contracts.md`)
- Process timeout and error mapping policy in adapters

Risk notes:

- Host tool availability drift across CI/local environments
- Output non-determinism due to tool versions

### Batch 3.2: Browser Providers Foundation

Scope:

- Mermaid
- BPMN
- Browser execution bootstrap using `native-browser` feature

Progress update (2026-03-03):

- Mermaid provider runtime path is wired through `mmdc` in `native-browser` builds.
- Explicit error/status mapping is now applied at server render boundary.
- BPMN provider baseline is registered with status-mapped pending runtime implementation.
- Browser pooling/runtime hardening remains pending for next increments.

Deliverables:

- Browser provider interface and lifecycle hooks
- Minimal browser session/pool management baseline
- Contract tests with fixture-driven input/output checks

Key dependencies:

- Batch 3.1 error mapping and timeout handling
- Feature-gated runtime path for environments without browser support

Risk notes:

- Headless browser lifecycle flakiness
- Increased CI runtime cost and caching complexity

### Batch 3.3: Pipeline and Plugin Providers

Scope:

- Vega / VegaLite multi-step pipeline
- Plugin provider handshake through `kroki-plugins`

Deliverables:

- Pipeline execution chain contract and validation
- Plugin subprocess protocol baseline and error semantics
- Integration tests for pipeline edge cases and plugin failures

Key dependencies:

- Batch 3.1 execution and error infrastructure
- Plugin lifecycle ownership model from core/plugins

Risk notes:

- Multi-step error attribution ambiguity
- Plugin protocol versioning drift

### Batch 3.4: Transport and Middleware Hardening

Scope:

- Server route expansion (`/render`, `/render/batch`, health endpoints)
- Auth, rate limiting, and metrics baseline middleware

Deliverables:

- Adapter-level DTO and error response conventions
- Middleware policy wiring with tests
- Smoke checks for critical server paths

Key dependencies:

- Batches 3.1-3.3 provider semantics and errors
- Observability baseline conventions

Risk notes:

- Middleware ordering regressions
- Inconsistent status code mapping from domain errors

## Dependency Ordering

1. Batch 3.1 (Command) must land first.
2. Batch 3.2 (Browser) depends on 3.1 error and timeout conventions.
3. Batch 3.3 (Pipeline/Plugin) depends on 3.1 execution baseline and 3.2 optional browser interop.
4. Batch 3.4 (Transport hardening) consolidates provider semantics from earlier batches.

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
