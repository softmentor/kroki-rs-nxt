---
title: Getting Started
label: kroki-rs-nxt.user-guide.getting-started
---

# Getting Started

Get up and running with kroki-rs-nxt in minutes.

## Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Graphviz (`dot`) | latest | Graphviz provider |
| D2 (`d2`) | latest | D2 provider |
| Mermaid CLI (`mmdc`) | latest | Mermaid provider (CLI fallback path) |
| Ditaa (`ditaa`) | latest | Ditaa provider |
| Excalidraw (`excalidraw`) | latest | Excalidraw provider |
| Wavedrom CLI (`wavedrom-cli`) | latest | Wavedrom provider |
| Vega CLI (`vg2svg`) | latest | Vega provider |
| Vega-Lite CLI (`vl2vg`) | latest | Vega-Lite provider (also requires `vg2svg`) |
| Chromium / Chrome | latest | Browser providers (Mermaid CDP, BPMN) |

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

### Convert

Run a conversion:

```bash
kroki convert --diagram-type graphviz --source 'digraph G { A -> B; }'
kroki convert --diagram-type d2 --source 'a -> b'
kroki convert --diagram-type mermaid --source 'graph TD; A-->B;'
kroki convert --diagram-type vegalite --input chart.vl.json
```

Input file mode with auto-detection (works for all supported diagrams):

```bash
kroki convert --input ./sample.dot          # auto-detects graphviz from .dot
kroki convert --input ./diagram.mmd         # auto-detects mermaid from .mmd
kroki convert --input ./process.bpmn        # auto-detects bpmn from .bpmn
```

Output format selection:

```bash
kroki convert -t graphviz -s 'digraph G { A -> B; }' -f png -o graph.png
kroki convert -t d2 -s 'a -> b' -f webp -o diagram.webp
```

Stdin/stdout piping:

```bash
echo 'digraph G { A -> B; }' | kroki convert -t graphviz -o -
cat diagram.dot | kroki convert -t graphviz -f png > output.png
```

Output behavior:
- If `--output` is provided, output is written there. Use `-` for stdout.
- If `--output` is omitted, CLI auto-writes to `./kroki-<diagram_type>-<timestamp>.<ext>` and logs the file path.

Supported file extensions for auto-detection: `.dot`, `.gv` (graphviz), `.mmd`, `.mermaid` (mermaid), `.d2`, `.puml`, `.plantuml`, `.excalidraw`, `.bpmn`, `.vega`, `.vl` (vegalite), `.ditaa`, `.wavedrom`.

If Graphviz (`dot`) is unavailable, CLI falls back to bootstrap `echo` provider.

### Encode and Decode

Encode text to deflate + base64 for use in GET URLs:

```bash
kroki encode 'digraph G { A -> B; }'
echo 'digraph G { A -> B; }' | kroki encode
```

Decode back to plain text:

```bash
kroki decode eNpLyUwvSizIUHBXKM8vyklRBABQ_gVo
```

### Version

```bash
kroki version
```

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
- provider selection (`graphviz`, `d2`, `mermaid`, `bpmn`, `ditaa`, `excalidraw`, `wavedrom`, `vega`, `vegalite`, `echo`)
- output format selection (SVG, PNG, WebP) and payload mode controls
- prefilled examples with one-click reset
- optional auto-render while editing
- dark/light theme switch

## Run Web App Surface

For the standalone frontend surface (`@kroki/web-app`) with host-native and host-container run steps, see:

- [Web App User Guide](#kroki-rs-nxt.user-guide.web-app)

Quick container start:

```bash
./scripts/run-web-app-container.sh
```

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

### Standard Kroki Endpoints (Wire-Compatible)

These endpoints match the original [Kroki API](https://docs.kroki.io/kroki/setup/http-clients/):

**POST raw text** (`/{type}/{format}`):

```bash
curl -sS \
  -H "content-type: text/plain" \
  -d 'digraph G { A -> B; }' \
  http://127.0.0.1:8000/graphviz/svg
```

**POST JSON** (`/`):

```bash
curl -sS \
  -H "content-type: application/json" \
  -d '{"diagram_type":"graphviz","output_format":"svg","diagram_source":"digraph G { A -> B; }"}' \
  http://127.0.0.1:8000/
```

**GET with encoded source** (`/{type}/{format}/{encoded}`):

```bash
# Encode source first: kroki encode < file.dot
ENCODED=$(printf 'digraph G { A -> B; }' | kroki encode)
curl -sS "http://127.0.0.1:8000/graphviz/svg/${ENCODED}"
```

**PNG output** (any endpoint):

```bash
curl -sS \
  -H "content-type: text/plain" \
  -d 'digraph G { A -> B; }' \
  http://127.0.0.1:8000/graphviz/png -o graph.png
```

### Legacy Render Endpoint

The `/render` endpoint uses a different JSON shape and is retained for backward compatibility:

```bash
curl -sS \
  -H "content-type: application/json" \
  -d '{"source":"digraph G { A -> B; }","diagram_type":"graphviz","output_format":"Svg","source_encoding":"plain"}' \
  http://127.0.0.1:8000/render | jq
```

### Provider Examples

D2:

```bash
curl -sS -H "content-type: text/plain" -d 'a -> b' http://127.0.0.1:8000/d2/svg
```

Mermaid:

```bash
curl -sS -H "content-type: text/plain" \
  -d 'graph TD; A-->B;' http://127.0.0.1:8000/mermaid/svg
```

BPMN:

```bash
curl -sS -H "content-type: text/plain" \
  -d @process.bpmn http://127.0.0.1:8000/bpmn/svg
```

Vega-Lite:

```bash
curl -sS -H "content-type: text/plain" \
  -d @chart.vl.json http://127.0.0.1:8000/vegalite/svg
```

If a provider's tool is not installed, use `echo` provider for testing:

```bash
curl -sS -H "content-type: text/plain" -d 'A -> B' http://127.0.0.1:8000/echo/svg
```

### Error Responses (RFC 7807)

All error responses use RFC 7807 Problem Details format (`application/problem+json`):

```json
{
  "type": "https://kroki.io/errors/validation_failed",
  "title": "Bad Request",
  "status": 400,
  "detail": "Validation failed: source must not be empty"
}
```

Error/status mapping:
- `400` — validation errors (`validation_failed`)
- `413` — input exceeds `max_input_size` (`payload_too_large`)
- `415` — unsupported output format (`unsupported_format`)
- `422` — tool process failed (`process_failed`)
- `500` — internal error (`internal_error`)
- `503` — required tool/runtime unavailable (`tool_not_found`), or circuit breaker open (`circuit_open`)
- `504` — execution timeout (`execution_timeout`)

## What's Next?

- **[Features](#kroki-rs-nxt.user-guide.features)**: Learn about the multi-surface architecture
- **[Configuration](#kroki-rs-nxt.user-guide.configuration)**: Customize runtime and build settings
- **[Developer Guide](#kroki-rs-nxt.developer-guide.index)**: Dive into the architecture and codebase
