---
title: Configuration
label: kroki-rs-nxt.user-guide.configuration
---

# Configuration

kroki-rs-nxt uses two configuration files with distinct purposes:

| File | Purpose | Used By |
|------|---------|---------|
| `kroki.toml` | Runtime behavior (server, tools, auth) | Server, CLI at runtime |
| `devflow.toml` | Build workflow (fmt, lint, test, CI) | Developers, CI |

---

## Runtime Status (March 3, 2026)

The `kroki.toml` model below is the **target configuration shape** for migration parity.
Current bootstrap runtime support in `kroki-server` is limited.

Implemented now:
- runtime config loading from `kroki.toml` (plus `KROKI_CONFIG` override)
- server bind and mode flags/env (`--mode`, `--host`, `--port`, `--bind`, admin bind variants)
- debug logging mode (`--debug`)
- admin endpoints (`/health`, `/metrics`)
- API key auth middleware (`server.auth`)
- token-bucket rate limiting middleware (`server.rate_limit`)
- provider-scoped circuit breaker manager (`server.circuit_breaker`)
- input/output size guardrails (`max_input_size`, `max_output_size`)
- provider tool discovery via PATH (`dot`, `d2`, `mmdc`)
- native browser backend using `headless_chrome` (CDP) for Mermaid in `native-browser` builds
- browser manager with pool-based concurrency, context TTL recycle, and adaptive failure recycle
- font URL handling for browser renderers with cache-backed `@font-face` harness injection

Planned (not fully wired yet in runtime middleware/config loader):
- per-key specific rate limits (currently global limiter)
- BPMN browser runtime implementation (provider currently returns explicit pending-runtime error)
- deeper config validation/error diagnostics for malformed runtime files

---

## Runtime Configuration (`kroki.toml`)

Controls the diagram server and CLI behavior.

```toml
[server]
port = 8000
admin_port = 8081
host = "localhost"
log_level = "info"
timeout_ms = 5000
max_input_size = 1048576      # 1MB
max_output_size = 52428800    # 50MB

[server.auth]
enabled = false
api_keys = [{ key = "your-api-key", label = "default", rate_limit = 10 }]
header_name = "X-API-Key"

[server.rate_limit]
enabled = true
requests_per_second = 100
burst_size = 10

[server.circuit_breaker]
enabled = true
failure_threshold = 5
reset_timeout_secs = 60

[server.metrics]
enabled = true
export_endpoint = true

[browser]
pool_size = 4
context_ttl_requests = 100

[graphviz]
bin_path = "/usr/bin/dot"
timeout_ms = 5000

[mermaid]
timeout_ms = 10000

[[plugins]]
name = "my-tool"
command = "my-tool-bin"
args = ["--format", "{format}"]
stdin = true
timeout_ms = 5000
formats = ["svg", "png"]
```

### Environment Variable Overrides

| Variable | Description |
|----------|-------------|
| `KROKI_SERVER_MODE` | Server mode (`dev` or `prod`) |
| `KROKI_SERVER_BIND` | Full bind address override (example: `0.0.0.0:8000`) |
| `KROKI_SERVER_HOST` | Host/IP override when `KROKI_SERVER_BIND` is unset |
| `KROKI_SERVER_PORT` | Port override when `KROKI_SERVER_BIND` is unset |
| `KROKI_SERVER_ADMIN_BIND` | Full admin bind address override (example: `0.0.0.0:8081`) |
| `KROKI_SERVER_ADMIN_HOST` | Admin host/IP override when `KROKI_SERVER_ADMIN_BIND` is unset |
| `KROKI_SERVER_ADMIN_PORT` | Admin port override when `KROKI_SERVER_ADMIN_BIND` is unset |
| `KROKI_BROWSER_POOL_SIZE` | Browser pool concurrency for native browser rendering |
| `KROKI_BROWSER_CONTEXT_TTL_REQUESTS` | Requests handled before browser recycle |
| `KROKI_BROWSER_ADAPTIVE_RECYCLE_FAILURES` | Consecutive browser failures before adaptive recycle |
| `KROKI_FONT_CACHE_DIR` | Directory used for cached font artifacts |
| `KROKI_<TOOL>_BIN` | Override binary path for a tool |
| `KROKI_<TOOL>_TIMEOUT` | Override timeout (ms) for a tool |
| `KROKI_<TOOL>_CONFIG` | Override config file path for a tool |
| `RUST_LOG` | Set log level (e.g., `info`, `debug`, `kroki_server=debug`) |

Server logging note:
- Use `kroki-server --debug` to force debug-level logs.
- Log output includes `date time`, `file`, and `line` for debugging traces.

---

## Build Configuration (`devflow.toml`)

Controls the development workflow via devflow (`dwf`).

```toml
[project]
name = "kroki-rs-nxt"
stack = ["rust", "node"]

[runtime]
profile = "host"

[container]
image = "kroki-rs-nxt-ci"
engine = "auto"

[targets]
pr = ["fmt:check", "lint:static", "build:debug", "test:unit", "test:integration"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "test:smoke"]
release = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "test:smoke", "package:artifact"]

[extensions.rust]
source = "builtin"
version = "^0.2"
api_version = 1
required = true

[extensions.node]
source = "builtin"
required = false

[extensions.host_deps]
source = "path"
path = "./tools/devflow-ext-host-deps.mjs"
required = true
```

See [Build & Release](#kroki-rs-nxt.developer-guide.build-and-release) for full details on the devflow integration.
