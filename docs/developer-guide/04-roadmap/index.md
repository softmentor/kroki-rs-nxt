---
title: Roadmap
label: kroki-rs-nxt.developer-guide.roadmap.index
---

# Project Roadmap & Migration Plan

## Vision

kroki-rs-nxt is a next-generation, multi-surface diagram platform built from the ground up. It replaces kroki-rs with a clean hexagonal architecture supporting multiple interaction surfaces (CLI, Desktop, Server, VS Code, Web) while preserving the core domain logic and patterns proven in kroki-rs.

This is **not** a backward-compatible migration. kroki-rs-nxt is a fresh codebase that leverages kroki-rs patterns and domain knowledge but makes no compatibility guarantees with the original.

## Tooling Foundation

kroki-rs-nxt uses **devflow v0.2.0** (`dwf`) as its workflow orchestration platform from day one. Gaps and improvements discovered during kroki-rs-nxt development will feed back into **devflow v0.3.0**.

---

## Execution Tracking

Roadmap phases define the plan. Day-to-day delivery status is maintained in **Execution** docs:

- [v0.1.0 Execution Tracker](#kroki-rs-nxt.developer-guide.execution.v010)

Use execution docs to track concrete tasks, ownership, and readiness for phase gates.

---

## Phased Rollout

### Phase 0: Governance and Contracts

**Status: COMPLETE** (delivered as part of devflow)

Deliverables:
- Compatibility and deprecation policy defined
- Versioning strategy for devflow and kroki-rs-nxt established
- Quality gates and ownership model documented
- Baseline ADRs approved

### Phase 1: devflow MVP

**Status: COMPLETE** (v0.2.0 released 2026-03-02)

Deliverables:
- Canonical command graph (`setup`, `fmt`, `lint`, `build`, `test`, `verify`, `ci generate`)
- Rust and Node extension support (builtin + subprocess extensions)
- Container execution with deterministic fingerprinting
- GitHub Actions workflow generation
- Validated across reference projects (rust-lib, node-ts, react, tauri, vscode-extension, python-ext)

### Phase 2: kroki-rs-nxt Bootstrap

**Goal**: Scaffold the monorepo, establish build infrastructure, create foundation crates.

Deliverables:
- Cargo workspace with core, adapter, and app crates
- pnpm workspace for TypeScript surfaces (placeholder)
- `devflow.toml` baseline configuration
- Comprehensive project documentation
- Stub implementations that compile and pass baseline checks

**Status: COMPLETE** (closed 2026-03-03)

Exit Criteria:
- [x] `core/sdk-rust` compiles with domain trait definitions
- [x] `apps/cli` and `apps/server` have stub entry points
- [x] `cargo check` passes for the entire workspace
- [x] `dwf check:pr` passes (fmt, lint, build, test)
- [x] Documentation covers architecture, structure, workflow, and roadmap

Phase 2 Closure Tasks:
- [x] Land one minimal vertical slice (request -> provider stub -> output contract) through core + adapter + app surface.
- [x] Freeze and document v0.1.0-alpha.0 core request/response/error contract boundaries for migration batches.
- [x] Define explicit Phase 3 provider migration backlog (batch breakdown, dependency ordering, and risk notes).
- [x] Add a formal Phase 2 closure update in execution logs with phase gate decision and owners.

Phase 2 closure references:
- [Core Contract Boundaries (v0.1.0-alpha.0)](#kroki-rs-nxt.developer-guide.core-contracts)
- [Phase 3 Provider Migration Backlog](#kroki-rs-nxt.developer-guide.phase3-backlog)
- [v0.1.0 Execution Tracker](#kroki-rs-nxt.developer-guide.execution.v010)

### Phase 3: Feature Migration by Capability Slice (Current Phase)

**Goal**: Migrate kroki-rs functionality into the new architecture in prioritized batches.

**Status: IN PROGRESS** (started 2026-03-03)

Latest progress:
- Landed command-provider slices for Graphviz and D2 through core -> transport -> CLI/server.
- Added capability registry and provider metadata contract in `kroki-core`.
- Added server admin endpoints (`/health`, `/metrics`) and public playground route (`/playground`).
- Started Batch 3.2 groundwork with Mermaid provider registration and `native-browser` feature-gated runtime path.

Detailed batch plan:
- [Phase 3 Provider Migration Backlog](#kroki-rs-nxt.developer-guide.phase3-backlog)

| Batch | Scope | Key Components |
|-------|-------|----------------|
| 1 | Core Providers | Graphviz, D2 (CommandProvider pattern) |
| 2 | Browser Providers | Mermaid, BPMN (native-browser feature, CDP) |
| 3 | Pipeline + Plugins | Vega/VegaLite (multi-step), PluginProvider (subprocess) |
| 4 | Server Middleware | Auth, rate limiting, circuit breaker, metrics, Prometheus |
| 5 | CLI Features | Convert, batch, cache system, TUI upgrade (Ratatui) |

Per-batch requirements:
- Contract tests for each migrated provider
- Conformance tests verifying output parity with kroki-rs
- No regression in baseline latency or error budget

Exit Criteria:
- [ ] Feature parity target reached for v0.1.0 scope
- [ ] All provider conformance tests pass
- [ ] Migration dashboard shows parity status by provider

### Phase 4: Multi-Surface Expansion

**Goal**: Build additional interaction surfaces beyond CLI and Server.

Deliverables:
- **Desktop app** (`apps/desktop`): Tauri app with Rust backend + Lit frontend
- **Web dashboard** (`apps/web-app`): Lit + TypeScript web UI
- **VS Code extension** (`apps/vscode-ext`): TypeScript extension with diagram preview
- **MyST plugin** (`apps/myst-plugin`): TypeScript plugin surface for docs-native rendering workflows
- **TypeScript SDK** (`core/sdk-ts`): Wasm/FFI bindings exposing core logic to TS surfaces
- **Design system** (`shared/design-system`): Shared Lit components and CSS tokens

Exit Criteria:
- [ ] Desktop app renders diagrams via Tauri commands
- [ ] Web dashboard provides interactive diagram editing
- [ ] VS Code extension provides in-editor diagram preview
- [ ] MyST plugin provides rendering integration for MyST documentation workflows
- [ ] `core/sdk-ts` exposes key domain operations via Wasm

### Phase 5: Stabilization & Release

**Goal**: Harden, package, and release v0.1.0.

Deliverables:
- Performance tuning and cache observability
- CI quality gates via devflow
- Release packaging and automation
- Migration guide from kroki-rs to kroki-rs-nxt
- Contributor documentation

Exit Criteria:
- [ ] v0.1.0 RC passes all quality gates and release checklist
- [ ] Performance benchmarks meet or exceed kroki-rs baselines
- [ ] Public documentation is complete and reviewed

---

## devflow v0.3.0 Feedback Loop

Issues discovered during kroki-rs-nxt development that require devflow improvements:

| Issue | Impact | Target |
|-------|--------|--------|
| *(To be populated during development)* | | devflow v0.3.0 |

---

## Phase Gate Rules

- Each phase has explicit exit criteria listed above
- Phase gate failures trigger explicit decision review
- If exit criteria fail **twice consecutively**, open a decision review issue and adjust scope/sequence before continuing
- Review this roadmap at the end of each phase

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Over-abstraction in early architecture | Enforce phase gates; migrate by vertical slices, not abstract scaffolding |
| Tool lock-in before platform maturity | Keep devflow compatibility shim; use stable v0.2.0 command contracts |
| Over-engineering early layers | Start with stubs; add complexity only when tests demand it |
| Release burden across two active lines | Designate kroki-rs as maintenance-only during transition |
| Contributor confusion between repos | Publish explicit repo purpose and migration matrix |

## Timeline (Indicative)

- **Phase 2** (2-3 weeks): Bootstrap workspace and documentation
- **Phase 3** (4-6 weeks): Feature migration batches
- **Phase 4** (4-6 weeks): Multi-surface expansion
- **Phase 5** (2-3 weeks): Hardening and release
