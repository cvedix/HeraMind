#!/usr/bin/env python3
"""Run the upstream PaddleOCR-VL backend for HeraMind's extension on loopback."""

import logging
import os
from threading import Lock, Thread

import uvicorn
import upstream_server as backend

LOGGER = logging.getLogger(__name__)
INFERENCE_LOCK = Lock()
predict = backend._run_predict


def predict_serially(*args, **kwargs):
    with INFERENCE_LOCK:
        return predict(*args, **kwargs)


def warm_model():
    with INFERENCE_LOCK:
        try:
            backend.get_pipeline()
        except Exception:
            LOGGER.exception("PaddleOCR-VL model initialization failed")


def main():
    backend._run_predict = predict_serially
    backend.app.add_api_route("/health", backend.health, methods=["POST"])
    if os.getenv("PADDLE_VL_WARMUP") == "1":
        Thread(target=warm_model, daemon=True).start()
    uvicorn.run(backend.app, host="127.0.0.1", port=int(os.getenv("PORT", "8000")))


if __name__ == "__main__":
    main()
