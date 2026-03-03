---
title: Repository Structure
label: kroki-rs-nxt.developer-guide.repository-structure
---

# Repository Structure Guide

This document describes the current monorepo layout and the intended expansion path. Where relevant, sections call out whether an item is active now or planned for later phases.

## Directory Layout

```
kroki-rs-nxt/
│
├── apps/                           # Executable surfaces
│   ├── cli/                        # Rust CLI (active)
│   ├── server/                     # Rust server (active)
│   ├── desktop/                    # Tauri app baseline package (planned feature expansion)
│   ├── myst-plugin/                # MyST plugin baseline package (planned feature expansion)
│   ├── vscode-ext/                 # VS Code extension baseline package (planned feature expansion)
│   └── web-app/                    # Web dashboard baseline package (planned feature expansion)
│
├── core/                           # Pure domain logic & SDKs
│   ├── sdk-rust/                   # Core business logic and ports
│   ├── plugins/                    # Plugin framework crate
│   └── sdk-ts/                     # TS/Wasm SDK baseline package (planned feature expansion)
│
├── adapters/                       # Infrastructure implementations
│   ├── storage/                    # Storage adapter crate
│   └── transport/                  # Transport adapter crate
│
├── shared/                         # Cross-stack resources
│   ├── design-system/              # Design system scaffold
│   └── scripts/                    # Shared scripts scaffold
│
├── docs/                           # MyST documentation
│   ├── myst.yml                    # MyST configuration
│   ├── toc.yml                     # Documentation table of contents
│   ├── user-guide/                 # User-facing documentation
│   ├── developer-guide/            # Developer-facing documentation
│   │   ├── 01-getting-started/
│   │   ├── 02-design/
│   │   ├── 03-development/
│   │   ├── 04-roadmap/
│   │   ├── 06-execution/
│   │   └── 10-resources/
│   └── reference.md
│
├── Cargo.toml                      # Root Rust workspace
├── package.json                    # Root Node workspace manifest
├── pnpm-workspace.yaml             # pnpm workspace member list
├── devflow.toml                    # devflow workflow configuration
├── CLAUDE.md                       # Project-specific assistant instructions
├── LICENSE                         # MIT license
└── README.md                       # Project overview
```

---

## Workspace Membership

### Rust Workspace (`Cargo.toml`)

| Member Path | Package Name | Status | Layer |
|-------------|-------------|--------|-------|
| `core/sdk-rust` | `kroki-core` | Active | Core |
| `core/plugins` | `kroki-plugins` | Active | Core |
| `adapters/storage` | `kroki-adapter-storage` | Active | Adapter |
| `adapters/transport` | `kroki-adapter-transport` | Active | Adapter |
| `apps/cli` | `kroki-cli` | Active | App |
| `apps/server` | `kroki-server` | Active | App |
| `apps/desktop/src-tauri` | `kroki-desktop` | Planned (commented in root workspace) | App |

### pnpm Workspace (`pnpm-workspace.yaml`)

| Member Path | Package Name | Status | Layer |
|-------------|-------------|--------|-------|
| `core/sdk-ts` | `@kroki/sdk` | Bootstrap baseline package | Core |
| `apps/desktop/src` | `@kroki/desktop-ui` | Bootstrap baseline package | App |
| `apps/myst-plugin` | `@kroki/myst-plugin` | Bootstrap baseline package | App |
| `apps/vscode-ext` | `@kroki/vscode` | Bootstrap baseline package | App |
| `apps/web-app` | `@kroki/web-app` | Bootstrap baseline package | App |
| `shared/design-system` | `@kroki/design-system` | Bootstrap baseline package | Shared |

---

## Naming Conventions

### Rust Crates

Pattern: `kroki-<qualifier>`

| Crate | Description |
|-------|-------------|
| `kroki-core` | Core domain logic and ports |
| `kroki-plugins` | Plugin framework |
| `kroki-adapter-storage` | Storage adapter implementations |
| `kroki-adapter-transport` | Transport adapter implementations |
| `kroki-cli` | CLI binary |
| `kroki-server` | Server binary |
| `kroki-desktop` | Tauri desktop binary (planned) |

### TypeScript Packages

Pattern: `@kroki/<name>`

| Package | Description |
|---------|-------------|
| `@kroki/sdk` | Wasm bindings for core logic |
| `@kroki/desktop-ui` | Tauri Lit frontend |
| `@kroki/myst-plugin` | MyST plugin integration surface |
| `@kroki/vscode` | VS Code extension |
| `@kroki/web-app` | Web dashboard |
| `@kroki/design-system` | Shared UI components |

---

## Folder Responsibilities

| Directory | Responsibility | Main Stack |
|-----------|---------------|------------|
| `apps/` | User-facing applications and interaction surfaces | Polyglot |
| `core/` | Domain logic, business rules, and SDK definitions | Rust |
| `adapters/` | Infrastructure implementations (cache, transport, IO boundaries) | Rust |
| `shared/` | Cross-surface assets, scripts, and design primitives | TS/CSS |
| `docs/` | Product, architecture, and execution documentation | Markdown |

---

## Rules of Engagement

1. **Dependency Direction**: `apps -> adapters -> core`.
2. **Configuration Locality**: Keep environment-specific config inside each app package.
3. **Shared Logic**: Rust/TS shared domain logic should be exposed through `core/sdk-ts`.
4. **Testing Strategy**:
   - Unit tests: in-module (`#[cfg(test)]` for Rust, co-located `.test.ts` for TS)
   - Integration tests: `<crate>/tests/`
   - E2E tests: `apps/<app>/tests/`
   - Contract tests: validate adapter implementations against core trait contracts
5. **Execution Tracking**: Track phase work and delivery status in `docs/developer-guide/06-execution/`.
