---
title: "ADR-003: devflow Integration"
label: kroki-rs-nxt.adr.0003
---

# ADR-003: devflow v0.2.0 as Workflow Orchestration from Day One

## Status

**Accepted**

## Context

kroki-rs used ad-hoc Makefile targets for build orchestration. As a polyglot monorepo, kroki-rs-nxt needs a unified command surface that works across Rust and TypeScript stacks with local/CI parity.

## Decision

Use **devflow v0.2.0** (`dwf`) as the workflow orchestration platform from the project's inception. Configure via `devflow.toml` with both Rust and Node extensions.

Canonical commands (`dwf fmt:check`, `dwf lint:static`, `dwf build:debug`, `dwf test:unit`, `dwf verify`) provide a stable developer interface regardless of underlying toolchain.

## Consequences

**Positive:**
- Local/CI parity via deterministic container execution
- Stable command contract for all contributors
- Auto-generated GitHub Actions workflows
- Feedback loop into devflow v0.3.0

**Negative:**
- Dependency on external tool (devflow)
- Contributors must install devflow
- Early adoption may surface devflow gaps

**Mitigation:** Direct Cargo/pnpm commands remain available as fallback.
