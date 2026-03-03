---
title: Repository Structure
label: kroki-rs-nxt.developer-guide.repository-structure
---

# Repository Structure Guide

## Directory Layout

```
kroki-rs-nxt/
│
├── apps/                           # Executable Surfaces (Interactions)
│   ├── cli/                        # Rust (Ratatui TUI)
│   │   ├── Cargo.toml              #   Package: kroki-cli
│   │   └── src/
│   │       └── main.rs             #   Binary entry point
│   ├── desktop/                    # Tauri App (Rust + Lit/TS)
│   │   ├── src-tauri/              #   Tauri Rust Backend (Cargo workspace member)
│   │   │   ├── Cargo.toml          #     Package: kroki-desktop
│   │   │   └── src/
│   │   └── src/                    #   Tauri Lit Frontend (pnpm workspace member)
│   │       └── package.json
│   ├── server/                     # Rust (Axum HTTP API)
│   │   ├── Cargo.toml              #   Package: kroki-server
│   │   └── src/
│   │       └── main.rs             #   Binary entry point
│   ├── vscode-ext/                 # VS Code Plugin (TypeScript)
│   │   └── package.json
│   └── web-app/                    # Web Dashboard (Lit + TypeScript)
│       └── package.json
│
├── core/                           # Pure Domain Logic & SDKs
│   ├── sdk-rust/                   # Primary Business Logic & Traits
│   │   ├── Cargo.toml              #   Package: kroki-core
│   │   └── src/
│   │       └── lib.rs
│   ├── sdk-ts/                     # Wasm/FFI Bindings for TS surfaces
│   │   └── package.json
│   └── plugins/                    # Extensibility framework
│       ├── Cargo.toml              #   Package: kroki-plugins
│       └── src/
│           └── lib.rs
│
├── adapters/                       # Implementation of Core Traits
│   ├── storage/                    # DB/File implementations
│   │   ├── Cargo.toml              #   Package: kroki-adapter-storage
│   │   └── src/
│   │       └── lib.rs
│   └── transport/                  # HTTP/IPC handlers
│       ├── Cargo.toml              #   Package: kroki-adapter-transport
│       └── src/
│           └── lib.rs
│
├── shared/                         # Cross-stack resources
│   ├── design-system/              # Shared Lit components/CSS
│   │   └── package.json
│   └── scripts/                    # Global CI/CD & Build scripts
│
├── docs/                           # MyST documentation
│   ├── myst.yml                    # MyST configuration
│   ├── toc.yml                     # Table of contents
│   ├── index.md                    # Landing page
│   └── ...
│
├── Cargo.toml                      # Root Rust Workspace
├── package.json                    # Root pnpm Workspace
├── pnpm-workspace.yaml             # pnpm workspace member list
├── devflow.toml                    # devflow workflow configuration
├── CLAUDE.md                       # Claude Code project instructions
├── LICENSE                         # MIT License
└── README.md                       # Project overview
```

---

## Workspace Membership

### Rust Workspace (`Cargo.toml`)

| Member Path | Package Name | Type | Layer |
|-------------|-------------|------|-------|
| `core/sdk-rust` | `kroki-core` | lib | Core |
| `core/plugins` | `kroki-plugins` | lib | Core |
| `adapters/storage` | `kroki-adapter-storage` | lib | Adapter |
| `adapters/transport` | `kroki-adapter-transport` | lib | Adapter |
| `apps/cli` | `kroki-cli` | bin | App |
| `apps/server` | `kroki-server` | bin | App |
| `apps/desktop/src-tauri` | `kroki-desktop` | bin | App (Phase 4) |

### pnpm Workspace (`pnpm-workspace.yaml`)

| Member Path | Package Name | Layer |
|-------------|-------------|-------|
| `core/sdk-ts` | `@kroki/sdk` | Core |
| `apps/desktop/src` | `@kroki/desktop-ui` | App |
| `apps/vscode-ext` | `@kroki/vscode` | App |
| `apps/web-app` | `@kroki/web-app` | App |
| `shared/design-system` | `@kroki/design-system` | Shared |

---

## Naming Conventions

### Rust Crates

Pattern: `kroki-<qualifier>`

| Crate | Description |
|-------|-------------|
| `kroki-core` | Core domain logic and traits |
| `kroki-plugins` | Plugin framework |
| `kroki-adapter-storage` | Storage adapter implementations |
| `kroki-adapter-transport` | Transport adapter implementations |
| `kroki-cli` | CLI binary |
| `kroki-server` | Server binary |
| `kroki-desktop` | Tauri desktop binary |

### TypeScript Packages

Pattern: `@kroki/<name>`

| Package | Description |
|---------|-------------|
| `@kroki/sdk` | Wasm bindings for core logic |
| `@kroki/desktop-ui` | Tauri Lit frontend |
| `@kroki/vscode` | VS Code extension |
| `@kroki/web-app` | Web dashboard |
| `@kroki/design-system` | Shared UI components |

---

## Folder Responsibilities

| Directory | Responsibility | Main Stack |
|-----------|---------------|------------|
| `apps/` | User-facing applications (CLI, Web, Desktop, VS Code, Server) | Polyglot |
| `core/` | Domain logic, business rules, and SDKs | Rust |
| `adapters/` | Infrastructure implementations (cache, HTTP, IPC) | Rust |
| `shared/` | Global assets, CI scripts, and design system | TS/CSS |
| `docs/` | Project documentation and architecture records | Markdown |

---

## Rules of Engagement

1. **Dependency Direction**: `apps` -> `adapters` -> `core`. Core must never depend on an App or Adapter.

2. **Configuration Locality**: Keep environment-specific configs (like `tauri.conf.json` or `vite.config.ts`) inside the specific app folder.

3. **Shared Logic**: Any logic shared between Rust and TS must be exposed via `core/sdk-ts` (generated via Wasm or FFI).

4. **Testing Strategy**:
   - Unit tests: in-module (`#[cfg(test)]` for Rust, co-located `.test.ts` for TS)
   - Integration tests: `<crate>/tests/` directory
   - E2E tests: `apps/<app>/tests/`
   - Contract tests: validate adapter implementations against core trait contracts
