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
| `KROKI_<TOOL>_BIN` | Override binary path for a tool |
| `KROKI_<TOOL>_TIMEOUT` | Override timeout (ms) for a tool |
| `KROKI_<TOOL>_CONFIG` | Override config file path for a tool |
| `RUST_LOG` | Set log level (e.g., `info`, `debug`, `kroki_server=debug`) |

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
pr = ["fmt:check", "lint:static", "build:debug", "test:unit"]
main = ["fmt:check", "lint:static", "build:release", "test:unit", "test:integration", "test:smoke"]

[extensions.rust]
source = "builtin"
version = "^0.2"
api_version = 1
required = true

[extensions.node]
source = "builtin"
required = false
```

See [Build & Release](#kroki-rs-nxt.developer-guide.build-and-release) for full details on the devflow integration.
