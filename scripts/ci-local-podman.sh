#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_NAME="kroki-rs-nxt-ci:local"

mkdir -p "$ROOT_DIR/.cargo-cache/registry" \
         "$ROOT_DIR/.cargo-cache/git" \
         "$ROOT_DIR/.cargo-cache/sccache" \
         "$ROOT_DIR/target/ci"

if ! command -v podman >/dev/null 2>&1; then
  echo "[ci-local] Error: podman is not installed or not on PATH."
  exit 1
fi

if ! podman info >/dev/null 2>&1; then
  echo "[ci-local] Error: Podman daemon/machine is not available."
  echo "[ci-local] Run: podman machine init (first time), then podman machine start"
  exit 1
fi

echo "[ci-local] Building Podman CI image..."
podman build -f "$ROOT_DIR/Dockerfile.devflow" -t "$IMAGE_NAME" "$ROOT_DIR"

echo "[ci-local] Running devflow PR gate in container..."
podman run --rm \
  -v "$ROOT_DIR:/workspace:Z" \
  -w /workspace \
  -e CARGO_HOME=/workspace/.cargo-cache \
  -e CARGO_TARGET_DIR=/workspace/target/ci \
  -e SCCACHE_DIR=/workspace/.cargo-cache/sccache \
  -e RUSTC_WRAPPER=sccache \
  -e npm_config_cache=/workspace/.npm-cache \
  -e PATH=/workspace/.cargo-cache/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  "$IMAGE_NAME" \
  bash -c 'npm ci && dwf check:pr'
