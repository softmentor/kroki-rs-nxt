---
title: Build & Release
label: kroki-rs-nxt.developer-guide.build-and-release
---
# Build & Release Strategy

## Overview

kroki-rs-nxt is a polyglot monorepo managed by two workspace systems:

- **Cargo Workspaces** for Rust crates (core, adapters, apps)
- **pnpm Workspaces** for TypeScript packages (web-app, vscode-ext, design-system, sdk-ts)

**devflow** (`dwf`) provides the unified command surface across both stacks.

## Current Phase Notes

As of Phase 2 bootstrap:
- Build and test topology is defined in `devflow.toml`.
- Several runtime surfaces are still bootstrap-baseline and not yet feature complete.
- Release process is documented as the target model and will be operationalized as implementation matures.

---

## devflow Integration

`devflow.toml` is the single source of truth for build workflow commands. All developers and CI use the same commands regardless of the underlying stack.

### Canonical Commands

| Command | What it does |
|---------|-------------|
| `dwf setup` | Verify toolchains, sync dependencies |
| `dwf setup:deps` | Sync language deps and verify host runtime deps (`dot`, `d2`, `mmdc`) |
| `dwf setup:host-deps` | Verify only host runtime deps (`dot`, `d2`, `mmdc`) |
| `dwf fmt:check` | Check formatting (rustfmt + prettier) |
| `dwf fmt:fix` | Auto-fix formatting |
| `dwf lint:static` | Static analysis (clippy + eslint + tsc) |
| `dwf build:debug` | Debug build for all workspace members |
| `dwf build:release` | Release build with optimizations |
| `dwf test:unit` | Run unit tests |
| `dwf test:integration` | Run integration tests |
| `dwf test:smoke` | Run smoke tests |
| `dwf check:pr` | Full PR verification gate (includes integration tests) |
| `dwf ci:generate` | Generate GitHub Actions workflow |
| `dwf ci:check` | Validate committed CI workflow |

Host runtime dependency install mode:

```bash
KROKI_HOST_DEPS_MODE=install dwf setup:deps
```

### Target Profiles

```toml
[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit", "test:integration"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "test:smoke"]
release = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "test:smoke", "package:artifact"]
```

---

## Rust Surfaces

Managed via **Cargo Workspaces** (root `Cargo.toml`).

### Build

```bash
# Build specific package
cargo build -p kroki-core
cargo build -p kroki-server

# Build all workspace members
cargo build --workspace

# Release build
cargo build --workspace --release
```

### Test

```bash
# Test specific package
cargo test -p kroki-core

# Test all
cargo test --workspace

# Integration tests only
cargo test --workspace --test '*'
```

### Lint

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

---

## TypeScript Surfaces

Managed via **pnpm Workspaces** (root `package.json` + `pnpm-workspace.yaml`).

### Build

```bash
# Build specific package
pnpm --filter @kroki/web-app build
pnpm --filter @kroki/design-system build

# Build all TS packages
pnpm build
```

### Lint

```bash
pnpm --filter @kroki/web-app lint
pnpm lint  # all packages
```

### Test

```bash
pnpm --filter @kroki/web-app test
pnpm test  # all packages
```

---

## Desktop App (Tauri)

The Tauri app in `apps/desktop` bridges both workspace systems:

- `apps/desktop/src-tauri/` is a member of the **Cargo workspace**
- `apps/desktop/src/` is a member of the **pnpm workspace**

### Development

```bash
# From apps/desktop
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

---

## CI Strategy

### Workflow Topology

Generated via `dwf ci:generate`:

```
prep ──► build ──┬──► fmt-check
                 ├──► lint
                 ├──► test-unit
                 └──► test-smoke
```

- **prep**: Resolve container fingerprint, restore image cache, emit outputs
- **build**: Compile workspace once, save cargo/sccache cache (runs inside CI container)
- **verify jobs**: Run in parallel, read-only cache, separate required checks

### Container Strategy

- **Fingerprinted CI images**: `ghcr.io/softmentor/kroki-rs-nxt-ci:<fingerprint>`
- **Fingerprint inputs**: `Dockerfile`, `Makefile`, `rust-toolchain.toml`
- **Cache root**: `.cache/devflow/` with subdirs for registry, git, sccache, target

### Cache Layers (ordered by ROI)

1. CI image reuse from GHCR by fingerprint
2. Image tar cache for same-runner restore
3. Cargo + sccache cache keyed by `runner.os + fingerprint + hash(Cargo.lock)`
4. BuildKit cache for image builds

### Local CI Parity Checklist

Use this ordered checklist before phase gates:

1. Verify workflow contract:

```bash
dwf ci:generate
dwf ci:check
```

2. Verify host gate:

```bash
dwf check:pr
```

3. Verify container parity gate:

```bash
podman machine start
./scripts/ci-local-podman.sh
```

4. Optional strict `dwf` container-runtime check:

```bash
podman build -f Dockerfile.devflow -t kroki-rs-nxt-ci:latest .
mkdir -p .cache/devflow/node/npm .cache/devflow/rust/cargo .cache/devflow/rust/cargo/sccache .cache/devflow/rust/target
dwf --config devflow.container.toml check:pr
```

Container-mode note:
- Default project config runs `dwf` in host mode.
- Keep a separate `devflow.container.toml` for parity-only validation to avoid disrupting daily host workflow.

### CI Drift Status (Current)

Current repository state keeps a **customized** `.github/workflows/ci.yml` for cache and runtime optimizations.

Implication:
- `dwf ci:check` reports workflow drift by design in this phase.

Why this happens:
- `dwf ci:check` enforces byte-level alignment with generated workflow structure.
- Local customizations (cache layout, bootstrap differences, command wrapping) are flagged as drift even when functionally valid.

Two operating modes:

1. Strict devflow compliance mode:
   - `dwf ci:generate --force`
   - keep generated workflow unmodified
   - `dwf ci:check` passes

2. Optimized custom workflow mode (current):
   - keep tuned workflow in `.github/workflows/ci.yml`
   - treat `dwf ci:check` failure as expected drift
   - validate behavior with host + container parity checks (`dwf check:pr`, `./scripts/ci-local-podman.sh`)

Legacy devflow examples were strict-compliance mode by default:
- generate workflow from `devflow.toml`
- avoid manual edits in workflow file
- push optimizations into devflow config/generator support where possible

### Test Tiers

| Tier | Trigger | Checks |
|------|---------|--------|
| **PR minimal gate** | Pull request | fmt, lint, build, test:unit, test:integration |
| **Full verification** | Push to main/dev | fmt, lint, build, test:unit, test:integration, test:smoke |
| **Release** | Tag | Full verification + package:artifact |

---

## Release Pipeline

This is the intended release flow for `v0.1.0` and later milestones.

1. `dwf check:pr` passes all PR gates
2. Version bump in workspace `Cargo.toml` and `package.json`
3. `dwf package:artifact` creates release binaries
4. Tag with `git tag v<version>`
5. CI builds release artifacts for target platforms
6. Publish to GitHub Releases

### Platform Targets

| Surface | Artifact | Distribution |
|---------|----------|-------------|
| Server | Linux binary (amd64, arm64) | Docker image, binary download |
| CLI | Multi-platform binary | Homebrew, binary download |
| Desktop | Platform installers (.dmg, .msi, .AppImage) | GitHub Releases |
| VS Code | .vsix package | VS Code Marketplace |
| Web App | Static site bundle | CDN / self-hosted |
