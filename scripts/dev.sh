#!/usr/bin/env bash
# Start trunk's dev server with hot reload on http://localhost:8080.
set -euo pipefail
cd "$(dirname "$0")/.."

api_port="${HOTSPOT_EDITOR_API_PORT:-8082}"
python3 scripts/hotspot_editor_api.py --port "$api_port" &
api_pid=$!

cleanup() {
  kill "$api_pid" >/dev/null 2>&1 || true
  wait "$api_pid" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

echo "Game:           http://127.0.0.1:8080/"
echo "Hotspot editor: http://127.0.0.1:8080/hotspot-editor/"
echo "Frame picker:   http://127.0.0.1:8080/frame-picker/"
echo "Editor write API: http://127.0.0.1:${api_port}/"
env -u NO_COLOR trunk serve "$@"
