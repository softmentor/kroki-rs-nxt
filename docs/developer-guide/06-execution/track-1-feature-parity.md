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
| `/{type}/{format}` | POST | Raw text (`Content-Type: text/plain`) | - [x] Parity |
| `/` | POST | JSON `{"diagram_type", "output_format", "diagram_source"}` | - [x] Parity |
| `/{type}/{format}/{encoded}` | GET | Base64+Deflate encoded source in URL | - [x] Parity |

### Current State

- `POST /render` exists but uses a different API shape — **deprecate** in favor of standard kroki endpoints
- Additional surface-specific endpoints (webapp, desktop) will be added later only when needed

---

## CLI Compatibility with Original Kroki CLI

Reference: [docs.kroki.io/kroki/setup/kroki-cli](https://docs.kroki.io/kroki/setup/kroki-cli/)

| Command | Description | Status |
|---------|-------------|--------|
| `convert <file>` | Transform diagram to output format | - [x] Parity |
| `encode` | Encode text to deflate + base64 | - [x] Parity |
| `decode` | Decode deflate + base64 to text | - [x] Parity |
| `completion` | Shell autocompletion scripts | - [x] Parity |
| `version` | Show version info | - [x] Parity |

### Required `convert` Options

- [x] `-t/--type` with auto-detection from file extension
- [x] `-f/--format` output format selection
- [x] `-o/--out-file` with stdout support (`-`)
- [x] Stdin input support (`-`)
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
| Graphviz | SVG, PNG, PDF | SVG + PNG/WebP via resvg | CommandProvider | - [x] Parity (SVG+raster) |
| D2 | SVG, PNG, PDF | SVG + PNG/WebP via resvg | CommandProvider | - [x] Parity (SVG+raster) |
| Mermaid | SVG, PNG, PDF (CDP) | SVG + PNG/WebP (CDP + mmdc) | BrowserProvider | - [x] Parity (SVG+raster) |
| BPMN | SVG, PNG, PDF (CDP) | SVG (browser-backed) | BrowserProvider | - [x] Parity (SVG+raster) |
| Ditaa | PNG (JAR/exe) | PNG + SVG | CommandProvider | - [x] Parity |
| Excalidraw | SVG | SVG | CommandProvider | - [x] Parity |
| Wavedrom | SVG, PNG | SVG + PNG/WebP via resvg | CommandProvider | - [x] Parity |
| Vega | SVG (`vg2svg`) | SVG + PNG/WebP via resvg | PipelineProvider | - [x] Parity |
| Vega-Lite | SVG (`vl2vg` + `vg2svg`) | SVG + PNG/WebP via resvg | PipelineProvider | - [x] Parity |
| Echo | N/A | Testing stub | — | - [x] Test utility |

**Summary**: 9/9 production providers implemented. SVG primary; PNG/WebP via transport-layer resvg conversion.

### Output Format Support

| Format | kroki-rs | kroki-rs-nxt | Status |
|--------|----------|--------------|--------|
| SVG | All providers | All 9 providers | - [x] Parity |
| PNG | 6 providers | All providers via resvg conversion | - [x] Parity |
| PDF | 4 providers | Declared, not yet wired | - [ ] Gap |
| WebP | Post-processing (SVG/PNG → WebP) | All providers via resvg conversion | - [x] Parity |

`OutputFormat` enum declares SVG/PNG/WebP/PDF. SVG→PNG and SVG→WebP conversion implemented in `adapters/transport/src/conversion.rs` using `resvg` + `image` crates. PDF requires a dedicated crate (future work).

---

## Server Features

| Feature | kroki-rs | kroki-rs-nxt | Status |
|---------|----------|--------------|--------|
| POST render (raw text) | `/{type}/{format}` | `/render` (JSON) and `/{type}/{format}` | - [x] Parity |
| GET diagram (URL-encoded) | `/{type}/{format}/{encoded}` | `/{type}/{format}/{encoded}` | - [x] Parity |
| POST render (JSON) | N/A | `/` and `/render` | - [x] Parity |
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
| RFC 7807 errors | Problem Details JSON | `application/problem+json` | - [x] Parity |

---

## CLI Features

| Feature | kroki-rs | kroki-rs-nxt | Status |
|---------|----------|--------------|--------|
| `convert` single file | Yes | Yes | - [x] Parity |
| `-o/--out-file` | Yes | Yes (stdout via `-`) | - [x] Parity |
| Stdin/stdout piping | Yes | Yes | - [x] Parity |
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
| Image converter | SVG→WebP, PNG→WebP | SVG→PNG, SVG→WebP via resvg | - [x] Parity |
| Base64/Deflate decode | Standard + URL-Safe + Raw | Standard + URL-Safe + NoPad | - [x] Parity |
| File type auto-detection | Extension mapping (7+ types) | 10 extensions (.dot, .mmd, .d2, etc.) | - [x] Parity |

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
| Providers | 9/9 (100%) | All production providers implemented |
| Output Formats | SVG+PNG+WebP (75%) | PDF not yet wired |
| Server API | ~90% | Admin dashboard gap |
| Middleware | ~95% | Admin auth gap |
| CLI | ~75% | Batch, cache, font, config flags |
| Browser Pool | ~95% | Near parity with improvements |
| Configuration | ~60% | Per-tool config, plugin config |
| Cross-cutting | ~75% | Adaptive timeout, caching |

**Overall Phase 3 completion: ~80%**

---

## Execution Order

| Priority | Batch | Description | Effort |
|----------|-------|-------------|--------|
| 1 | API compat | Standard kroki endpoints (POST raw, POST JSON, GET encoded) | DONE |
| 2 | CLI compat | encode, decode, version, stdin/stdout, auto-detect, `-o` | DONE |
| 3 | 3a | Output format wiring (PNG/WebP) via `resvg`/`image` | DONE |
| 4 | 3b | Command providers (Ditaa, Excalidraw, Wavedrom) | DONE |
| 5 | 3c | Pipeline providers (Vega, Vega-Lite) | DONE |
| 6 | 3d | BPMN runtime wiring | DONE |
| 7 | Server | RFC 7807 error responses | DONE |
| 8 | Server | Admin dashboard, richer metrics | M |
| 9 | CLI | Batch, font, cache, config flags | M |
| 10 | 3e | Plugin system (`core/plugins`) | L |
| 11 | Config | Per-tool config, adaptive timeout, cache layer | M |

**Effort key**: S = Small (1-2 days), M = Medium (3-5 days), L = Large (1-2 weeks)
