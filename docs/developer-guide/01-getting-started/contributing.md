---
title: Contributing
label: kroki-rs-nxt.developer-guide.contributing
---

# Contributing

Thank you for your interest in contributing to kroki-rs-nxt.

## Branching Strategy

| Branch | Purpose |
|--------|---------|
| `main` | Stable release branch |
| `dev` | Integration branch for active development |
| `feature/<name>` | Feature branches (branch from `dev`) |
| `fix/<name>` | Bug fix branches (branch from `dev` or `main`) |
| `release/<version>` | Release preparation branches |

## Workflow

1. Fork or branch from `dev`
2. Make changes following the [coding conventions](#architecture-rules)
3. Run `dwf verify` to ensure all checks pass
4. Submit a pull request targeting `dev`

## Architecture Rules

- **Dependency direction**: `apps -> adapters -> core` (never the reverse)
- Core must have zero infrastructure dependencies
- Environment-specific configs stay in their app folder
- Shared Rust/TS logic goes through `core/sdk-ts` (Wasm)

## Quality Gates

Every PR must pass:

- `fmt:check` — code formatting
- `lint:static` — static analysis (clippy, eslint)
- `build:debug` — workspace compilation
- `test:unit` — unit tests

## Code Review

- All PRs require at least one review
- Architecture-impacting changes require an ADR
- New providers require contract tests
