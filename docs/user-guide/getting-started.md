---
title: Getting Started
label: kroki-rs-nxt.user-guide.getting-started
---

# Getting Started

Get up and running with kroki-rs-nxt in minutes.

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Graphviz (`dot`) | latest | Graphviz provider execution for CLI/server render |
| D2 (`d2`) | latest | D2 provider execution for CLI/server render |
| Mermaid CLI (`mmdc`) | latest | Mermaid browser-runtime rendering in CLI/server |

References:
- [Graphviz](https://graphviz.org/)
- [DOT language](https://graphviz.org/doc/info/lang.html)

Host dependency bootstrap helper:

```bash
./scripts/install-runtime-deps.sh
# optional (skip chromium)
./scripts/install-runtime-deps.sh --without-chromium
```

## Install Binary

You do not need to clone the repository to use the CLI binary.

Current install flow expects an already packaged local `dist/kroki` binary (for example from an extracted release bundle).

```bash
./install.sh
```

`install.sh` behavior:
- default source is local dist: `INSTALL_SOURCE=dist`
- future release URL source (for example GHCR-hosted release artifacts):
  - `INSTALL_SOURCE=url RELEASE_URL=<cli-url> RELEASE_SERVER_URL=<server-url>`
- optional local build fallback: `INSTALL_SOURCE=build`

Install location defaults to `~/.local/bin` (`PREFIX` can override). Installer places both:
- `kroki` (CLI)
- `kroki-server` (server)

## Shell Completion

`install.sh` also generates completion files in `~/.local/share/kroki/completions`.

Manual completion generation:

```bash
kroki completions --shell bash --output ~/.local/share/kroki/completions
kroki completions --shell zsh --output ~/.local/share/kroki/completions
kroki completions --shell fish --output ~/.local/share/kroki/completions
```

## CLI Usage

Inspect CLI usage/options/examples:

```bash
kroki --help
kroki convert --help
```

Run a conversion:

```bash
kroki convert --diagram-type graphviz --source 'digraph G { A -> B; }'
kroki convert --diagram-type d2 --source 'a -> b'
kroki convert --diagram-type mermaid --source 'graph TD; A-->B;'
```

Input file mode (works for all supported diagrams):

```bash
kroki convert --diagram-type graphviz --input ./sample.dot
```

Output behavior:
- If `--output` is provided, output is written there.
- If `--output` is omitted, CLI auto-writes to `./kroki-<diagram_type>-<timestamp>.<ext>` and logs the file path.

If Graphviz (`dot`) is unavailable, CLI falls back to bootstrap `echo` provider.

## Start Server

Development mode (localhost bind):

```bash
kroki-server --mode dev
```

Production mode (all-interface bind):

```bash
kroki-server --mode prod
```

Explicit host/port:

```bash
kroki-server --mode dev --host 127.0.0.1 --port 8000
```

Debug mode:

```bash
kroki-server --mode dev --debug
```

Admin surface override:

```bash
kroki-server --mode dev --admin-port 8081
```

The startup logs print surface version and listen ports.

## Run Server In Local Container

Use the bundled runtime image when you want all server dependencies preinstalled (`dot`, `d2`, `mmdc`, Chromium).

Run with Podman (default):

```bash
./scripts/run-server-container.sh
```

Run with Docker:

```bash
ENGINE=docker ./scripts/run-server-container.sh
```

Run detached:

```bash
DETACH=1 ./scripts/run-server-container.sh
```

Custom host ports:

```bash
PUBLIC_PORT=9000 ADMIN_PORT=9001 ./scripts/run-server-container.sh
```

Container URLs:
- public API: `http://127.0.0.1:<PUBLIC_PORT>`
- admin API: `http://127.0.0.1:<ADMIN_PORT>`

Quick checks:

```bash
curl -sS http://127.0.0.1:8000/capabilities | jq
curl -sS http://127.0.0.1:8081/health | jq
```

Detached container controls:

```bash
podman logs -f kroki-rs-nxt-server
podman stop kroki-rs-nxt-server
```

## Run Published Release Image

When release images are available, run directly from GHCR:

```bash
# Podman
podman run --rm \
  -p 8000:8000 \
  -p 8081:8081 \
  ghcr.io/softmentor/kroki-rs-nxt-server:<tag>

# Docker
docker run --rm \
  -p 8000:8000 \
  -p 8081:8081 \
  ghcr.io/softmentor/kroki-rs-nxt-server:<tag>
```

## Try Web Playground

Start server, then open:
- [http://127.0.0.1:8000/playground](http://127.0.0.1:8000/playground)

Playground supports:
- three-pane editor layout (examples sidebar, source editor, preview pane)
- provider selection (`graphviz`, `d2`, `mermaid`, `bpmn`, `echo`)
- output format and payload mode controls
- prefilled examples with one-click reset
- optional auto-render while editing
- dark/light theme switch

## Admin Endpoints

Default admin address is `http://127.0.0.1:8081` in dev mode.

Health:

```bash
curl -sS http://127.0.0.1:8081/health | jq
```

Metrics:

```bash
curl -sS http://127.0.0.1:8081/metrics | head -n 20
```

## Try Server Render API

In another terminal while server is running:

```bash
curl -sS http://127.0.0.1:8000/capabilities | jq
```

Render with plain payload:

```bash
curl -sS \
  -H "content-type: application/json" \
  -d '{"source":"digraph G { A -> B; }","diagram_type":"graphviz","output_format":"Svg","source_encoding":"plain"}' \
  http://127.0.0.1:8000/render | jq
```

Render with base64 payload (debug path):

```bash
PAYLOAD=$(printf 'digraph G { A -> B; }' | base64)
curl -sS \
  -H "content-type: application/json" \
  -d "{\"source\":\"\",\"source_encoded\":\"$PAYLOAD\",\"source_encoding\":\"base64\",\"diagram_type\":\"graphviz\",\"output_format\":\"Svg\"}" \
  http://127.0.0.1:8000/render | jq
```

Render with D2:

```bash
curl -sS \
  -H "content-type: application/json" \
  -d '{"source":"a -> b","diagram_type":"d2","output_format":"Svg","source_encoding":"plain"}' \
  http://127.0.0.1:8000/render | jq
```

Mermaid (feature-gated browser path):

```bash
curl -sS \
  -H "content-type: application/json" \
  -d '{"source":"graph TD; A-->B;","diagram_type":"mermaid","output_format":"Svg","source_encoding":"plain"}' \
  http://127.0.0.1:8000/render | jq
```

Note:
- Mermaid rendering requires `mmdc` on `PATH`.
- If `mmdc` is missing, server returns `503` with error code `tool_unavailable`.

BPMN baseline:
- `bpmn` provider is registered and status/error mapped.
- Runtime rendering path is pending; current server response is `500` with `internal_error`.

If Graphviz is not installed, use `echo` provider:

```bash
curl -sS \
  -H "content-type: application/json" \
  -d '{"source":"A -> B","diagram_type":"echo","output_format":"Svg","source_encoding":"plain"}' \
  http://127.0.0.1:8000/render | jq
```

Render error/status mapping (server `/render`):
- `400` validation errors
- `415` unsupported output format
- `503` required tool/runtime unavailable
- `504` execution timeout
- `422` tool process failed
- `500` internal/runtime not yet implemented

## What's Next?

- **[Features](#kroki-rs-nxt.user-guide.features)**: Learn about the multi-surface architecture
- **[Configuration](#kroki-rs-nxt.user-guide.configuration)**: Customize runtime and build settings
- **[Developer Guide](#kroki-rs-nxt.developer-guide.index)**: Dive into the architecture and codebase
