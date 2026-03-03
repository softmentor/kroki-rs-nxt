---
title: Testing Strategy
label: kroki-rs-nxt.developer-guide.testing-strategy
---

# Testing Strategy

This guide explains how tests are organized, maintained, and run across the monorepo.

Related decision: [ADR-006](#kroki-rs-nxt.adr.0006)

## Goals

- Validate internal correctness and external API usability.
- Keep tests maintainable as the repository grows.
- Support fast PR feedback and deeper mainline confidence.

## Rust Test Organization

### 1) Unit Tests (`src/`)

Use `#[cfg(test)]` tests near implementation when testing private/internal behavior.

Examples:
- parsing helpers
- internal validation branches
- private utility edge cases

### 2) Integration/Public API Tests (`tests/`)

Use crate-level `tests/` to validate behavior as an external consumer.

Why this matters:
- verifies `pub` API is actually usable
- catches accidental visibility and API-boundary regressions

### 3) Binary Crate Pattern

For binary surfaces, keep `main.rs` thin and move testable logic to `src/lib.rs`.

Current pattern in this repo:
- `apps/cli/src/main.rs` calls into `apps/cli/src/lib.rs`
- `apps/server/src/main.rs` calls into `apps/server/src/lib.rs`
- integration tests live under `apps/*/tests/`

## Suggested Taxonomy

- **Unit**: fast internal correctness checks
- **Integration**: public API and cross-module behavior
- **Smoke**: service/process-level critical path checks
- **Load/Perf**: stress, throughput, latency envelopes

## Folder Conventions

Rust:
- Unit tests: in `src/` modules with `#[cfg(test)]`
- Integration tests: `<crate>/tests/*.rs`
- Test fixtures: `<crate>/tests/fixtures/*`
- Test resources/expected artifacts: `<crate>/tests/resources/*`
- Optional tier split: `<crate>/tests/smoke.rs`, `<crate>/tests/load.rs`

TypeScript:
- Unit/integration tests inside each package (co-located or package `tests/`)
- Keep package scripts consistent: `build`, `lint`, `test`

## Fixture, Config, and Resource Handling

### Fixtures (`tests/fixtures/`)

Use fixtures for:
- input payloads
- sample requests
- test-only configuration files (for example `server-test-config.toml`)

Guidelines:
- Keep fixtures small and focused on one behavior per file.
- Prefer plain-text and structured formats (`.txt`, `.json`, `.toml`, `.yaml`) over binary where possible.
- Name files by behavior, not by ticket id.

### Resources (`tests/resources/`)

Use resources for:
- expected outputs
- static assets needed by tests
- golden/reference files for snapshot-like checks

Guidelines:
- Keep expected output files deterministic.
- Version expected-output updates in the same PR as behavior changes.

### Path Resolution

In Rust tests, resolve fixture/resource paths using `env!(\"CARGO_MANIFEST_DIR\")`:

```rust
let path = std::path::PathBuf::from(env!(\"CARGO_MANIFEST_DIR\"))
    .join(\"tests\")
    .join(\"fixtures\")
    .join(\"example.txt\");
```

This avoids dependence on the process working directory and is CI-safe.

## Dependencies for Tests

Use `dev-dependencies` for test-only crates and tooling.

Typical examples:
- `tokio` test runtime helpers
- HTTP test clients for smoke checks
- assertion and fixture helpers

Avoid pulling test-only crates into production dependencies unless required.

## Commands and Workflows

### Local fast loop

```bash
dwf fmt:check
dwf lint:static
dwf build:debug
dwf test:unit
```

### PR verification (dwf 0.2.0)

```bash
dwf check:pr
```

### Per-crate verification

```bash
cargo test -p kroki-core
cargo test -p kroki-server
```

## CI Guidance

### PR lane (required)

- formatting
- lint/static checks
- build
- fast test tiers (unit + baseline integration)

### Main lane (broader)

- all PR checks
- wider integration set
- smoke checks

### Scheduled/manual lane

- load/performance checks
- long-running environment-dependent tests

## Maintenance Rules

- New public APIs must have integration tests in `tests/`.
- Critical bug fixes should include regression tests.
- Keep test names descriptive and stable.
- Keep heavy/integration tests out of default fast PR path unless critical.

## Smoke Script

Use the server smoke script to validate runtime critical paths:

```bash
scripts/smoke-server.sh
```

Current script checks:
- Admin endpoints: `/health`, `/metrics`
- Public capability endpoint: `/capabilities`
- Render path status behavior for `echo`, `graphviz`, `mermaid`, `bpmn`
- Tool-aware expectations for host dependencies (`dot`, `mmdc`)


## Current `dwf` behavior note

- `dwf check:pr` now includes both `test:unit` and `test:integration` in this repository target profile.
- Keep fast checks in unit tests and use integration tests for public API and fixture/resource validation.
