#!/usr/bin/env sh
set -eu

MODE="check"
INSTALL_CHROMIUM=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      MODE="${2:-check}"
      shift 2
      ;;
    --without-chromium)
      INSTALL_CHROMIUM=0
      shift
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [ "$MODE" != "check" ] && [ "$MODE" != "install" ]; then
  echo "invalid mode: $MODE (expected: check|install)" >&2
  exit 2
fi

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

missing=""
check_dep() {
  name="$1"
  if ! has_cmd "$name"; then
    if [ -z "$missing" ]; then
      missing="$name"
    else
      missing="$missing $name"
    fi
  fi
}

run_checks() {
  missing=""
  check_dep dot
  check_dep d2
  check_dep mmdc
}

run_checks
if [ -n "$missing" ] && [ "$MODE" = "install" ]; then
  install_args=""
  if [ "$INSTALL_CHROMIUM" = "0" ]; then
    install_args="--without-chromium"
  fi
  echo "[host-deps] missing:$missing"
  echo "[host-deps] installing runtime dependencies..."
  if [ -n "$install_args" ]; then
    sh scripts/install-runtime-deps.sh "$install_args"
  else
    sh scripts/install-runtime-deps.sh
  fi
  run_checks
fi

if [ -n "$missing" ]; then
  echo "[host-deps] missing runtime dependencies:$missing" >&2
  echo "[host-deps] run: KROKI_HOST_DEPS_MODE=install dwf setup:deps" >&2
  echo "[host-deps] or:  ./scripts/install-runtime-deps.sh" >&2
  exit 1
fi

echo "[host-deps] runtime dependencies available"
