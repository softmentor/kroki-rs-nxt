---
title: User Guide
label: kroki-rs-nxt.user-guide.user-index
---

# User Guide

Welcome to the kroki-rs-nxt user guide. This section covers everything you need to use the diagram platform effectively.

## Topics

- **[Features](#kroki-rs-nxt.user-guide.features)**: Multi-surface architecture, supported diagram types, and key capabilities.
- **[Configuration](#kroki-rs-nxt.user-guide.configuration)**: Runtime configuration (`kroki.toml`) and environment variable overrides.

## Quick Reference

### Surfaces

| Surface | Command | Description |
|---------|---------|-------------|
| **Server** | `cargo run -p kroki-server` | HTTP API on port 8000 |
| **CLI** | `cargo run -p kroki-cli` | Terminal interface |
| **Desktop** | `pnpm tauri dev` | Native desktop app (Phase 4) |
| **Web** | `pnpm --filter @kroki/web-app dev` | Web dashboard (Phase 4) |
| **VS Code** | Extension marketplace | Editor integration (Phase 4) |
| **MyST Plugin** | `pnpm --filter @kroki/myst-plugin dev` | Documentation integration surface (Planned) |

### Diagram Types

Graphviz, Mermaid, D2, BPMN, Vega, Vega-Lite, Wavedrom, Ditaa, Excalidraw, and custom plugins.

### Output Formats

SVG, PNG, WebP, PDF.
