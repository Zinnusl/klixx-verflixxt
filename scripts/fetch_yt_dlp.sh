#!/usr/bin/env bash
# Download a local yt-dlp binary into tmp/ for research-only video frame extraction.
set -euo pipefail
cd "$(dirname "$0")/.."

mkdir -p tmp
curl -L --fail --show-error \
  -o tmp/yt-dlp \
  https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp
chmod +x tmp/yt-dlp
tmp/yt-dlp --version
