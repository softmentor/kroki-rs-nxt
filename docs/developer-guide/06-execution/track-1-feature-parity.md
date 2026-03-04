---
title: "Track 1: Feature Parity — kroki-rs-nxt vs kroki-rs"
label: kroki-rs-nxt.developer-guide.execution.track-1
---

# Track 1: Feature Parity — kroki-rs-nxt vs kroki-rs

## Purpose

Track feature parity of kroki-rs-nxt (v0.1.0-alpha.1) against kroki-rs (v0.0.8) and the original [Kroki](https://kroki.io) project. Identify gaps, partial implementations, and new improvements to retain.

Date baseline: **2026-03-04**.

---

## Current Strengths (Retain)

These are **new capabilities** in kroki-rs-nxt not present in kroki-rs. They represent architectural improvements and should be retained.

- [ ] Hexagonal architecture (core/adapters/apps separation)
- [ ] `CapabilityRegistry` + `/capabilities` endpoint
- [ ] Interactive `/playground` with live editor
- [ ] Shell completions (bash/zsh/fish/powershell)
- [ ] Adaptive browser recycling (failure-based, not just TTL)
- [ ] Mermaid dual-path (CDP primary + `mmdc` CLI fallback)
- [ ] Dev/Prod server modes (`--mode dev|prod`)
- [ ] Multi-surface architecture foundation
- [ ] Typed domain DTOs (`DiagramRequest`/`DiagramResponse`/`DiagramOptions`)
- [ ] `ProviderCategory` enum (Command, Browser, Pipeline, Plugin)

---

## API Compatibility with Original Kroki

kroki-rs-nxt must be **wire-compatible** with the original kroki API so existing tools and integrations work without changes.

Reference: [docs.kroki.io/kroki/setup/http-clients](https://docs.kroki.io/kroki/setup/http-clients/)

### Required Endpoints

| Endpoint | Method | Body | Status |
|----------|--------|------|--------|
| `/{type}/{format}` | POST | Raw text (`Content-Type: text/plain`) | - [ ] Gap |
| `/` | POST | JSON `{"diagram_type", "output_format", "diagram_source"}` | - [ ] Gap |
| `/{type}/{format}/{encoded}` | GET | Base64+Deflate encoded source in URL | - [ ] Gap |

### Current State

- `POST /render` exists but uses a different API shape — **deprecate** in favor of standard kroki endpoints
- Additional surface-specific endpoints (webapp, desktop) will be added later only when needed

---

## CLI Compatibility with Original Kroki CLI

Reference: [docs.kroki.io/kroki/setup/kroki-cli](https://docs.kroki.io/kroki/setup/kroki-cli/)

| Command | Description | Status |
|---------|-------------|--------|
| `convert <file>` | Transform diagram to output format | - [ ] Partial (needs `-o`, stdin, auto-detect) |
| `encode` | Encode text to deflate + base64 | - [ ] Gap |
| `decode` | Decode deflate + base64 to text | - [ ] Gap |
| `completion` | Shell autocompletion scripts | - [x] Parity |
| `version` | Show version info | - [ ] Gap |

### Required `convert` Options

- [ ] `-t/--type` with auto-detection from file extension
- [ ] `-f/--format` output format selection
- [ ] `-o/--out-file` with stdout support (`-`)
- [ ] Stdin input support (`-`)
- [ ] `-c/--config` config file path override
- [ ] `KROKI_ENDPOINT` env var support

### File Extension Auto-Detection

| Extension | Diagram Type |
|-----------|-------------|
| `.dot`, `.gv` | graphviz |
| `.mmd`, `.mermaid` | mermaid |
| `.d2` | d2 |
| `.puml`, `.plantuml` | plantuml |
| `.excalidraw` | excalidraw |
| `.bpmn` | bpmn |
| `.vega` | vega |
| `.vl`, `.vl.json` | vegalite |
| `.ditaa` | ditaa |
| `.wavedrom` | wavedrom |

---

## Diagram Providers

### Provider Status Matrix

| Provider | kroki-rs | kroki-rs-nxt | Pattern | Status |
|----------|----------|--------------|---------|--------|
| Graphviz | SVG, PNG, PDF | SVG only | CommandProvider | - [ ] Partial — wire PNG, PDF |
| D2 | SVG, PNG, PDF | SVG only | CommandProvider | - [ ] Partial — wire PNG, PDF |
| Mermaid | SVG, PNG, PDF (CDP) | SVG only (CDP + mmdc) | BrowserProvider | - [ ] Partial — wire PNG, PDF |
| BPMN | SVG, PNG, PDF (CDP) | Stub (returns error) | BrowserProvider | - [ ] Gap — wire runtime |
| Ditaa | PNG (JAR/exe) | Not implemented | CommandProvider | - [ ] Gap |
| Excalidraw | SVG | Not implemented | CommandProvider | - [ ] Gap |
| Wavedrom | SVG, PNG | Not implemented | CommandProvider | - [ ] Gap |
| Vega | SVG (`vg2svg`) | Not implemented | CommandProvider | - [ ] Gap |
| Vega-Lite | SVG (`vl2vg` + `vg2svg`) | Not implemented | PipelineProvider | - [ ] Gap |
| Echo | N/A | Testing stub | — | - [x] Test utility |

**Summary**: 3/9 production providers implemented, all SVG-only.

### Output Format Support

| Format | kroki-rs | kroki-rs-nxt | Status |
|--------|----------|--------------|--------|
| SVG | All providers | 3 providers | - [ ] Partial |
| PNG | 6 providers | Declared but not wired | - [ ] Gap |
| PDF | 4 providers | Declared but not wired | - [ ] Gap |
| WebP | Post-processing (SVG/PNG → WebP) | Deps present, not wired | - [ ] Gap |

`OutputFormat` enum declares SVG/PNG/WebP/PDF. `resvg` and `image` crate dependencies exist. Conversion logic needs wiring.

---

## Server Features

| Feature | kroki-rs | kroki-rs-nxt | Status |
|---------|----------|--------------|--------|
| POST render (raw text) | `/{type}/{format}` | `/render` (JSON) | - [ ] Different API — migrate |
| GET diagram (URL-encoded) | `/{type}/{format}/{encoded}` | N/A | - [ ] Gap |
| POST render (JSON) | N/A | `/render` | - [ ] Needs standard shape |
| Playground | N/A | `/playground` | - [x] New improvement |
| Capabilities | N/A | `/capabilities` JSON | - [x] New improvement |
| Health | JSON with pool health | JSON with status/version | - [ ] Partial (no pool health) |
| Metrics | Prometheus (8 types) | Prometheus (2 types) | - [ ] Partial |
| Admin dashboard | HTML UI | N/A | - [ ] Gap |
| Dual port | 8000 + 8081 | 8000 + 8081 | - [x] Parity |
| Dev/Prod modes | N/A | `--mode dev|prod` | - [x] New improvement |

---

## Middleware Stack

| Middleware | kroki-rs | kroki-rs-nxt | Status |
|-----------|----------|--------------|--------|
| Authentication (API key) | Header-based | Header-based | - [x] Parity |
| Admin auth (bcrypt) | Password-based | N/A | - [ ] Gap |
| Rate limiting | Token bucket per IP | Token bucket per IP | - [x] Parity |
| Circuit breaker | Per-provider, metrics | Per-provider, metrics | - [x] Parity |
| CORS | tower-http | tower-http | - [x] Parity |
| Input size validation | `max_input_size` | `max_input_size` | - [x] Parity |
| Output size validation | `max_output_size` | `max_output_size` | - [x] Parity |
| RFC 7807 errors | Problem Details JSON | Custom JSON | - [ ] Gap |

---

## CLI Features

| Feature | kroki-rs | kroki-rs-nxt | Status |
|---------|----------|--------------|--------|
| `convert` single file | Yes | Yes | - [x] Parity (core) |
| `-o/--out-file` | Yes | N/A | - [ ] Gap |
| Stdin/stdout piping | Yes | N/A | - [ ] Gap |
| `--font` flag | Yes | N/A | - [ ] Gap |
| `batch` directory | Parallel, auto-detect | Placeholder only | - [ ] Gap |
| `serve` | Start HTTP server | Separate binary | - [x] Architectural change |
| Shell completions | N/A | bash/zsh/fish/powershell | - [x] New improvement |
| Auto output naming | N/A | `kroki-{provider}-{ts}.{ext}` | - [x] New improvement |
| Cache support | `--cache-dir` SHA-256 | N/A | - [ ] Gap |

---

## Browser Pool

| Feature | kroki-rs | kroki-rs-nxt | Status |
|---------|----------|--------------|--------|
| Headless Chrome (CDP) | `headless_chrome` | `headless_chrome` | - [x] Parity |
| Pool size config | `KROKI_BROWSER_POOL_SIZE` | Same env var | - [x] Parity |
| Context TTL | Request-count recycling | Request-count recycling | - [x] Parity |
| Adaptive failure recycling | N/A | Failure-based recycling | - [x] New improvement |
| Semaphore concurrency | Yes | Yes | - [x] Parity |
| Font injection | CSS via `#kroki-fonts` | CSS via font manager | - [x] Parity |
| Health endpoint | `get_pool_health()` | `get_pool_health()` | - [x] Parity |

---

## Configuration

| Feature | kroki-rs | kroki-rs-nxt | Status |
|---------|----------|--------------|--------|
| `kroki.toml` | Full config model | Full config model | - [x] Parity |
| `KROKI_*` env overrides | ~15 variables | Similar coverage | - [x] Parity |
| Per-tool `bin_path` | `KROKI_{TOOL}_BIN` | N/A | - [ ] Gap |
| Per-tool `timeout_ms` | `KROKI_{TOOL}_TIMEOUT` | Request-level only | - [ ] Gap |
| Per-tool `config_path` | `KROKI_{TOOL}_CONFIG` | N/A | - [ ] Gap |
| Plugin config `[[plugins]]` | Array of plugin defs | N/A | - [ ] Gap |

---

## Cross-Cutting Features

| Feature | kroki-rs | kroki-rs-nxt | Status |
|---------|----------|--------------|--------|
| Adaptive timeout | Base + 1ms/10B, max 10s | Fixed per-request | - [ ] Gap |
| SHA-256 content caching | File-based cache | N/A (`adapters/storage` empty) | - [ ] Gap |
| Font manager | URL/file, SHA-256, 5MB max | URL/file, SHA-256, 10MB max | - [x] Parity+ |
| Image converter | SVG→WebP, PNG→WebP | Deps present, not wired | - [ ] Gap |
| Base64/Deflate decode | Standard + URL-Safe + Raw | Standard + URL-Safe + NoPad | - [x] Parity |
| File type auto-detection | Extension mapping (7+ types) | N/A (batch not done) | - [ ] Gap |

---

## Storage Adapter

| Feature | Status |
|---------|--------|
| Filesystem cache (SHA-256 keyed) | - [ ] Gap — `adapters/storage` empty |
| Repo-specific cache namespacing | - [ ] Gap — configurable at runtime |
| Cache middleware (check → generate → store) | - [ ] Gap |

Cache namespacing: Configurable at runtime via `kroki.toml` or env var, with a sensible default (project name from config, or hash of working directory). Prevents prune on one project from affecting another.

---

## Parity Summary

| Category | Score | Key Gaps |
|----------|-------|----------|
| Providers | 3/9 (33%) | Ditaa, Excalidraw, Wavedrom, Vega, VegaLite, BPMN runtime |
| Output Formats | SVG only (25%) | PNG, PDF, WebP not wired |
| Server API | ~70% | Standard kroki endpoints, admin dashboard, RFC 7807 |
| Middleware | ~85% | Admin auth, RFC 7807 errors |
| CLI | ~40% | encode/decode, batch, cache, font, stdin/stdout |
| Browser Pool | ~95% | Near parity with improvements |
| Configuration | ~60% | Per-tool config, plugin config |
| Cross-cutting | ~50% | Adaptive timeout, caching, image conversion |

**Overall Phase 3 completion: ~35%**

---

## Execution Order

| Priority | Batch | Description | Effort |
|----------|-------|-------------|--------|
| 1 | API compat | Standard kroki endpoints (POST raw, POST JSON, GET encoded) | M |
| 2 | CLI compat | encode, decode, version, stdin/stdout, auto-detect, `-o` | M |
| 3 | 3a | Output format wiring (PNG/WebP/PDF) via `resvg`/`image` | M |
| 4 | 3b | Command providers (Ditaa, Excalidraw, Wavedrom) | S each |
| 5 | 3c | Pipeline providers (Vega, Vega-Lite) | S, M |
| 6 | 3d | BPMN runtime wiring | M |
| 7 | Server | Admin dashboard, RFC 7807, richer metrics | M |
| 8 | CLI | Batch, font, cache, config flags | M |
| 9 | 3e | Plugin system (`core/plugins`) | L |
| 10 | Config | Per-tool config, adaptive timeout, cache layer | M |

**Effort key**: S = Small (1-2 days), M = Medium (3-5 days), L = Large (1-2 weeks)
