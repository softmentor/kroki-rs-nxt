---
title: Coding Conventions and Standards
label: kroki-rs-nxt.developer-guide.coding-conventions
---

# Coding Conventions and Standards

This page defines coding standards across stacks and the static analysis tools used to enforce them.

## Core Principles

- Preserve hexagonal boundaries: `apps -> adapters -> core`.
- Keep domain logic in `core/`; keep infrastructure logic in `adapters/`; keep composition and UX in `apps/`.
- Prefer small, testable units with explicit error handling.
- Update documentation when contracts, behavior, or structure changes.

## Repository Standards

### Architecture and Boundaries

- Core must not depend on adapters or apps.
- Adapters must not depend on apps.
- Shared Rust/TypeScript domain logic should flow through `core/sdk-ts`.
- Environment-specific configuration stays local to each surface package.

### Naming and Structure

- Rust crates: `kroki-<qualifier>`.
- TypeScript packages: `@kroki/<name>`.
- Keep files and modules aligned with domain responsibilities and migration slices.

### Testing Baseline

- Rust unit tests in-module and integration tests under `<crate>/tests/`.
- Keep test inputs in `<crate>/tests/fixtures/` and expected/static artifacts in `<crate>/tests/resources/`.
- TypeScript tests colocated or under package test folders.
- Contract tests for providers and adapter conformance where applicable.

## Static Analysis and Quality Tools

### 1) Rust Stack

Primary tools:
- `rustfmt` for formatting
- `clippy` for linting and correctness checks
- `cargo test` for unit/integration test validation

Canonical commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### 2) TypeScript/Node Stack

Primary tools:
- `eslint` for linting
- `tsc` for type-checking
- `prettier` for formatting

Canonical commands (through devflow where available):

```bash
dwf lint:static
```

Direct package-level commands (when package scaffolds are active):

```bash
pnpm lint
pnpm test
```

### 3) Workspace-Level Orchestration

`devflow` (`dwf`) is the canonical cross-stack command surface.

Recommended verification sequence:

```bash
dwf fmt:check
dwf lint:static
dwf build:debug
dwf test:unit
dwf check:pr
```

## Pull Request Standards

Before opening a PR:

1. Ensure branch is based on `dev`.
2. Run `dwf check:pr` and resolve failures.
3. Confirm docs are updated for any behavior, structure, or contract changes.
4. Add or update tests for the impacted scope.

## Exceptions and Escalation

- If a rule conflicts with a required migration step, document the exception in the PR and link the relevant roadmap/execution item.
- For architecture-impacting changes, create or update an ADR before merge.


## Lockfile Policy

- Commit `Cargo.lock` for deterministic Rust app/workspace builds.
- Commit `package-lock.json` for deterministic Node workspace installs.
- Update lockfiles in the same PR when dependency changes are introduced.
