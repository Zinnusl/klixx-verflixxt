#!/usr/bin/env bash
# Serve the already-built web/dist/ via Python. For hot reload use dev.sh.
set -euo pipefail
cd "$(dirname "$0")/../web/dist"

PORT="${PORT:-8080}"
echo "Serving web/dist on http://localhost:${PORT}"
python3 -m http.server "${PORT}"
