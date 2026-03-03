---
title: References
label: kroki-rs-nxt.developer-guide.references
---

# References

This section captures supporting context and implementation references for kroki-rs-nxt.

## Reference Set

- [kroki-rs-nxt GitHub Repository](https://github.com/softmentor/kroki-rs-nxt)
- [kroki-rs (legacy baseline)](https://github.com/softmentor/kroki-rs)
- [devflow](https://github.com/softmentor/devflow)
- [Official Kroki](https://kroki.io)
- [Rust Language](https://www.rust-lang.org)

---

## Repository Architecture Guide (Detailed)

This section preserves the original architecture guide content for historical and planning continuity.

### Vision and Context

- **Multi-Surface Split:** A major architectural shift splitting core logic into backend libraries while keeping UX surfaces (CLI, Tauri app, VS Code extension, Web App) lightweight.
- **Workflow Orchestration:** `devflow` v0.2.0 is the standard workflow orchestrator. Any gaps or improvements discovered during the build of `kroki-rs-nxt` feed into devflow v0.3.0.
- **Legacy Migration:** `kroki-rs` is in maintenance mode (bug fixes only). `kroki-rs-nxt` is built as a new stack while leveraging kroki-rs domain foundations.
- **Single Monorepo:** kroki-rs-nxt is built as a single repository comprising both Rust and TypeScript ecosystems.

### Architecture Overview

The repository follows a **Hexagonal (Ports & Adapters)** pattern.
- **Core:** Contains pure business logic and defines interface traits.
- **Adapters:** Implements interfaces for specific technologies (e.g., SQLite, HTTP).
- **Apps:** Entry points that compose the Core and Adapters into runnable applications.

### Project Layout

```text
├── apps/                    # Executable Surfaces (Interactions)
│   ├── cli/                 # Rust (Ratatui TUI)
│   ├── desktop/             # Tauri App (Rust + Lit/TS)
│   │   ├── src-tauri/       # Tauri Rust Backend
│   │   └── src/             # Tauri Lit Frontend
│   ├── myst-plugin/         # MyST Plugin (TypeScript)
│   ├── server/              # Rust (Axum/Actix HTTP API)
│   ├── vscode-ext/          # VS Code Plugin (TypeScript)
│   └── web-app/             # Web Dashboard (Lit + TypeScript)
│
├── core/                    # Pure Domain Logic & SDKs
│   ├── sdk-rust/            # Primary Business Logic & Traits
│   ├── sdk-ts/              # Wasm/FFI Bindings for TS surfaces
│   └── plugins/             # Extensibility framework
│
├── adapters/                # Implementation of Core Traits
│   ├── storage/             # DB/File implementations
│   └── transport/           # HTTP/IPC handlers
│
├── shared/                  # Cross-stack resources
│   ├── design-system/       # Shared Lit components/CSS
│   └── scripts/             # Global CI/CD & Build scripts
│
├── Cargo.toml               # Root Rust Workspace
└── package.json             # Root NPM/pnpm Workspace
```

### Rules of Engagement

1. **Dependency Direction:** `apps` -> `adapters` -> `core`.
2. **Configuration:** Keep environment-specific configs (such as `tauri.conf.json` and `vite.config.ts`) inside the specific app folder.
3. **Shared Logic:** Any logic shared between Rust and TS must be exposed via `core/sdk-ts`.
4. **Testing:** Mirror `src` structure for unit tests and use `apps/<app>/tests` for integration/E2E tests.
5. **Workflow Orchestration:** Root orchestration uses `devflow` via `devflow.toml`.
