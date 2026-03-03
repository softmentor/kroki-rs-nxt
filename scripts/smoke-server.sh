#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SERVER_BIN="$ROOT_DIR/target/release/kroki-server"
PUBLIC_PORT="${PUBLIC_PORT:-8090}"
ADMIN_PORT="${ADMIN_PORT:-8091}"
BASE_URL="http://127.0.0.1:${PUBLIC_PORT}"
ADMIN_URL="http://127.0.0.1:${ADMIN_PORT}"

cleanup() {
  if [ -n "${SERVER_PID:-}" ]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

echo "[smoke] building release binaries..."
cargo build --release -p kroki-server >/dev/null

echo "[smoke] starting server..."
"$SERVER_BIN" --mode dev --port "$PUBLIC_PORT" --admin-port "$ADMIN_PORT" >/tmp/kroki-smoke.log 2>&1 &
SERVER_PID=$!
sleep 1

echo "[smoke] check /health"
health_code=$(curl -sS -o /tmp/kroki-smoke-health.json -w "%{http_code}" "$ADMIN_URL/health")
[ "$health_code" = "200" ]

echo "[smoke] check /metrics"
metrics_code=$(curl -sS -o /tmp/kroki-smoke-metrics.txt -w "%{http_code}" "$ADMIN_URL/metrics")
[ "$metrics_code" = "200" ]

echo "[smoke] check /capabilities"
caps_code=$(curl -sS -o /tmp/kroki-smoke-caps.json -w "%{http_code}" "$BASE_URL/capabilities")
[ "$caps_code" = "200" ]
grep -q '"provider_id":"graphviz"' /tmp/kroki-smoke-caps.json
grep -q '"provider_id":"d2"' /tmp/kroki-smoke-caps.json
grep -q '"provider_id":"mermaid"' /tmp/kroki-smoke-caps.json
grep -q '"provider_id":"bpmn"' /tmp/kroki-smoke-caps.json

echo "[smoke] check echo render"
echo_code=$(curl -sS -H "content-type: application/json" \
  -d '{"source":"A -> B","diagram_type":"echo","output_format":"Svg"}' \
  -o /tmp/kroki-smoke-echo.json -w "%{http_code}" "$BASE_URL/render")
[ "$echo_code" = "200" ]

echo "[smoke] check graphviz render status"
gv_code=$(curl -sS -H "content-type: application/json" \
  -d '{"source":"digraph G {\\nA -> B;\\n}","diagram_type":"graphviz","output_format":"Svg"}' \
  -o /tmp/kroki-smoke-graphviz.txt -w "%{http_code}" "$BASE_URL/render")
if command -v dot >/dev/null 2>&1; then
  [ "$gv_code" = "200" ]
else
  [ "$gv_code" = "503" ]
fi

echo "[smoke] check mermaid render status"
mm_code=$(curl -sS -H "content-type: application/json" \
  -d '{"source":"graph TD; A-->B;","diagram_type":"mermaid","output_format":"Svg"}' \
  -o /tmp/kroki-smoke-mermaid.txt -w "%{http_code}" "$BASE_URL/render")
if command -v mmdc >/dev/null 2>&1; then
  [ "$mm_code" = "200" ]
else
  [ "$mm_code" = "503" ]
fi

echo "[smoke] check bpmn baseline status"
bpmn_code=$(curl -sS -H "content-type: application/json" \
  -d '{"source":"<?xml version=\"1.0\"?><definitions></definitions>","diagram_type":"bpmn","output_format":"Svg"}' \
  -o /tmp/kroki-smoke-bpmn.txt -w "%{http_code}" "$BASE_URL/render")
[ "$bpmn_code" = "500" ]

echo "[smoke] PASS"
