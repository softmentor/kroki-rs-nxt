---
title: "ADR-006: Test Structure and Taxonomy"
label: kroki-rs-nxt.adr.0006
---

# ADR-006: Test Structure and Taxonomy for Multi-Surface Monorepo

## Status

**Accepted**

## Context

kroki-rs-nxt is a polyglot monorepo with multiple Rust crates and planned TypeScript surfaces. We need a testing structure that is maintainable over time, validates public APIs correctly, and keeps CI fast and predictable.

Key constraints:
- Library crates must verify their public API from outside the crate boundary.
- Binary-only crates are difficult to integration test unless logic is exposed through a library target.
- Large services need clear test tiers (unit, integration, smoke, load) with different CI policies.
- Some test dependencies are not appropriate as production dependencies.

## Decision

Adopt a two-level Rust testing model with explicit test taxonomy:

1. **Unit tests in `src/`** for private/internal behavior.
2. **Integration and public API tests in `tests/`** for crate-consumer behavior.

For binary crates (`apps/cli`, `apps/server`):
- Keep `main.rs` thin (bootstrap/wiring only).
- Move reusable logic and testable public surface to `src/lib.rs`.
- Place public behavior checks in `tests/`.

Fixture and resource policy:
- Use per-crate `tests/fixtures/` for structured test input and config examples.
- Use per-crate `tests/resources/` for expected-output artifacts and static assets.
- Resolve paths via `CARGO_MANIFEST_DIR` in tests to avoid brittle working-directory assumptions.
- Keep test-only config files (for example `.toml`) under `tests/fixtures/`, not under runtime config directories.

Test taxonomy:
- **Unit**: fast, internal logic checks
- **Integration**: public API and cross-module behavior
- **Smoke**: high-level service/path sanity checks
- **Load/Perf**: heavy, opt-in performance and stress validation

CI policy:
- PR gate runs fast tiers by default (fmt, lint, build, unit/integration baseline).
- Main branch runs broader verification.
- Load/perf tests run on scheduled/manual lanes.

## Consequences

**Positive:**
- Public API usability is continuously validated from external test context.
- Binary crate logic becomes reusable and testable.
- Test organization scales with repository growth and multi-surface expansion.
- CI can be optimized by tiering tests.
- Fixture and resource layout is predictable across crates.

**Negative:**
- Slightly more project structure overhead (`src/lib.rs` + `tests/`).
- Requires discipline to avoid mixing tiers.
- Some tests need dedicated dev dependencies and runtime setup.

**Mitigation:**
- Document conventions in Developer Guide testing strategy.
- Keep PR gate focused on fast tiers.
- Use clear naming and folder layout per tier.
