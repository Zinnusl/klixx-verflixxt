#!/usr/bin/env bash
# Release build for deployment. Output: web/dist/.
set -euo pipefail
cd "$(dirname "$0")/.."

if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
  echo "Installing wasm32-unknown-unknown target..."
  rustup target add wasm32-unknown-unknown
fi

env -u NO_COLOR trunk build --release

echo ""
echo "Built to web/dist/."
echo "Test the release build with: ./scripts/serve_web.sh"
