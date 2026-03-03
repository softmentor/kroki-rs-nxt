#!/usr/bin/env sh
set -eu

REPO_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PREFIX=${PREFIX:-$HOME/.local/bin}
COMPLETION_DIR=${COMPLETION_DIR:-$HOME/.local/share/kroki/completions}
INSTALL_SOURCE=${INSTALL_SOURCE:-dist}
RELEASE_URL=${RELEASE_URL:-}
RELEASE_SERVER_URL=${RELEASE_SERVER_URL:-}
CLI_BIN_NAME="kroki"
SERVER_BIN_NAME="kroki-server"
TARGET_CLI_BIN="$REPO_DIR/target/release/$CLI_BIN_NAME"
TARGET_SERVER_BIN="$REPO_DIR/target/release/$SERVER_BIN_NAME"
DIST_CLI_BIN="$REPO_DIR/dist/$CLI_BIN_NAME"
DIST_SERVER_BIN="$REPO_DIR/dist/$SERVER_BIN_NAME"
TMP_BIN=""
TMP_SERVER_BIN=""

mkdir -p "$PREFIX"
mkdir -p "$COMPLETION_DIR"

install_from_dist() {
  if [ ! -x "$DIST_CLI_BIN" ]; then
    echo "[install] Error: dist binary not found at $DIST_CLI_BIN" >&2
    echo "[install] Build/package the release first or use INSTALL_SOURCE=build" >&2
    exit 1
  fi
  if [ ! -x "$DIST_SERVER_BIN" ]; then
    echo "[install] Error: dist binary not found at $DIST_SERVER_BIN" >&2
    echo "[install] Build/package the release first or use INSTALL_SOURCE=build" >&2
    exit 1
  fi
  install -m 0755 "$DIST_CLI_BIN" "$PREFIX/$CLI_BIN_NAME"
  install -m 0755 "$DIST_SERVER_BIN" "$PREFIX/$SERVER_BIN_NAME"
}

install_from_build() {
  echo "[install] Building release binaries..."
  cargo build --release -p kroki-cli -p kroki-server --manifest-path "$REPO_DIR/Cargo.toml"
  if [ ! -x "$TARGET_CLI_BIN" ]; then
    echo "[install] Error: expected binary not found at $TARGET_CLI_BIN" >&2
    exit 1
  fi
  if [ ! -x "$TARGET_SERVER_BIN" ]; then
    echo "[install] Error: expected binary not found at $TARGET_SERVER_BIN" >&2
    exit 1
  fi
  install -m 0755 "$TARGET_CLI_BIN" "$PREFIX/$CLI_BIN_NAME"
  install -m 0755 "$TARGET_SERVER_BIN" "$PREFIX/$SERVER_BIN_NAME"
}

install_from_url() {
  if [ -z "$RELEASE_URL" ]; then
    echo "[install] Error: RELEASE_URL is required when INSTALL_SOURCE=url" >&2
    exit 1
  fi
  TMP_BIN=$(mktemp "${TMPDIR:-/tmp}/kroki-cli.XXXXXX")
  echo "[install] Downloading CLI binary from $RELEASE_URL ..."
  curl -fsSL "$RELEASE_URL" -o "$TMP_BIN"
  chmod +x "$TMP_BIN"
  install -m 0755 "$TMP_BIN" "$PREFIX/$CLI_BIN_NAME"
  if [ -n "$RELEASE_SERVER_URL" ]; then
    TMP_SERVER_BIN=$(mktemp "${TMPDIR:-/tmp}/kroki-server.XXXXXX")
    echo "[install] Downloading server binary from $RELEASE_SERVER_URL ..."
    curl -fsSL "$RELEASE_SERVER_URL" -o "$TMP_SERVER_BIN"
    chmod +x "$TMP_SERVER_BIN"
    install -m 0755 "$TMP_SERVER_BIN" "$PREFIX/$SERVER_BIN_NAME"
  else
    echo "[install] RELEASE_SERVER_URL not set; skipping server binary install"
  fi
}

case "$INSTALL_SOURCE" in
  dist) install_from_dist ;;
  build) install_from_build ;;
  url) install_from_url ;;
  *)
    echo "[install] Error: unsupported INSTALL_SOURCE '$INSTALL_SOURCE' (use dist|build|url)" >&2
    exit 1
    ;;
esac

echo "[install] Installed $CLI_BIN_NAME to $PREFIX/$CLI_BIN_NAME"
if [ -x "$PREFIX/$SERVER_BIN_NAME" ]; then
  echo "[install] Installed $SERVER_BIN_NAME to $PREFIX/$SERVER_BIN_NAME"
fi
echo "[install] Ensure '$PREFIX' is on PATH"
"$PREFIX/$CLI_BIN_NAME" --version || true
"$PREFIX/$SERVER_BIN_NAME" --version || true

completion_ok=0
for shell in bash zsh fish; do
  if "$PREFIX/$CLI_BIN_NAME" completions --shell "$shell" --output "$COMPLETION_DIR" >/dev/null 2>&1; then
    completion_ok=1
  fi
done
if [ "$completion_ok" -eq 1 ]; then
  echo "[install] Completion files generated under $COMPLETION_DIR"
else
  echo "[install] Completion generation skipped (binary does not expose completions command yet)"
fi

if [ -n "$TMP_BIN" ] && [ -f "$TMP_BIN" ]; then
  rm -f "$TMP_BIN"
fi
if [ -n "$TMP_SERVER_BIN" ] && [ -f "$TMP_SERVER_BIN" ]; then
  rm -f "$TMP_SERVER_BIN"
fi
