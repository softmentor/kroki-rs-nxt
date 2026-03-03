#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ENGINE=${ENGINE:-podman}
IMAGE_NAME=${IMAGE_NAME:-kroki-rs-nxt-server:local}
CONTAINER_NAME=${CONTAINER_NAME:-kroki-rs-nxt-server}
PUBLIC_PORT=${PUBLIC_PORT:-8000}
ADMIN_PORT=${ADMIN_PORT:-8081}
DETACH=${DETACH:-0}

if ! command -v "$ENGINE" >/dev/null 2>&1; then
  echo "[container] Error: $ENGINE is not installed or not on PATH." >&2
  exit 1
fi

echo "[container] Building image: $IMAGE_NAME"
"$ENGINE" build -f "$ROOT_DIR/Dockerfile.server-runtime" -t "$IMAGE_NAME" "$ROOT_DIR"

if [ "$DETACH" = "1" ]; then
  "$ENGINE" rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
  echo "[container] Starting detached container: $CONTAINER_NAME on ports ${PUBLIC_PORT}/${ADMIN_PORT}"
  "$ENGINE" run -d --rm \
    --name "$CONTAINER_NAME" \
    -p "${PUBLIC_PORT}:8000" \
    -p "${ADMIN_PORT}:8081" \
    "$IMAGE_NAME"
  echo "[container] Follow logs: $ENGINE logs -f $CONTAINER_NAME"
  echo "[container] Stop: $ENGINE stop $CONTAINER_NAME"
else
  echo "[container] Running kroki-server on ports ${PUBLIC_PORT}/${ADMIN_PORT}"
  "$ENGINE" run --rm \
    --name "$CONTAINER_NAME" \
    -p "${PUBLIC_PORT}:8000" \
    -p "${ADMIN_PORT}:8081" \
    "$IMAGE_NAME"
fi
