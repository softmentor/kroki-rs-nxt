---
title: "ADR-002: Monorepo Strategy"
label: kroki-rs-nxt.adr.0002
---

# ADR-002: Single Monorepo for All Surfaces

## Status

**Accepted**

## Context

kroki-rs-nxt will support multiple technology stacks (Rust + TypeScript) across multiple surfaces. The question is whether to use a single monorepo or split into multiple repositories.

## Decision

Start kroki-rs-nxt as a **single workspace monorepo**. Split only when ownership and release velocity evidence justifies it.

The monorepo uses dual workspace management:
- **Cargo Workspaces** for Rust crates
- **pnpm Workspaces** for TypeScript packages

## Consequences

**Positive:**
- Atomic cross-surface changes
- Single CI pipeline
- Simplified dependency management
- Easier contributor onboarding

**Negative:**
- Larger repository size over time
- CI times may grow with more surfaces
- Requires disciplined folder structure

**Mitigation:** Use devflow target profiles to scope CI checks by affected surface.
