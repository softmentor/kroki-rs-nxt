#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ENGINE=${ENGINE:-podman}
IMAGE_NAME=${IMAGE_NAME:-node:20-bookworm}
CONTAINER_NAME=${CONTAINER_NAME:-kroki-rs-nxt-web-app}
WEB_PORT=${WEB_PORT:-5173}
DETACH=${DETACH:-0}
PNPM_VERSION=${PNPM_VERSION:-9.15.0}

if ! command -v "$ENGINE" >/dev/null 2>&1; then
  echo "[container] Error: $ENGINE is not installed or not on PATH." >&2
  exit 1
fi

if [ "$ENGINE" = "podman" ]; then
  VOLUME_SUFFIX=':Z'
else
  VOLUME_SUFFIX=''
fi

RUN_CMD="npm i -g pnpm@${PNPM_VERSION} && pnpm install && pnpm --filter @kroki/web-app dev --host 0.0.0.0 --port ${WEB_PORT}"

if [ "$DETACH" = "1" ]; then
  "$ENGINE" rm -f "$CONTAINER_NAME" >/dev/null 2>&1 || true
  echo "[container] Starting detached web app container: $CONTAINER_NAME on port ${WEB_PORT}"
  "$ENGINE" run -d --rm \
    --name "$CONTAINER_NAME" \
    -p "${WEB_PORT}:5173" \
    -v "${ROOT_DIR}:/workspace${VOLUME_SUFFIX}" \
    -w /workspace \
    "$IMAGE_NAME" \
    sh -lc "$RUN_CMD"
  echo "[container] Open: http://127.0.0.1:${WEB_PORT}"
  echo "[container] Follow logs: $ENGINE logs -f $CONTAINER_NAME"
  echo "[container] Stop: $ENGINE stop $CONTAINER_NAME"
else
  echo "[container] Running web app on port ${WEB_PORT}"
  echo "[container] Open: http://127.0.0.1:${WEB_PORT}"
  "$ENGINE" run --rm \
    --name "$CONTAINER_NAME" \
    -p "${WEB_PORT}:5173" \
    -v "${ROOT_DIR}:/workspace${VOLUME_SUFFIX}" \
    -w /workspace \
    "$IMAGE_NAME" \
    sh -lc "$RUN_CMD"
fi
