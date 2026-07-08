#!/bin/bash
# Development mode with Extension Runner rebuild
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔧 Building Extension Runner (debug)..."
"$SCRIPT_DIR/build-extension-runner.sh" debug

echo "🚀 Starting Tauri dev mode..."
cd "$PROJECT_ROOT"
npm run tauri:dev
