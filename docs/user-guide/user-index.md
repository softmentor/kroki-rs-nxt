---
title: User Guide
label: kroki-rs-nxt.user-guide.user-index
---

# User Guide

Welcome to the kroki-rs-nxt user guide. This section covers everything you need to use the diagram platform effectively.

## Topics

- **[Features](#kroki-rs-nxt.user-guide.features)**: Multi-surface architecture, supported diagram types, and key capabilities.
- **[Configuration](#kroki-rs-nxt.user-guide.configuration)**: Runtime configuration (`kroki.toml`) and environment variable overrides.
- **[Web App](#kroki-rs-nxt.user-guide.web-app)**: Run `@kroki/web-app` in host-native and host-container modes.

## Quick Reference

### Surfaces

| Surface | Command | Description |
|---------|---------|-------------|
| **Server** | `kroki-server --mode dev` | HTTP API with `/render`, `/capabilities`, `/playground` |
| **Server Admin** | `kroki-server --mode dev --admin-port 8081` | Admin URLs: `/health`, `/metrics` |
| **Server (Container)** | `./scripts/run-server-container.sh` | Local container with Graphviz, D2, Mermaid CLI, Chromium |
| **CLI** | `kroki convert --help` | Binary-first CLI usage with completions support |
| **Desktop** | `pnpm tauri dev` | Native desktop app (Phase 4) |
| **Web (Host)** | `pnpm --filter @kroki/web-app dev` | Web app on local host |
| **Web (Container)** | `./scripts/run-web-app-container.sh` | Web app in Podman/Docker container |
| **VS Code** | Extension marketplace | Editor integration (Phase 4) |
| **MyST Plugin** | `pnpm --filter @kroki/myst-plugin dev` | Documentation integration surface (Planned) |

### Diagram Types

Graphviz, Mermaid, D2, BPMN, Vega, Vega-Lite, Wavedrom, Ditaa, Excalidraw, and custom plugins.

### Output Formats

SVG, PNG, WebP, PDF.
