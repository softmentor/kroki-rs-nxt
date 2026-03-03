---
title: Getting Started
label: kroki-rs-nxt.user-guide.getting-started
---

# Getting Started

Get up and running with kroki-rs-nxt in minutes.

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | stable (via rustup) | Core language |
| Node.js | 20+ | TypeScript surfaces |
| pnpm | 9+ | Package management |
| devflow (`dwf`) | v0.2.0+ | Workflow orchestration |

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/softmentor/kroki-rs-nxt.git
cd kroki-rs-nxt
```

### 2. Verify Toolchains

```bash
dwf setup
```

### 3. Build

```bash
dwf build:debug
```

### 4. Run Tests

```bash
dwf test:unit
```

### 5. Start the Server

```bash
cargo run -p kroki-server
```

The server starts on `http://localhost:8000`.

## What's Next?

- **[Features](#kroki-rs-nxt.user-guide.features)**: Learn about the multi-surface architecture
- **[Configuration](#kroki-rs-nxt.user-guide.configuration)**: Customize runtime and build settings
- **[Developer Guide](#kroki-rs-nxt.developer-guide.index)**: Dive into the architecture and codebase
