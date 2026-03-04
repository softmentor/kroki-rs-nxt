---
title: "Track 2: devflow v0.3.0 — Workflow Consistency & Optimization"
label: kroki-rs-nxt.developer-guide.execution.track-2
---

# Track 2: devflow v0.3.0 — Workflow Consistency & Optimization

## Purpose

Capture devflow v0.2.0 limitations discovered during kroki-rs-nxt development and define the improvement backlog for devflow v0.3.0. Focus on simplifying and driving consistency between host modes (native/container) and environments (local/GH Actions).

**v0.3.0 is a breaking change from v0.2.0.** Prominently documented in CHANGELOG.

Date baseline: **2026-03-04**.

---

## Design Principles

1. **devflow core is stack-agnostic** — zero knowledge of Rust, Node, Python
2. **Stack specifics in core trusted extensions** — ship with devflow binary, activated in `devflow.toml`
3. **Simplicity and consistency** — same workflow shape regardless of host mode or environment
4. **Optimization is first-class** — image fingerprinting, layer caching, incremental builds
5. **Repo-scoped isolation** — cache keys, image tags, artifacts include repo identity

---

## Terminology

| Term | Definition |
|------|-----------|
| **host:native** | Running directly on host without containers |
| **host:container** | Running inside a devflow-managed container |
| **env:local** | Execution on developer's local machine |
| **env:gh** | Execution on GitHub Actions runners |
| **stack-image** | OS basics + stack toolchains — cross-repo reusable |
| **project-image** | stack-image + project-specific system deps — per-project |
| **repo-image** | project-image + lockfile dependencies cached — per-repo |

### Host Mode × Environment Matrix

| | env:local | env:gh |
|---|----------|--------|
| **host:native** | Developer runs `dwf` directly | Future: stock GH ubuntu runner |
| **host:container** | Developer runs in podman/docker | GH Actions with container image |

**Future possibilities** (designed for, not implemented in v0.3.0):

- host:native + env:gh — stock ubuntu runners without containers (no consistency guarantee unless host OS matches)
- host:container + env:local via `act` — running GH Actions workflows locally

---

## Canonical Workflow

```
setup → build → verify → ship
```

| Stage | host:native | host:container |
|-------|-------------|----------------|
| **setup** | `setup:toolchain` + `setup:deps` | Build/restore repo-image (layered) |
| **build** | Compile on host | Compile inside repo-image |
| **verify** | Test on host | Test inside repo-image |
| **ship** | `ship:package` → `dist/` | `ship:package` (env:local) or `ship:release` (env:gh only) |

- `ship:release` is only available on env:gh (tagging, artifact publication)
- `ship:package` produces local artifacts in `dist/` for both environments

---

## Three-Layer Image Strategy

```
┌──────────────────────────────────────────────────────────────┐
│ Layer 3: repo-image (per-repo, per-lockfile)                  │
│ Fingerprint: project-image tag + hash(Cargo.lock,             │
│              pnpm-lock.yaml, ...)                             │
│ Contents: All repo dependencies fetched & compiled            │
│ Rebuild: When lockfiles change                                │
│ Warming: Done ONCE per workflow, shared across all steps      │
├──────────────────────────────────────────────────────────────┤
│ Layer 2: project-image (per-project)                          │
│ Fingerprint: stack-image tag + hash(dockerfile.project)       │
│ Contents: Project-specific system deps (graphviz, chromium)   │
│ Rebuild: When dockerfile.project changes (rare)               │
│ Defined by: dockerfile.project in consuming repo              │
├──────────────────────────────────────────────────────────────┤
│ Layer 1: stack-image (cross-repo reusable)                    │
│ Fingerprint: hash(extension versions from devflow.toml)       │
│ Contents: build-essential, Rust 1.85, Node 20, sccache        │
│ Rebuild: When stack versions change (rare)                    │
│ Publishable: Yes, to GHCR for cross-repo reuse               │
└──────────────────────────────────────────────────────────────┘
```

### Layer Design Rationale

**Why three layers?**

