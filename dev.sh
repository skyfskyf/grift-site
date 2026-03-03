#!/usr/bin/env bash
# Build the WASM package and start a local server for development.
# Usage: ./dev.sh [--no-serve]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> Building WASM package..."
wasm-pack build "$SCRIPT_DIR/crates/grift_wasm" \
  --target web \
  --out-dir "$SCRIPT_DIR/site/pkg" \
  --dev

if [[ "${1:-}" == "--no-serve" ]]; then
  echo "==> Build complete. Files in site/"
  exit 0
fi

echo "==> Starting local server at http://localhost:8080"
echo "    Press Ctrl+C to stop."

# Try common simple HTTP servers
if command -v python3 &>/dev/null; then
  cd "$SCRIPT_DIR/site" && python3 -m http.server 8080
elif command -v npx &>/dev/null; then
  cd "$SCRIPT_DIR/site" && npx serve -l 8080
else
  echo "ERROR: No suitable HTTP server found. Install python3 or Node.js."
  exit 1
fi
