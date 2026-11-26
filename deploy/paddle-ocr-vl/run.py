"""Preload the official PaddleOCR-VL service before accepting OCR requests."""

import importlib.util
import os
from pathlib import Path
import sys


def main() -> None:
    data_dir = Path(os.environ["HERAMIND_OCR_DATA_DIR"])
    source = data_dir / "upstream" / "server.py"
    spec = importlib.util.spec_from_file_location("heramind_paddle_ocr_server", source)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Cannot load PaddleOCR-VL service: {source}")
    service = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = service
    spec.loader.exec_module(service)

    print("Loading PaddleOCR-VL 1.6 on CPU; model cache:", os.environ["PADDLE_PDX_CACHE_HOME"], flush=True)
    service.get_pipeline(False, False)
    if "--download-only" in sys.argv:
        print("PaddleOCR-VL model cache is ready.", flush=True)
        return

    import uvicorn

    uvicorn.run(service.app, host="127.0.0.1", port=int(os.environ.get("HERAMIND_OCR_PORT", "8000")))


if __name__ == "__main__":
    main()