| Concern | stack-image | project-image | repo-image |
|---------|-------------|---------------|------------|
| Scope | Stack toolchains | Project system deps | Repo lockfile deps |
| Reuse | Cross-repo (high) | Per-project (medium) | Per-repo (low) |
| Change frequency | Rare | Rare | Frequent |
| Install time | Slow (toolchains) | Slow (apt packages) | Medium (fetch/compile) |
| Bloat risk | Low (standard) | Medium (project-specific) | Low (cached) |

Project-specific system deps (graphviz, chromium) **don't belong in stack-image** because they reduce cross-repo reuse and bloat images for projects that don't need them. They **don't belong in repo-image** because they rarely change and are slow to install. The **project-image** layer sits between: per-project, rarely rebuilt, cleanly separated.

### stack-image: Cross-Repo Reusable

- Contains only OS basics + stack toolchains (no project-specific tools)
- Publishable to GHCR with fingerprint-based tag
- Pulled by any repo with matching extension versions
- Rebuilt only when stack versions change in `devflow.toml`

### project-image: Per-Project System Dependencies

Defined in `dockerfile.project` — a **dedicated Dockerfile** in the consuming repo:

```dockerfile
# dockerfile.project — project-specific system dependencies
# This file participates in the project-image fingerprint.
# Only add system/OS-level dependencies here.

FROM ${STACK_IMAGE}

RUN apt-get update && apt-get install -y \
    graphviz \
    && rm -rf /var/lib/apt/lists/*

# Network services, complex deps, etc. use full Dockerfile syntax
```

Referenced in `devflow.toml`:

```toml
[project]
name = "kroki-rs-nxt"
dockerfile_project = "./dockerfile.project"
```

**Benefits**:

- Dockerfile format is expressive (network services, multi-step installs, conditional logic)
- Fingerprint is clean — only changes when actual deps change
- Standard format ops/DevOps teams understand
- No false-positive fingerprint changes from unrelated `devflow.toml` edits
- Each repo defines its own project-image on top of a shared stack-image

### repo-image: Cache Storage

**Lessons from v0.2.0**: Volume mounts (`-v` flags) caused issues:

