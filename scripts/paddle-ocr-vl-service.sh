#!/usr/bin/env bash
# Local PaddleOCR-VL service for HeraMind on Apple Silicon.
set -euo pipefail

ocr_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ocr_data="${HERAMIND_OCR_DATA_DIR:-$ocr_root/data/paddle-ocr-vl-service}"
ocr_python="$ocr_data/venv/bin/python"
ocr_uv="$ocr_data/bootstrap/bin/uv"
ocr_server="$ocr_data/upstream/server.py"
ocr_server_sha="2e4f961a8433d9b3bf5cc3050721654437f45ce8875a9f390240d93016874a97"

export HERAMIND_OCR_DATA_DIR="$ocr_data"
export PADDLE_PDX_CACHE_HOME="$ocr_data/models"
export HF_HOME="$ocr_data/huggingface"
export PADDLE_PDX_MODEL_SOURCE=huggingface
# Skip connectivity probes; actual downloads still verify HTTPS normally.
export PADDLE_PDX_DISABLE_MODEL_SOURCE_CHECK=True
export OMP_NUM_THREADS="${OMP_NUM_THREADS:-4}"
export OPENBLAS_NUM_THREADS="${OPENBLAS_NUM_THREADS:-4}"

case "${1:-status}" in
  setup)
    if [[ "$(uname -s)" != Darwin || "$(uname -m)" != arm64 ]]; then
      echo "This dependency lock targets macOS Apple Silicon." >&2
      exit 1
    fi
    mkdir -p "$ocr_data/upstream"
    if [[ ! -x "$ocr_uv" ]]; then
      python3 -m venv "$ocr_data/bootstrap"
      "$ocr_data/bootstrap/bin/python" -m pip install 'uv==0.13.6'
    fi
    "$ocr_uv" python install 3.11.18 --install-dir "$ocr_data/python"
    if [[ ! -x "$ocr_python" ]]; then
      "$ocr_uv" venv "$ocr_data/venv" --python \
        "$ocr_data/python/cpython-3.11.18-macos-aarch64-none/bin/python3.11"
    fi
    "$ocr_uv" pip sync --python "$ocr_python" \
      "$ocr_root/deploy/paddle-ocr-vl/requirements-macos.lock"
    if [[ ! -f "$ocr_server" ]]; then
      curl -fL --retry 3 \
        'https://raw.githubusercontent.com/camthink-ai/NeoMind-Extensions/v2.14.0/extensions/paddle-ocr-vl/server/server.py' \
        -o "$ocr_server"
    fi
    ;;
  download|start)
    if [[ ! -x "$ocr_python" || ! -f "$ocr_server" ]]; then
      echo "Run bash scripts/paddle-ocr-vl-service.sh setup first." >&2
      exit 1
    fi
    ;;
  status)
    exec curl --fail --silent --show-error --max-time 5 \
      "http://127.0.0.1:${HERAMIND_OCR_PORT:-8000}/health"
    ;;
  *)
    echo "Usage: $0 [setup|download|start|status]" >&2
    exit 1
    ;;
esac

ocr_actual_sha="$(shasum -a 256 "$ocr_server" | cut -d ' ' -f 1)"
if [[ "$ocr_actual_sha" != "$ocr_server_sha" ]]; then
  echo "Unexpected upstream server content; refusing to run it." >&2
  exit 1
fi

case "${1:-status}" in
  download) exec "$ocr_python" -u "$ocr_root/deploy/paddle-ocr-vl/run.py" --download-only ;;
  start) exec "$ocr_python" -u "$ocr_root/deploy/paddle-ocr-vl/run.py" ;;
esac
