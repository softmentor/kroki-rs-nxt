---
title: "ADR-004: Wasm Bridge for TypeScript"
label: kroki-rs-nxt.adr.0004
---

# ADR-004: Wasm Bridge for Rust-to-TypeScript Shared Logic

## Status

**Accepted**

## Context

TypeScript surfaces (desktop frontend, web app, VS Code extension) need access to core domain logic (validation, format detection, diagram type registry). Reimplementing this logic in TypeScript would create drift and maintenance burden.

## Decision

Expose core Rust domain logic to TypeScript surfaces via **WebAssembly bindings** generated from `core/sdk-rust` into `core/sdk-ts` using `wasm-pack`.

## Consequences

**Positive:**
- Business logic written once in Rust, shared everywhere
- Type safety preserved through generated TypeScript bindings
- Performance benefits of Wasm for compute-heavy operations

**Negative:**
- Wasm bundle size adds to frontend payload
- Not all Rust APIs can be exposed through Wasm (e.g., filesystem, network)
- Build pipeline complexity (wasm-pack step)

**Mitigation:** Only expose pure domain logic through Wasm. Infrastructure operations (HTTP, filesystem) remain in native adapters.
