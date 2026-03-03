# kroki-rs-nxt

Next-generation, multi-surface diagram platform built with Rust.

## Overview

kroki-rs-nxt is a complete rewrite of [kroki-rs](https://github.com/softmentor/kroki-rs), designed from the ground up as a hexagonal architecture with multiple interaction surfaces:

- **CLI** - Interactive terminal UI (Ratatui TUI)
- **Server** - HTTP API (Axum)
- **Desktop** - Native app (Tauri + Lit)
- **VS Code** - Editor extension (TypeScript)
- **Web App** - Browser dashboard (Lit + TypeScript)

All surfaces share a single core domain engine that handles diagram generation, validation, caching, and plugin management.

## Architecture

```
apps/ (CLI, Server, Desktop, VS Code, Web)
  └── adapters/ (Storage, Transport)
        └── core/ (Domain Logic, SDKs, Plugins)
```

Follows **Hexagonal (Ports & Adapters)** pattern. See [docs/developer-guide/02-design/architecture.md](docs/developer-guide/02-design/architecture.md) for details.

## Project Status

**Phase 2: Bootstrap** - Scaffolding the monorepo structure and documentation. See [docs/developer-guide/04-roadmap/index.md](docs/developer-guide/04-roadmap/index.md) for the full migration plan.

## Getting Started

### Prerequisites

- Rust (stable, via rustup)
- Node.js 20+ and pnpm 9+
- [devflow](https://github.com/softmentor/devflow) v0.2.0+ (`dwf`)

### Build

```bash
# Verify toolchains
dwf setup

# Build all workspace members
dwf build:debug

# Run tests
dwf test:unit

# Full PR verification
dwf verify
```

### Project Commands

| Command | Description |
|---------|-------------|
| `dwf setup` | Verify toolchains and dependencies |
| `dwf fmt:check` | Check code formatting |
| `dwf lint:static` | Run static analysis |
| `dwf build:debug` | Debug build |
| `dwf test:unit` | Run unit tests |
| `dwf verify` | Full PR verification gate |

## Documentation

- [Architecture](docs/developer-guide/02-design/architecture.md) - Hexagonal architecture and domain model
- [Repository Structure](docs/developer-guide/02-design/repository-structure.md) - Folder layout and workspace membership
- [Build & Release](docs/developer-guide/03-development/build-and-release.md) - Build strategy and CI pipeline
- [Development Workflow](docs/developer-guide/03-development/workflow.md) - Getting started and daily workflow
- [Migration from kroki-rs](docs/developer-guide/04-roadmap/migration-from-kroki-rs.md) - Module mapping and changes
- [Roadmap](docs/developer-guide/04-roadmap/index.md) - Phased migration plan with gate criteria

## License

MIT - see [LICENSE](LICENSE)
