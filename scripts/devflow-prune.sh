#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TASK="deep"
KEEP_RUNS="${PRUNE_GH_RUN_LIMIT:-100}"
DELETE_FAILED_RUNS="${PRUNE_GH_DELETE_FAILED:-true}"

need_cmd() {
  local cmd="$1"
  local msg="$2"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "[prune] Error: missing '$cmd'. ${msg}"
    exit 1
  fi
}

usage() {
  cat <<'EOF'
Usage: scripts/devflow-prune.sh --task <name>

Tasks:
  runs         Prune GitHub Actions workflow runs.
  containers   Prune local Podman/Docker resources.
  deep         Run cache (local + GH) + runs + containers + temp cleanup.

Environment:
  PRUNE_GH_RUN_LIMIT=<n>       Number of recent GH workflow runs to keep (default: 100)
  PRUNE_GH_DELETE_FAILED=true  Delete failed/canceled GH runs (default: true)
EOF
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --task)
        TASK="${2:-}"
        shift 2
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        echo "[prune] Error: unknown argument '$1'"
        usage
        exit 1
        ;;
    esac
  done
}

prune_local_cache() {
  echo "[prune] Pruning local cache/work directories..."
  rm -rf \
    "$ROOT_DIR/.cache/devflow" \
    "$ROOT_DIR/.cargo-cache/registry" \
    "$ROOT_DIR/.cargo-cache/git" \
    "$ROOT_DIR/.cargo-cache/sccache" \
    "$ROOT_DIR/target/ci"
  mkdir -p \
    "$ROOT_DIR/.cache/devflow/node/npm" \
    "$ROOT_DIR/.cache/devflow/rust/cargo/sccache" \
    "$ROOT_DIR/.cache/devflow/rust/target" \
    "$ROOT_DIR/.cargo-cache/registry" \
    "$ROOT_DIR/.cargo-cache/git" \
    "$ROOT_DIR/.cargo-cache/sccache" \
    "$ROOT_DIR/target/ci"
  echo "[prune] Local cache prune complete."
}

prune_gh_cache() {
  need_cmd gh "Install GitHub CLI and run 'gh auth login'."
  need_cmd jq "Install jq for JSON filtering."

  echo "[prune] Pruning GitHub Actions caches..."
  local total_size threshold ref
  threshold=8589934592 # 8 GiB

  # Delete stale PR caches not accessed in 24h.
  gh cache list --limit 100 --json id,ref,lastAccessedAt | jq -r '
    .[] | select(.ref | startswith("refs/pull/"))
         | select((.lastAccessedAt | sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601) < (now - 86400))
         | .id' | xargs -I {} gh cache delete {} >/dev/null 2>&1 || true

  total_size="$(gh cache list --limit 100 --json sizeInBytes --jq '[.[].sizeInBytes] | add // 0')"
  echo "[prune] GH cache size: $((total_size / 1024 / 1024)) MB"
  if [[ "$total_size" -lt "$threshold" ]]; then
    echo "[prune] GH cache is below threshold; skipping aggressive prune."
    return
  fi

  echo "[prune] Threshold exceeded; pruning duplicate cache families per ref..."
  while IFS= read -r ref; do
    [[ -n "$ref" ]] || continue
    gh cache list --ref "$ref" --json id,key | jq -r '.[] | select(.key | contains("cargo-")) | .id' | tail -n +2 | xargs -I {} gh cache delete {} >/dev/null 2>&1 || true
    gh cache list --ref "$ref" --json id,key | jq -r '.[] | select(.key | contains("docker-ci-")) | .id' | tail -n +2 | xargs -I {} gh cache delete {} >/dev/null 2>&1 || true
    gh cache list --ref "$ref" --json id,key | jq -r '.[] | select(.key | contains("buildkit-")) | .id' | tail -n +11 | xargs -I {} gh cache delete {} >/dev/null 2>&1 || true
  done < <(gh cache list --limit 100 --json ref --jq '.[].ref' | sort | uniq)
  echo "[prune] GH cache prune complete."
}

prune_gh_runs() {
  need_cmd gh "Install GitHub CLI and run 'gh auth login'."
  echo "[prune] Pruning GitHub Actions workflow runs (keep latest ${KEEP_RUNS})..."

  if [[ "${DELETE_FAILED_RUNS}" == "true" ]]; then
    local failed canceled
    failed="$(gh run list --status failure --limit 1000 --json databaseId --jq '.[].databaseId' || true)"
    canceled="$(gh run list --status cancelled --limit 1000 --json databaseId --jq '.[].databaseId' || true)"
    {
      printf '%s\n' "$failed"
      printf '%s\n' "$canceled"
    } | sed '/^$/d' | sort -u | xargs -I {} gh run delete {} >/dev/null 2>&1 || true
  fi

  local all_runs total_count old_runs
  all_runs="$(gh run list --limit 1000 --json databaseId --jq '.[].databaseId' || true)"
  total_count="$(printf '%s\n' "$all_runs" | sed '/^$/d' | wc -l | tr -d ' ')"

  if [[ "$total_count" -gt "$KEEP_RUNS" ]]; then
    old_runs="$(printf '%s\n' "$all_runs" | sed '/^$/d' | tail -n +$((KEEP_RUNS + 1)))"
    printf '%s\n' "$old_runs" | xargs -I {} gh run delete {} >/dev/null 2>&1 || true
  fi
  echo "[prune] GH run prune complete."
}

prune_podman_or_docker() {
  local engine=""
  if command -v podman >/dev/null 2>&1; then
    engine="podman"
  elif command -v docker >/dev/null 2>&1; then
    engine="docker"
  else
    echo "[prune] No container engine found; skipping podman/docker prune."
    return
  fi

  echo "[prune] Pruning local container resources via $engine..."
  "$engine" container prune -f >/dev/null 2>&1 || true
  "$engine" image prune -af >/dev/null 2>&1 || true
  "$engine" volume prune -f >/dev/null 2>&1 || true
  "$engine" network prune -f >/dev/null 2>&1 || true
  "$engine" system prune -af >/dev/null 2>&1 || true
  echo "[prune] Container prune complete."
}

prune_local_temp() {
  echo "[prune] Pruning local Chrome code-sign clone temp bloat (if present)..."
  if [[ -n "$(command -v getconf || true)" ]]; then
    local temp_base bloat_dir
    temp_base="$(getconf DARWIN_USER_TEMP_DIR 2>/dev/null || true)"
    if [[ -n "$temp_base" ]]; then
      bloat_dir="$(echo "$temp_base" | sed 's/\/T\/$/\/X\//')com.google.Chrome.code_sign_clone"
      if [[ -d "$bloat_dir" ]]; then
        rm -rf "$bloat_dir"
        echo "[prune] Removed: $bloat_dir"
      fi
    fi
  fi
}

main() {
  parse_args "$@"

  case "$TASK" in
    runs)
      prune_gh_runs
      ;;
    containers)
      prune_podman_or_docker
      ;;
    deep)
      prune_local_cache
      prune_gh_cache
      prune_gh_runs
      prune_podman_or_docker
      prune_local_temp
      ;;
    *)
      echo "[prune] Error: invalid task '$TASK'"
      usage
      exit 1
      ;;
  esac
}

main "$@"
