---
title: Development Workflow
label: kroki-rs-nxt.developer-guide.workflow
---
# Development Workflow

## Workflow Intent

This guide defines how to work in the repository today (Phase 2 bootstrap) while staying aligned with the target operating model for later phases.

Use this document for:
- local setup and validation
- daily development loop
- PR readiness
- troubleshooting and escalation paths

Devflow is the default orchestration layer for development lifecycle tasks (`setup`, `fmt`, `lint`, `build`, `test`, `ci`).
Shell scripts in `scripts/` should stay focused on user/runtime helpers (for example binary install or host dependency install), not replacing `dwf` lifecycle commands.

Host runtime dependencies (`dot`, `d2`, `mmdc`) are wired into `dwf setup:deps` through a repository path extension.
Chromium/Chrome install is optional and can be requested through install mode.
Default mode verifies required host tools and fails with remediation instructions when something is missing.
Use `KROKI_HOST_DEPS_MODE=install dwf setup:deps` to auto-install missing host deps via `scripts/install-runtime-deps.sh`.

---

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable (via rustup) | Core language |
| Node.js | 20+ | TypeScript surfaces |
| pnpm | 9+ | Package management |
| devflow (`dwf`) | v0.2.0+ | Workflow orchestration |
| Docker or Podman | Latest | Container execution (optional) |

## Getting Started

```bash
# Clone the repository
git clone https://github.com/softmentor/kroki-rs-nxt.git
cd kroki-rs-nxt

# Verify toolchains
dwf setup

# Verify language and host runtime dependencies
dwf setup:deps

# Build all Rust workspace members
dwf build:debug

# Run unit tests
dwf test:unit
```

Expected current behavior:
- Rust workspace crates should compile and test in bootstrap baseline mode.
- Some surfaces and provider paths are placeholders and will be implemented in later phases.

---

## Daily Development Workflow

### Before Starting Work

```bash
git checkout dev
git pull origin dev
git checkout -b feature/<your-feature>
```

### While Developing

```bash
# Format code
dwf fmt:fix

# Check formatting (non-destructive)
dwf fmt:check

# Run linter
dwf lint:static

# Run unit tests
dwf test:unit

# Build specific crate
cargo build -p kroki-core
cargo test -p kroki-core
```

### Before Submitting PR

```bash
# Run full PR verification gate
dwf check:pr
```

This executes all checks in the `pr` target profile: `fmt:check`, `lint:static`, `build:debug`, `test:unit`, `test:integration`.

Containerized parity check (Podman):

```bash
podman machine init   # first time only
podman machine start
./scripts/ci-local-podman.sh
```

