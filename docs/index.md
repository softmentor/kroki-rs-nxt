---
title: Kroki-rs-nxt - Multi-Surface Diagram Platform
site:
  hide_outline: true
  hide_toc: true
  hide_title_block: true
label: kroki-rs-nxt.index
---

+++ { "kind": "split-image" }

Multi-Surface Architecture

# One Core. Many Surfaces.
# Native Rust Performance.

Kroki-rs-nxt is the next-generation diagram platform. A hexagonal architecture with a shared Rust core powering CLI, Server, Desktop, Web, and VS Code surfaces.

```{image} assets/images/k-rs-landing.jpeg
:class: only-light
```
```{image} assets/images/k-rs-landing.jpeg
:class: only-dark
```

{button}`Get Started <#kroki-rs-nxt.user-guide.getting-started>`

+++ { "kind": "justified" }

## The Vision

> **One diagram engine, many interaction surfaces. Write once in Rust, render everywhere — from terminal to browser to desktop.**

Managing diagram rendering across different environments and interfaces shouldn't require duplicating logic. We built **Kroki-rs-nxt** to solve this with a clean hexagonal architecture:

- **Core**: Pure Rust domain logic — providers, registry, validation, caching
- **Adapters**: Pluggable infrastructure — storage, HTTP transport, IPC
- **Surfaces**: CLI (Ratatui TUI), Server (Axum), Desktop (Tauri), Web (Lit), VS Code

---

## Structure of the Documentation

This guide covers everything from quick start to deep architecture:

- **[Get Started](#kroki-rs-nxt.user-guide.getting-started)**: Set up and run your first diagram generation.
- **[Features](#kroki-rs-nxt.user-guide.features)**: Discover the multi-surface architecture and capabilities.
- **[Architecture](#kroki-rs-nxt.developer-guide.architecture)**: Understand the hexagonal design and domain model.
- **[Developer Guide](#kroki-rs-nxt.developer-guide.index)**: Build, test, and contribute to the platform.
- **[Roadmap](#kroki-rs-nxt.developer-guide.roadmap.index)**: Track the phased migration plan.

> **Information is visual. Rendering should be effortless — on every surface.**

---
