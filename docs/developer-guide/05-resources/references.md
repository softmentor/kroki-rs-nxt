---
title: References
label: kroki-rs-nxt.developer-guide.references
---
# kroki-rs-nxt Repository Architecture Guide

This document defines the structure, standards, and initialization roadmap for the `kroki-rs-nxt` polyglot monorepo. It details the transition from `kroki-rs` to the next-generation architecture supporting Rust and TypeScript across multiple surfaces.

## 🚀 Vision and Context

- **Multi-Surface Split:** A major architectural shift splitting core logic into backend libraries while keeping UX surfaces (CLI, Tauri app, VS Code extension, Web App) lightweight.
- **Workflow Orchestration:** `devflow` v0.2.0 will be utilized as the standard workflow orchestrator. Any gaps or improvements discovered during the build of `kroki-rs-nxt` will be addressed in Devflow v0.3.0.
- **Legacy Migration:** `kroki-rs` will be placed in maintenance mode (bug fixes only). `kroki-rs-nxt` is built fresh as a new stack but will eventually leverage the base `kroki-rs` code for functionalities.
- **Single Monorepo:** To maximize development velocity and simplify governance initially, `kroki-rs-nxt` is built as a single repository comprising both Rust and TypeScript ecosystems.

## 🏗️ Architecture Overview

The repository follows a **Hexagonal (Ports & Adapters)** pattern.
- **Core:** Contains pure business logic and defines interface traits.
- **Adapters:** Implements interfaces for specific technologies (e.g., SQLite, HTTP).
- **Apps:** Entry points that compose the Core and Adapters into a runnable application.

## 📂 Project Layout

A flat, clearly bounded folder structure is enforced to keep separation of concerns strict:

```text
├── apps/                    # Executable Surfaces (Interactions)
│   ├── cli/                 # Rust (Ratatui TUI)
│   ├── desktop/             # Tauri App (Rust + Lit/TS)
│   │   ├── src-tauri/       # Tauri Rust Backend
│   │   └── src/             # Tauri Lit Frontend
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

### Folder Responsibilities

| Directory   | Responsibility                               | Main Stack |
| ----------- | -------------------------------------------- | ---------- |
| `apps/`     | User-facing applications (CLI, Web, Desktop) | Polyglot   |
| `core/`     | Domain logic, business rules, and SDKs       | Rust       |
| `adapters/` | Infrastructure implementations (DB, API clients) | Rust       |
| `shared/`   | Global assets, CI scripts, and design system | TS/CSS     |

## 🛠️ Build & Release Strategy

Each surface manages its own lifecycle but utilizes root-level orchestration via `devflow`:

### Rust Surfaces (`cli`, `server`, `desktop`)
Managed via **Cargo Workspaces**.
- **Build:** `devflow build` (wraps `cargo build -p <package_name>`)
- **Test:** `devflow test` (wraps `cargo test`)

### TypeScript Surfaces (`web-app`, `vscode-ext`, `shared/design-system`)
Managed via **NPM/pnpm Workspaces** or **Turborepo**.
- **Build:** `devflow build` (wraps `pnpm build` scoped via `--filter`)
- **Lint:** `devflow lint` (wraps `pnpm lint`)

### Desktop App (Tauri)
The Tauri app in `apps/desktop` bridges both worlds.
- The `src-tauri` folder is a member of the Rust workspace.
- The `src` folder is a member of the TS workspace.
- **Run:** `pnpm tauri dev` from the `apps/desktop` directory.

## ⚖️ Rules of Engagement

1. **Dependency Direction:** `apps` → `adapters` → `core`. Core must never depend on an App or Adapter.
2. **Configuration:** Keep environment-specific configs (like `tauri.conf.json` or `vite.config.ts`) inside the specific app folder.
3. **Shared Logic:** Any logic shared between Rust and TS must be exposed via the `core/sdk-ts` (generated via Wasm or FFI).
4. **Testing:** Mirror the `src` structure in `tests/` for unit tests; use `apps/<app>/tests` for integration/E2E tests.
5. **Workflow Orchestration:** Root orchestration must use `devflow`. Do not introduce ad-hoc bash scripts to the root where `devflow.toml` can handle the execution cleanly.

## 🗺️ Implementation Execution Plan (Phased Roadmap)

### Phase 0: Groundwork and Scaffolding
- Initialize `kroki-rs-nxt` repository with base Cargo and NPM workspace schemas.
- Set up `devflow.toml` at the root using `devflow -v 0.2.0`.
- Create base directories (`apps`, `core`, `adapters`, `shared`).

### Phase 1: Core Domain Migration (Vertical Slice)
- Extract foundational data structures and traits from legacy `kroki-rs`.
- Implement initial `core/sdk-rust` and minimal `adapters` to support it.
- Ensure end-to-end `devflow` pipeline validity (lint, fmt, build, test).

### Phase 2: CLI and Server Slices
- Re-implement baseline `kroki-cli` features inside `apps/cli`.
- Re-implement baseline server endpoints in `apps/server`.
- Validate Hexagonal layer boundaries through tests.

### Phase 3: Frontend SDK and Surfaces
- Configure `core/sdk-ts` with Wasm bindings wrapping the Rust SDK.
- Establish `shared/design-system`.
- Scaffold `apps/vscode-ext` and `apps/web-app` relying on `core/sdk-ts`.

### Phase 4: Observability, Packaging, and Release
- Introduce deployment packaging via Devflow extension configurations if needed.
- Monitor Devflow execution gaps and queue them for `devflow` v0.3.0.
- Publish `kroki-rs-nxt` preview release.
