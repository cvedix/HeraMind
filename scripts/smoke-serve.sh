#!/bin/bash
# Smoke-test serve on :9375 with a PERSISTENT data dir. The env's DB
# records ABSOLUTE extension paths under /tmp/heramind-smoke — keep that
# path alive as a symlink to the persistent copy so macOS /tmp cleanup
# can only break a symlink (recreated here), never the data.
set -e
cd "$(dirname "$0")/.."
export HERAMIND_DATA_DIR="$PWD/.smoke/data"
if [ ! -L /tmp/heramind-smoke ]; then
  rm -rf /tmp/heramind-smoke
  ln -s "$PWD/.smoke" /tmp/heramind-smoke
fi
exec ./target/release/heramind serve --port 9375
