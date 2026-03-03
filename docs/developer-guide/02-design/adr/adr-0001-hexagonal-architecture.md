---
title: "ADR-001: Hexagonal Architecture"
label: kroki-rs-nxt.adr.0001
---

# ADR-001: Hexagonal Architecture with Apps/Adapters/Core Layering

## Status

**Accepted**

## Context

kroki-rs (v0.0.8) is a single-crate monolithic application. As the project evolves to support multiple interaction surfaces (CLI, server, desktop, web, VS Code), the monolithic structure creates coupling between domain logic and infrastructure concerns.

We need an architecture that:
- Separates pure business logic from infrastructure
- Allows multiple surfaces to share the same core engine
- Enforces clear dependency boundaries
- Enables independent testing of each layer

## Decision

Adopt the **Hexagonal Architecture** (Ports & Adapters) pattern with three layers:

- **Core** (`core/`): Pure domain logic and trait definitions (ports). Zero infrastructure dependencies.
- **Adapters** (`adapters/`): Concrete implementations of core traits for specific technologies (storage, HTTP, IPC).
- **Apps** (`apps/`): Entry points that compose Core and Adapters into runnable applications.

Dependency direction is strictly enforced: `apps → adapters → core`.

## Consequences

**Positive:**
- Clear separation of concerns enables independent surface development
- Core logic is testable without infrastructure
- New surfaces can be added without modifying core
- Enforced by Cargo workspace dependency graph

**Negative:**
- More crates to maintain than a single-crate project
- Cross-crate changes require coordinated updates
- Initial overhead of defining clean trait boundaries
