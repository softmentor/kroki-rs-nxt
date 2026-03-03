# kroki-rs-nxt - Claude Code Project Configuration

## Project Overview

Polyglot monorepo for a multi-surface diagram generation platform. Hexagonal architecture (Ports & Adapters) with Rust core and TypeScript frontend surfaces.

## Key Architecture Rules

- **Dependency direction**: `apps → adapters → core` (NEVER the reverse)
- Core (`core/`) contains pure domain logic with zero infrastructure dependencies
- Adapters (`adapters/`) implement core traits for specific technologies
- Apps (`apps/`) compose core + adapters into runnable applications

## Workspace Structure

- **Rust workspace**: root `Cargo.toml` manages `core/sdk-rust`, `core/plugins`, `adapters/storage`, `adapters/transport`, `apps/cli`, `apps/server`
- **pnpm workspace**: `pnpm-workspace.yaml` manages `core/sdk-ts`, `apps/desktop/src`, `apps/vscode-ext`, `apps/web-app`, `shared/design-system`

## Crate Names

| Path | Package | Type |
|------|---------|------|
| `core/sdk-rust` | `kroki-core` | lib |
| `core/plugins` | `kroki-plugins` | lib |
| `adapters/storage` | `kroki-adapter-storage` | lib |
| `adapters/transport` | `kroki-adapter-transport` | lib |
| `apps/cli` | `kroki-cli` | bin |
| `apps/server` | `kroki-server` | bin |

## Build Commands

Use devflow for all build operations:
- `dwf fmt:check` / `dwf fmt:fix` - formatting
- `dwf lint:static` - clippy + eslint
- `dwf build:debug` / `dwf build:release` - build
- `dwf test:unit` - unit tests
- `dwf verify` - full PR gate

Or directly via Cargo:
- `cargo build --workspace` - build all Rust crates
- `cargo test --workspace` - test all Rust crates
- `cargo check -p <crate-name>` - check specific crate

## Configuration Files

- `devflow.toml` - build workflow orchestration (devflow CLI)
- `kroki.toml` - runtime configuration (diagram server/CLI)
- `Cargo.toml` (root) - Rust workspace definition
- `package.json` (root) - pnpm workspace definition

## Testing Strategy

- Unit tests: in-module `#[cfg(test)]` blocks
- Integration tests: `<crate>/tests/` directory
- E2E tests: `apps/<app>/tests/`
- Contract tests: validate adapter implementations against core traits

## Key Domain Types

- `DiagramProvider` trait in `core/sdk-rust` - central abstraction for diagram generation
- `DiagramRegistry` - provider registration and lookup
- `DiagramRequest` / `DiagramResponse` - domain DTOs
- `DiagramError` - typed error hierarchy
