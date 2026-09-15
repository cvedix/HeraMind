#!/bin/bash
# Build BOTH binaries needed for the smoke environment. The target directory
# gets periodically cleaned (by cargo clean, disk cleanup, or Docker) — this
# script ensures both heramind and heramind-extension-runner exist before serve.
set -e
cd "$(dirname "$0")/.."
NEED_BUILD=0
[ ! -f target/release/heramind ] && NEED_BUILD=1
[ ! -f target/release/heramind-extension-runner ] && NEED_BUILD=1
if [ $NEED_BUILD -eq 1 ]; then
    echo "==> Building heramind + runner (one or both missing)..."
    cargo build --release --bin heramind --bin heramind-extension-runner --features static
fi
exec ./scripts/smoke-serve.sh
