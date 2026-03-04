---
title: "ADR-007: Lit UI Platform with Vite Workspace and Rust Runtime Boundary"
label: kroki-rs-nxt.adr.0007
---

# ADR-007: Lit UI Platform with Vite Workspace and Rust Runtime Boundary

## Status

Accepted

## Context

Kroki-rs-nxt must support multiple presentation surfaces (web, desktop, future mobile/IDE) while preserving:

- consistent UX and visual identity
- shared component behavior
- Rust core parity for rendering and domain logic

Current risk without an explicit UI platform decision:

- duplicated UI implementation per surface
- style drift and behavior divergence
- weak contract boundaries between UI and Rust core
- expensive maintenance as surfaces grow

We need a design that is defensible under senior engineering review for:

- long-term maintainability
- architectural boundary clarity
- product velocity across many surfaces

## Decision

Adopt a **shared Lit UI platform** under `core/sdk-ts` with **pnpm + Vite** and an explicit **Rust runtime boundary**.

Adopt a **split state ownership model**:

- UI/application interaction state in TypeScript stores (reducer/event model)
- domain/render semantics and canonical error model in Rust contracts

### Chosen structure

Under `core/sdk-ts/packages`:

- `runtime-wasm`: typed boundary for Rust/WASM runtime contracts
- `ui-tokens`: theme and design-token source of truth
- `ui-components`: reusable host-agnostic Lit elements
- `app-playground`: composed editor experience
- `host-adapters`: host-specific integration adapters (`web`, `tauri`, future `vscode`, `mobile`)

Surface use:

- `apps/web-app` consumes `app-playground + host-adapters/web`
- `apps/desktop` consumes `app-playground + host-adapters/tauri`

### Non-goals

- No host-specific API access from reusable UI components.
- No migration of render-domain logic into TypeScript UI layer.
- No per-surface parallel UI stacks for equivalent functionality.
- No duplicate validation rules in UI when Rust already defines canonical behavior.

## Why This Is the Optimal Choice

### 1) Boundary integrity

Rust remains source of truth for rendering/domain logic. Lit remains source of truth for UI composition. The boundary is explicit and typed.

### 2) Reuse without lock-in

Lit components are standards-based web components and can run in web, webview, and tauri contexts without framework lock-in.

### 3) Fast iteration at scale

Vite + pnpm workspace gives fast feedback for UI development while preserving package isolation and reuse.

### 4) Controlled complexity

Host-specific concerns are isolated to adapters, reducing coupling and preventing cross-surface contamination.

## Alternatives Considered

### Alternative A: Build UI directly in each app surface

Rejected.

- Pros: Fast initial local delivery
- Cons: high drift risk, duplicate effort, weak consistency, costly later convergence

### Alternative B: Use a framework-specific shared app (single React/Vue app everywhere)

Rejected.

- Pros: large ecosystem
- Cons: stronger framework lock-in, less native fit for web-component reuse across heterogeneous hosts

### Alternative C: Move more core logic to TypeScript to simplify integration

Rejected.

- Pros: less bridge complexity initially
- Cons: long-term Rust/TS parity drift and duplicated domain behavior

## Consequences

### Positive

- Unified UI system across surfaces
- Stable runtime boundaries with Rust core
- Better long-term velocity and maintainability
- Clear ownership model (tokens/components/app composition/host adapters)

### Negative / Costs

- upfront package and contract design overhead
- stricter review discipline required for boundary hygiene
- additional CI/test matrix for shared packages
- state-model discipline required to prevent UI store sprawl

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Host APIs leak into shared UI components | Enforce adapter injection + lint constraints on imports |
| Token bypass creates visual drift | Token-only styling policy + component review checklist |
| Runtime contract churn | Version and freeze runtime contract at milestone gates |
| Over-abstraction slows delivery | Ship concrete web + desktop path first, generalize after validated reuse |
| UI/domain state duplication | Enforce split ownership and reject duplicated validation logic in reviews |

## State-Management Guardrails

1. UI stores hold interaction state only: layout, editor buffers, request lifecycle, diagnostics presentation.
2. Rust contracts hold canonical render semantics: validation, capability behavior, execution/error categories.
3. Render requests use monotonic revision IDs; stale responses must be discarded.
4. New render requests cancel prior in-flight requests when the host supports cancellation.
5. Persisted UI state must be schema-versioned and migration-safe.
6. Telemetry names for state transitions are shared across surfaces for parity checks.

## Engineering Review Position

For critical review challenge:

- This decision optimizes for **total system cost** over project lifetime, not local short-term coding speed.
- It creates explicit seams (runtime boundary + host adapter boundary) where complexity is expected.
- It preserves domain correctness by keeping Rust in control of render semantics.
- It reduces future re-platforming risk because the UI layer is standards-based and host-neutral.

## Implementation Guidance

1. Build tokens and base components first.
2. Deliver web playground as reference surface.
3. Reuse unchanged components in desktop tauri surface.
4. Add host adapters for IDE/mobile after web+desktop parity.
5. Track runtime contract changes through execution logs and milestone gates.
6. Add state-management conformance tests at `app-playground` and host-adapter boundaries.
