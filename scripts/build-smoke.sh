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
    # `--features static` embeds crates/heramind-api/static/ into the binary at
    # compile time. Nothing else populates it, so refresh it from the real
    # frontend build first — otherwise the binary serves whatever an earlier
    # session left behind. heramind-api/build.rs fails the build if this is
    # skipped and the embed turns out to be older than web/dist.
    if [ -f web/dist/index.html ] && [ web/dist/index.html -nt crates/heramind-api/static/index.html ]; then
        echo "==> Syncing web/dist → crates/heramind-api/static/ ..."
        mkdir -p crates/heramind-api/static
        rsync -a --delete web/dist/ crates/heramind-api/static/
    fi
    echo "==> Building heramind + runner (one or both missing)..."
    cargo build --release --bin heramind --bin heramind-extension-runner --features static
fi
exec ./scripts/smoke-serve.sh
