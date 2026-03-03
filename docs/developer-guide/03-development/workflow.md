---
title: Development Workflow
label: kroki-rs-nxt.developer-guide.workflow
---
# Development Workflow

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

# Build all Rust workspace members
dwf build:debug

# Run unit tests
dwf test:unit
```

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
dwf verify
```

This executes all checks in the `pr` target profile: `fmt:check`, `lint:static`, `build:debug`, `test:unit`.

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

1. **Define the provider** in `core/sdk-rust/src/providers/`:

```rust
// core/sdk-rust/src/providers/my_tool.rs
pub struct MyToolProvider { /* config fields */ }

#[async_trait]
impl DiagramProvider for MyToolProvider {
    fn validate(&self, source: &str) -> DiagramResult<()> { /* ... */ }
    async fn generate(&self, request: &DiagramRequest) -> DiagramResult<DiagramResponse> { /* ... */ }
    fn supported_formats(&self) -> &[OutputFormat] { /* ... */ }
}
```

2. **Register in DiagramRegistry** (in `core/sdk-rust/src/services/registry.rs`):

```rust
registry.register("my-tool", Arc::new(MyToolProvider::new(config)));
```

3. **Add tests** in `core/sdk-rust/tests/providers/`:
   - Unit test: validate + generate with mock input
   - Integration test: actual tool invocation (if tool is available)

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
RUST_LOG=devflow=debug dwf verify

# Check devflow config
dwf setup:doctor
```
