---
title: Developer Quickstart
label: kroki-rs-nxt.developer-guide.quickstart
---

# Developer Quickstart

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable (via rustup) | Core language |
| Node.js | 20+ | TypeScript surfaces |
| pnpm | 9+ | Package management |
| devflow (`dwf`) | v0.2.0+ | Workflow orchestration |
| Docker or Podman | Latest | Container execution (optional) |

## Setup

```bash
# Clone the repository
git clone https://github.com/softmentor/kroki-rs-nxt.git
cd kroki-rs-nxt

# Verify toolchains
dwf setup

# Verify Rust/Node dependencies and host runtime tools
dwf setup:deps

# Optional: auto-install missing host runtime deps
KROKI_HOST_DEPS_MODE=install dwf setup:deps

# Build all Rust workspace members
dwf build:debug

# Run unit tests
dwf test:unit
```

## Development Loop

```bash
# Format code
dwf fmt:fix

# Run linter
dwf lint:static

# Run unit tests
dwf test:unit

# Build specific crate
cargo build -p kroki-core
cargo test -p kroki-core
```

## Before Submitting a PR

```bash
# Run full PR verification gate
dwf check:pr
```

This runs all checks in the `pr` target: `fmt:check`, `lint:static`, `build:debug`, `test:unit`, `test:integration`.

## Containerized Local CI (Podman)

```bash
podman machine init   # first time only
podman machine start
./scripts/ci-local-podman.sh
```

Optional strict container-runtime check:

```bash
podman build -f Dockerfile.devflow -t kroki-rs-nxt-ci:latest .
mkdir -p .cache/devflow/node/npm .cache/devflow/rust/cargo .cache/devflow/rust/cargo/sccache .cache/devflow/rust/target
dwf --config devflow.container.toml check:pr
```

## Next Steps

- Read the [Architecture](#kroki-rs-nxt.developer-guide.architecture) to understand the hexagonal design
- See the [Development Workflow](#kroki-rs-nxt.developer-guide.workflow) for daily patterns
- Check the [Contributing Guide](#kroki-rs-nxt.developer-guide.contributing) for PR guidelines