PR checklist:
- Keep crate boundaries aligned with `apps -> adapters -> core`.
- Avoid introducing root-level ad-hoc scripts when `devflow.toml` can own the workflow.
- Update docs when architecture, commands, or structure change.
- Keep tests aligned with the repository test strategy in [Testing Strategy](#kroki-rs-nxt.developer-guide.testing-strategy).

---

## CI Workflow

CI workflows are auto-generated from `devflow.toml`:

```bash
# Generate GitHub Actions workflow
dwf ci:generate

# Validate committed workflow matches config
dwf ci:check
```

The generated workflow runs inside a fingerprinted CI container for reproducibility.

Current note:
- In this repository phase, CI workflow is intentionally customized for cache/runtime tuning.
- If `.github/workflows/ci.yml` is intentionally customized, `dwf ci:check` may fail with expected drift.

---

## Local CI Verification (Host + Container)

`devflow.toml` currently uses:

```toml
[runtime]
profile = "host"
```

That means `dwf ...` commands run on host toolchains by default.

Use this verification matrix before merge:

1. Host PR gate (fast, default):

```bash
dwf check:pr
```

2. Local container parity gate (recommended):

```bash
podman machine init   # first time only
podman machine start
./scripts/ci-local-podman.sh
```

3. Optional strict container-runtime `dwf` validation:

```bash
# Build expected image tag used by devflow container runtime
podman build -f Dockerfile.devflow -t kroki-rs-nxt-ci:latest .

# Ensure cache mount roots exist inside workspace
mkdir -p .cache/devflow/node/npm \
         .cache/devflow/rust/cargo \
         .cache/devflow/rust/cargo/sccache \
         .cache/devflow/rust/target
```

Create `devflow.container.toml` for container runtime checks:

```toml
[runtime]
profile = "container"

[container]
image = "kroki-rs-nxt-ci"
engine = "auto"

[extensions.host_deps]
source = "path"
path = "./tools/devflow-ext-host-deps.mjs"
required = false
trusted = true
```

Then run:

```bash
dwf --config devflow.container.toml check:pr
```

---

## Configuration Files

### Runtime Configuration (`kroki.toml`)

Controls the diagram server and CLI behavior at runtime:

```toml
[server]
port = 8000
admin_port = 8081
log_level = "info"
timeout_ms = 5000

[graphviz]
bin_path = "/usr/bin/dot"
timeout_ms = 5000

[mermaid]
timeout_ms = 10000
```

### Build Configuration (`devflow.toml`)

Controls the development workflow and CI pipeline. See `devflow.toml` in the repo root.

---

## Adding a New Diagram Provider

1. **Define the provider** in `core/sdk-rust/src/providers.rs`:

```rust
pub struct MyToolProvider { /* config fields */ }

#[async_trait]
impl DiagramProvider for MyToolProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> { /* ... */ }
    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> { /* ... */ }
    fn supported_formats(&self) -> &[OutputFormat] { /* ... */ }
}
```

2. **Register in DiagramRegistry** (in `core/sdk-rust/src/services.rs`):

```rust
registry.register("my-tool", Arc::new(MyToolProvider::new(config)));
```

3. **Add tests** in `core/sdk-rust/tests/`:
   - Unit test: validate + generate with mock input
   - Integration test: actual tool invocation (if tool is available)
   - Keep test fixtures/config in `tests/fixtures/` and expected artifacts in `tests/resources/`

4. **Update config** to include tool-specific settings in `kroki.toml`

---

## Adding a New App Surface

1. **Create the app directory** under `apps/<name>/`

2. **For Rust surfaces** (CLI, server):
   - Add `Cargo.toml` with dependencies on `kroki-core` and adapters
   - Add to root `Cargo.toml` workspace members
   - Create `src/main.rs` entry point

3. **For TypeScript surfaces** (web-app, vscode-ext):
   - Add `package.json` with `@kroki/sdk` dependency
   - Add to `pnpm-workspace.yaml`
   - Set up build tooling (Vite, esbuild, etc.)

4. **For Tauri surfaces** (desktop):
   - Add `src-tauri/` to Cargo workspace
   - Add `src/` to pnpm workspace
   - Configure `tauri.conf.json`

5. **Update devflow.toml** if the new surface requires additional CI targets

---

## Branching Strategy

| Branch | Purpose |
|--------|---------|
| `main` | Stable release branch |
| `dev` | Integration branch for active development |
| `feature/<name>` | Feature branches (branch from `dev`) |
| `fix/<name>` | Bug fix branches (branch from `dev` or `main`) |
| `release/<version>` | Release preparation branches |

---

## Troubleshooting

### Cargo workspace issues

```bash
# Clean all build artifacts
cargo clean

# Check specific package
cargo check -p kroki-core

# Verbose build output
cargo build -vv -p kroki-core
```

### devflow issues

```bash
# Debug output
RUST_LOG=devflow=debug dwf check:pr

# Check devflow config
dwf setup:doctor

# Validate only host runtime deps
dwf setup:host-deps

# Auto-install missing host runtime deps
KROKI_HOST_DEPS_MODE=install dwf setup:deps
```

### Local container parity issues

Image not found (`requested access to the resource is denied`):

```bash
podman build -f Dockerfile.devflow -t kroki-rs-nxt-ci:latest .
```

Untrusted path extension in container mode (`untrusted extension 'host_deps'`):

- In your container-profile config, set:
  - `[extensions.host_deps].required = false`
  - `[extensions.host_deps].trusted = true`

Podman mount failure on cache path (`statfs ... no such file or directory`):

```bash
mkdir -p .cache/devflow/node/npm \
         .cache/devflow/rust/cargo \
         .cache/devflow/rust/cargo/sccache \
         .cache/devflow/rust/target
```

Use workspace-relative cache roots for macOS/Podman compatibility:
- Prefer `.cache/devflow/...` under repo root.
- Avoid `/tmp/.cache/...` mounts for local parity runs.

### Cleanup and pruning

Devflow-native prune commands:

```bash
# prune local devflow caches/workdirs
dwf prune:cache --local

# prune GitHub Actions caches (requires gh auth)
dwf prune:cache --gh

# prune local + GH caches
dwf prune:cache --all

# prune GitHub Actions workflow runs
dwf setup:prune-runs

# prune local podman/docker resources
dwf setup:prune-containers

# deep cleanup across all scopes
dwf setup:prune-deep
```

Notes:
- `dwf prune:cache --local|--gh|--all` remains the primary cache cleanup path.
- `dwf setup:prune-runs`, `dwf setup:prune-deep`, and GH cache pruning require GitHub CLI auth (`gh auth status`) and `jq`.
- `dwf setup:prune-containers` and `dwf setup:prune-deep` use Podman if available, otherwise Docker.
- `dwf setup:prune-deep` additionally prunes known local temp bloat paths used by Chromium on macOS.
