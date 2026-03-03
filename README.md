# kroki-rs-nxt

Next-generation, multi-surface diagram platform built with Rust.

## Overview

kroki-rs-nxt is a complete rewrite of [kroki-rs](https://github.com/softmentor/kroki-rs), designed from the ground up as a hexagonal architecture with multiple interaction surfaces:

- **CLI** - Interactive terminal UI (Ratatui TUI)
- **Server** - HTTP API (Axum)
- **Desktop** - Native app (Tauri + Lit)
- **VS Code** - Editor extension (TypeScript)
- **Web App** - Browser dashboard (Lit + TypeScript)
- **MyST Plugin** - Documentation integration plugin (TypeScript)

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

# Verify language deps + host runtime deps (dot, d2, mmdc)
dwf setup:deps

# Optional: auto-install missing host runtime deps on host machine
KROKI_HOST_DEPS_MODE=install dwf setup:deps

# Build all workspace members
dwf build:debug

# Run tests
dwf test:unit

# Full PR verification
dwf check:pr

# Local CI parity (podman container path)
podman machine start
./scripts/ci-local-podman.sh
```

### Install CLI Binary

```bash
# Optional: install host runtime dependencies (graphviz, d2, node, mmdc, chromium)
./scripts/install-runtime-deps.sh

# Install from packaged dist binary (default source)
./install.sh

# Optional sources
INSTALL_SOURCE=build ./install.sh
INSTALL_SOURCE=url RELEASE_URL=<cli-url> RELEASE_SERVER_URL=<server-url> ./install.sh

kroki --help
kroki convert --help
kroki-server --help
kroki completions --shell zsh --output ~/.local/share/kroki/completions
```

CLI writes output to `--output <path>` when provided, otherwise auto-writes to
`./kroki-<diagram_type>-<timestamp>.<ext>`.

### Start Server

```bash
kroki-server --mode dev
# or
kroki-server --mode prod
# debug logs with file:line
kroki-server --mode dev --debug
```

Admin URLs (default dev bind):
- `http://127.0.0.1:8081/health`
- `http://127.0.0.1:8081/metrics`

### Run Server In Local Container (Podman/Docker)

Build and run with bundled runtime dependencies (`dot`, `d2`, `mmdc`, Chromium):

```bash
# Default engine: podman
./scripts/run-server-container.sh

# Optional: use docker
ENGINE=docker ./scripts/run-server-container.sh

# Optional: detached mode
DETACH=1 ./scripts/run-server-container.sh
```

Override host port mapping:

```bash
PUBLIC_PORT=9000 ADMIN_PORT=9001 ./scripts/run-server-container.sh
```

Then use:
- `http://127.0.0.1:<PUBLIC_PORT>/render`
- `http://127.0.0.1:<PUBLIC_PORT>/capabilities`
- `http://127.0.0.1:<ADMIN_PORT>/health`
- `http://127.0.0.1:<ADMIN_PORT>/metrics`

Detached container controls:

```bash
podman logs -f kroki-rs-nxt-server
podman stop kroki-rs-nxt-server
```

### Run Published Release Image

When release images are published, run them directly without local builds:

```bash
# Podman
podman run --rm \
  -p 8000:8000 \
  -p 8081:8081 \
  ghcr.io/softmentor/kroki-rs-nxt-server:<tag>

# Docker
docker run --rm \
  -p 8000:8000 \
  -p 8081:8081 \
  ghcr.io/softmentor/kroki-rs-nxt-server:<tag>
```

### Project Commands

| Command | Description |
|---------|-------------|
| `dwf setup` | Verify toolchains |
| `dwf setup:deps` | Sync Rust/Node deps and verify host runtime deps (`dot`, `d2`, `mmdc`) |
| `dwf setup:host-deps` | Verify only host runtime deps (`dot`, `d2`, `mmdc`) |
| `dwf fmt:check` | Check code formatting |
| `dwf lint:static` | Run static analysis |
| `dwf build:debug` | Debug build |
| `dwf test:unit` | Run unit tests |
| `dwf test:integration` | Run integration tests |
| `dwf check:pr` | Full PR verification gate |
| `dwf prune:cache --local` | Prune local cache/work directories |
| `dwf prune:cache --gh` | Prune GitHub Actions caches |
| `dwf prune:cache --all` | Prune local + GitHub Actions caches |
| `dwf setup:prune-runs` | Prune GitHub Actions workflow runs |
| `dwf setup:prune-containers` | Prune local podman/docker resources |
| `dwf setup:prune-deep` | Full cleanup (cache, runs, container, temp) |

## Documentation

- [Architecture](docs/developer-guide/02-design/architecture.md) - Hexagonal architecture and domain model
- [Repository Structure](docs/developer-guide/02-design/repository-structure.md) - Folder layout and workspace membership
- [Build & Release](docs/developer-guide/03-development/build-and-release.md) - Build strategy and CI pipeline
- [Development Workflow](docs/developer-guide/03-development/workflow.md) - Getting started and daily workflow
- [Migration from kroki-rs](docs/developer-guide/04-roadmap/migration-from-kroki-rs.md) - Module mapping and changes
- [Roadmap](docs/developer-guide/04-roadmap/index.md) - Phased migration plan with gate criteria
- [Changelog](CHANGELOG.md) - Release and notable change history

## License

MIT - see [LICENSE](LICENSE)