- Pre-create directories required before container starts (Podman mount failure if dirs don't exist)
- macOS/Podman: `/tmp/` mounts fail — workspace-relative paths required (`:Z` SELinux flag)
- Permission workarounds: `chmod 777` before, `chown` after container exits
- GH Actions cache (tar save/restore) is used between jobs; volume mounts only within a job

**v0.3.0 approach**: Cache delivery differs by environment, workflow shape is identical:

**env:local (podman/docker)**:

- Cache directory: `.cache/devflow/{repo-id}/{stack}/` on the host
- `ci:setup` ensures dirs exist, runs dep fetch/compile inside container
- Cache persists on host filesystem between runs (no tar needed)
- devflow handles dir creation + permission management (no manual `chmod`/`chown`)

**env:gh (GitHub Actions)**:

- GH Actions cache (tar save/restore) persists cache between workflow runs
- Key: `{repo}-{stack}-{lockfile-hash}` with restore fallback keys
- `ci:setup` restores cache, runs dep fetch/compile, saves updated cache
- Within a workflow run, all steps share the same restored cache (no re-download)

**In both cases**: Cache warming happens **once** at the `ci:setup` stage. All subsequent steps (build, verify) execute with the warmed cache available. No repeated downloads or compiles across fmt:check, lint:static, test:unit, etc. devflow abstracts the environment-specific cache plumbing.

### Fingerprinting at Every Lifecycle Step

Every `ci:setup` invocation checks fingerprints for all three layers:

1. Check stack-image fingerprint (hash of enabled extension versions) — Match? Pull from GHCR or local cache. Miss? Rebuild.
2. Check project-image fingerprint (hash of stack-image + `dockerfile.project`) — Match? Use cached. Miss? Rebuild.
3. Check repo-image fingerprint (hash of project-image + lockfiles) — Match? Use cached. Miss? Rebuild (fetch/compile deps).

No stale images slip through.

---

## Extension Dependency Declaration

Extensions declare stack-level requirements in `devflow.toml`:

```toml
[extensions.rust]
source = "builtin"
version = "^0.3"

# Stack deps → goes into stack-image
[extensions.rust.stack]
toolchain = "1.85"
tools = ["sccache", "cargo-nextest"]

# Cache layout → mounted/restored at runtime
[extensions.rust.cache]
paths = ["registry", "git", "sccache"]
# Relative to .cache/devflow/{repo}/rust/
```

### Runtime Configurability

Extension declarations are **runtime-configurable** — the extension reads its config from the project's `devflow.toml`, not hardcoded at compile time. Different projects can specify different Rust versions, tools, etc.

### Extension Distribution

| Source | Description | Trust |
|--------|-------------|-------|
| `builtin` | Ships with devflow binary (rust, node, python) | Trusted |
| `vendor` | Published packages (npm, crates.io) with version pinning | User-verified |
| `path` | Repo-local scripts (`source = "path"`) | Auto-trusted if git-tracked |

Core trusted extensions (rust, node) ship with the devflow binary. They are activated by declaring them in `devflow.toml`. The devflow examples (`examples/tauri/`, `examples/node/`) should be tested in CI to prevent drift.

### Eliminating the `host_deps` Workaround

`setup:deps` aggregates extension stack declarations and:

- In host:native mode: verifies tools exist on host, installs if missing
- In host:container mode: ensures they're in the stack-image

Project-specific system deps go in `dockerfile.project` (not extensions). No separate `host_deps` extension needed.

---

## Target Auto-Detection

`ci:setup` auto-detects the environment:

- `GITHUB_ACTIONS=true` → env:gh
- Otherwise → env:local
- If `devflow.toml` specifies an override, cross-verify against environment signals

```toml
[runtime]
host = "container"     # "native" or "container"
# environment is auto-detected (local vs gh)
```

---

## Cache Coordination

Repo-scoped cache structure:

```
.cache/devflow/
├── {repo-id}/              # from [project] name in devflow.toml
│   ├── rust/               # declared by extensions.rust.cache
│   │   ├── registry/
│   │   ├── git/
│   │   └── sccache/
│   ├── node/               # declared by extensions.node.cache
│   │   ├── npm/
│   │   └── pnpm-store/
│   └── images/             # managed by devflow core
│       ├── project-image.tar
│       └── repo-image.tar
└── shared/                 # cross-repo
    └── images/
        └── stack-image.tar
```

**Repo identity**: From `[project] name` in `devflow.toml`. Fallback: hash of repo root path.

**GH Actions cache keys**: Include repo name (automatic via `${{ github.repository }}`) + layer fingerprints. No collision risk across repos.

**Cache paths are configurable at runtime** via `[extensions.*.cache]` in `devflow.toml`.

---

## devflow Command Model (v0.3.0)

| Command | Purpose | Status |
|---------|---------|--------|
| `setup:doctor` | Validate toolchain availability | - [ ] Existing |
| `setup:deps` | Verify/install all dependencies (native + container) | - [ ] Enhanced |
| `setup:toolchain` | Install language runtimes | - [ ] Existing |
| `ci:setup` | Prepare containerized env (image lifecycle) | - [ ] **New** |
| `ci:generate` | Generate workflows (CI, release, docs publish) | - [ ] Enhanced scope |
| `ci:check` | Verify all workflow consistency (CI, release, docs) | - [ ] Enhanced scope |
| `ci:plan` | Preview execution strategy | - [ ] Existing |
| `prune:cache` | Reclaim disk space (`--native`, `--container`, `--all`) | - [ ] Enhanced |
| `prune:gh-runs` | Clean GitHub Actions workflow run history | - [ ] Renamed |
| `ship:package` | Bundle artifacts to `dist/` | - [ ] **New** |
| `ship:release` | Publish release with tagging (env:gh only) | - [ ] **New** |

### `ci:generate` Scope Expansion

- [ ] CI workflow (build + verify pipeline)
- [ ] Release workflow (tagging, artifact publication)
- [ ] Docs publishing workflow (if docs config detected)
- [ ] All generated workflows checked by `ci:check`

### `prune:cache` Options

- `--native` — clear host:native caches (toolchain caches, build artifacts)
- `--container` — clear host:container caches (images, volume caches)
- `--all` — clear both native and container caches
- Always repo-aware (current project's cache by default, `--global` for all repos)

---

## Limitation Resolution Map

| # | v0.2.0 Limitation | Workaround Location | v0.3.0 Resolution | Priority |
|---|------------------|--------------------|--------------------|----------|
| 1 | Container/host parity gap | `.github/workflows/ci.yml` (3-phase) | Three-layer image + `ci:setup` | - [ ] **P0** |
| 9 | CI image cache not optimized | Manual SHA256 fingerprinting | Fingerprint-based layer caching | - [ ] **P0** |
| 2 | No host dependency management | `tools/devflow-ext-host-deps.mjs` | `setup:deps` + `dockerfile.project` | - [ ] **P1** |
| 3 | npm cache not coordinated | Manual paths in scripts | Extension cache declarations | - [ ] **P1** |
| 5 | No Dockerfile generation | Manual `Dockerfile.devflow` | Extension-driven stack-image + `dockerfile.project` | - [ ] **P1** |
| 4 | Extension trust friction | `trusted = true` in devflow.toml | Auto-trust git-tracked extensions | - [ ] **P2** |
| 6 | CLI path hardcoding | `crates/devflow-cli` shim | `dwf` path resolution chain | - [ ] **P2** |
| 7 | Cache directory unstandardized | Multiple scripts | `.cache/devflow/{repo}/` | - [ ] **P2** |
| 10 | No cleanup commands | `scripts/devflow-prune.sh` | `prune:cache` + `prune:gh-runs` | - [ ] **P2** |
| 8 | Parallel execution in CI | Manual bash PID arrays | Abstract in `ci:generate` output | - [ ] **P3** |

---

## Workaround Files to Replace

| v0.2.0 Workaround File | v0.3.0 Replacement |
|------------------------|-------------------|
| `Dockerfile.devflow` | stack-image (auto-generated) + `dockerfile.project` |
| `.github/workflows/ci.yml` (manual) | `ci:generate` (CI + release + docs) |
| `tools/devflow-ext-host-deps.mjs` | `setup:deps` + extension declarations |
| `scripts/ci-local-podman.sh` | `ci:setup` with auto-detect |
| `scripts/devflow-host-deps.sh` | `setup:deps` |
| `scripts/devflow-prune.sh` | `prune:cache` + `prune:gh-runs` |
| `crates/devflow-cli/` shim | `dwf` path resolution |

---

## Execution Order

| Priority | Item | Description | Effort |
|----------|------|-------------|--------|
| 1 | - [ ] Image layer schema | Define fingerprinting algorithm, layer contracts | M |
| 2 | - [ ] `ci:setup` | Three-layer build/restore with fingerprinting | L |
| 3 | - [ ] `setup:deps` enhancement | Aggregate extension deps for host:native + host:container | M |
| 4 | - [ ] `ci:generate` v2 | CI + release + docs workflows | M |
| 5 | - [ ] Cache coordination | Repo-scoped `.cache/devflow/` with extension paths | M |
| 6 | - [ ] Extension trust | Auto-trust git-tracked repo extensions | S |
| 7 | - [ ] `prune:cache` + `prune:gh-runs` | Cleanup with native/container/repo awareness | M |
| 8 | - [ ] `ship:package` / `ship:release` | Artifact packaging and release workflow | M |

**Effort key**: S = Small (1-2 days), M = Medium (3-5 days), L = Large (1-2 weeks)

---

## Open Design Questions

1. Should stack-image and project-image be published to GHCR, or is local + GH Actions cache sufficient?
2. How should `ci:setup` auto-detect env (local vs gh)? `GITHUB_ACTIONS` env var + devflow.toml cross-verify?
3. What is the `ci:generate` output shape — single workflow file per concern, or composite actions?
4. How should extensions declare system (native) dependencies for both host:native verification and stack-image generation?
