---
title: Web App
label: kroki-rs-nxt.user-guide.web-app
---

# Web App

This page explains how to run the `@kroki/web-app` surface locally in:

- host-native mode
- host-container mode
- local development environment setup

Current state (as of **2026-03-04**):
- the web app is a frontend bootstrap using the shared `sdk-ts` packages
- default render path uses `RuntimeBridgeStub` (local stub runtime) for fast UI iteration
- server-backed runtime wiring is tracked as a follow-up increment

## Local Environment Setup

Prerequisites:

| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 20+ | JS runtime |
| pnpm | 9.x | Workspace package manager |
| Git | latest | Source checkout |

Install dependencies:

```bash
pnpm install
```

Optional validation:

```bash
pnpm -r --filter @kroki/runtime-wasm --filter @kroki/ui-tokens --filter @kroki/host-adapters --filter @kroki/app-playground --filter @kroki/web-app lint
pnpm -r --filter @kroki/runtime-wasm --filter @kroki/ui-tokens --filter @kroki/host-adapters --filter @kroki/app-playground --filter @kroki/web-app test
```

## Run Web App (Host-Native)

Start the dev server directly on your machine:

```bash
pnpm --filter @kroki/web-app dev
```

Open:
- [http://127.0.0.1:5173](http://127.0.0.1:5173)

Build production bundle:

```bash
pnpm --filter @kroki/web-app build
```

Preview production bundle locally:

```bash
pnpm --filter @kroki/web-app preview
```

Open:
- [http://127.0.0.1:4173](http://127.0.0.1:4173)

## Run Web App (Host-Container)

Quick-start using repository helper script:

```bash
./scripts/run-web-app-container.sh
```

Open:
- [http://127.0.0.1:5173](http://127.0.0.1:5173)

Common options:

```bash
# Docker engine instead of Podman
ENGINE=docker ./scripts/run-web-app-container.sh

# Custom web port
WEB_PORT=5174 ./scripts/run-web-app-container.sh

# Detached mode
DETACH=1 ./scripts/run-web-app-container.sh
```

You can also run directly with raw container commands:

Podman:

```bash
podman run --rm -it \
  -p 5173:5173 \
  -v "$PWD":/workspace \
  -w /workspace \
  docker.io/library/node:20-bookworm \
  bash -lc "npm i -g pnpm@9.15.0 && pnpm install && pnpm --filter @kroki/web-app dev --host 0.0.0.0 --port 5173"
```

Docker:

```bash
docker run --rm -it \
  -p 5173:5173 \
  -v "$PWD":/workspace \
  -w /workspace \
  node:20-bookworm \
  bash -lc "npm i -g pnpm@9.15.0 && pnpm install && pnpm --filter @kroki/web-app dev --host 0.0.0.0 --port 5173"
```

Open:
- [http://127.0.0.1:5173](http://127.0.0.1:5173)

## Local Integration with kroki-server

If you want server endpoints available at the same time (for future server-backed web runtime wiring), run server in parallel:

```bash
kroki-server --mode dev --port 8000 --admin-port 8081
```

Server URLs:
- public API: [http://127.0.0.1:8000](http://127.0.0.1:8000)
- admin health: [http://127.0.0.1:8081/health](http://127.0.0.1:8081/health)
- admin metrics: [http://127.0.0.1:8081/metrics](http://127.0.0.1:8081/metrics)

## Troubleshooting

`pnpm: command not found`:

```bash
npm install -g pnpm@9.15.0
```

Port already in use:

- change web dev port: `pnpm --filter @kroki/web-app dev --host 0.0.0.0 --port 5174`
- change preview port: `pnpm --filter @kroki/web-app preview --host 0.0.0.0 --port 4174`

Dependency drift after workspace updates:

```bash
pnpm install --frozen-lockfile
```
