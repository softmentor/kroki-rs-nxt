#!/usr/bin/env sh
set -eu

# Installs runtime dependencies required by kroki binary/server on host systems.
# Intended for end users running installed binaries (not repo developers).

if [ "${1:-}" = "--help" ]; then
  cat <<'HELP'
Usage: scripts/install-runtime-deps.sh [--without-chromium]

Installs host dependencies used by kroki surfaces:
- graphviz (dot)
- d2
- nodejs
- @mermaid-js/mermaid-cli (mmdc)
- chromium (optional)

On macOS: uses Homebrew.
On Debian/Ubuntu: uses apt + npm.
HELP
  exit 0
fi

INSTALL_CHROMIUM=1
if [ "${1:-}" = "--without-chromium" ]; then
  INSTALL_CHROMIUM=0
fi

OS="$(uname -s)"

install_npm_mermaid() {
  if command -v npm >/dev/null 2>&1; then
    npm install -g @mermaid-js/mermaid-cli
  else
    echo "[deps] npm not found; cannot install @mermaid-js/mermaid-cli" >&2
    exit 1
  fi
}

if [ "$OS" = "Darwin" ]; then
  if ! command -v brew >/dev/null 2>&1; then
    echo "[deps] Homebrew is required on macOS: https://brew.sh" >&2
    exit 1
  fi

  brew install graphviz d2 node
  if [ "$INSTALL_CHROMIUM" = "1" ]; then
    brew install --cask chromium || true
  fi
  install_npm_mermaid
elif [ "$OS" = "Linux" ]; then
  if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y graphviz curl ca-certificates gnupg

    if ! command -v node >/dev/null 2>&1; then
      curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key \
        | sudo gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
      echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_20.x nodistro main" \
        | sudo tee /etc/apt/sources.list.d/nodesource.list >/dev/null
      sudo apt-get update
      sudo apt-get install -y nodejs
    fi

    if [ "$INSTALL_CHROMIUM" = "1" ]; then
      sudo apt-get install -y chromium || true
    fi

    if ! command -v d2 >/dev/null 2>&1; then
      echo "[deps] d2 is not installed. Install from https://d2lang.com/tour/install" >&2
    fi

    install_npm_mermaid
  else
    echo "[deps] Unsupported Linux distribution for automatic install."
    echo "Install graphviz, d2, nodejs, and @mermaid-js/mermaid-cli manually."
    exit 1
  fi
else
  echo "[deps] Unsupported OS: $OS" >&2
  exit 1
fi

echo "[deps] done"
