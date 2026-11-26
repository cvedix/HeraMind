#!/usr/bin/env python3
"""Run the upstream PaddleOCR-VL backend for HeraMind's extension on loopback."""

import gc
import logging
import os
from threading import Lock, Thread

import uvicorn
import upstream_server as backend

LOGGER = logging.getLogger(__name__)
INFERENCE_LOCK = Lock()
predict = backend._run_predict
load_pipeline = backend.get_pipeline


def get_pipeline(
    use_doc_orientation_classify=False,
    use_doc_unwarping=False,
):
    key = (bool(use_doc_orientation_classify), bool(use_doc_unwarping))
    cache = backend._PIPELINE_CACHE
    if cache and key not in cache and key not in backend._PIPELINE_ERRORS:
        # Upstream evicts after loading. Release the previous model first so
        # switching preprocessing options only requires memory for one pipeline.
        LOGGER.info("Releasing PaddleOCR-VL pipeline before loading %s", key)
        _, pipeline = cache.popitem()
        pipeline.close()
        del pipeline
        gc.collect()
        if os.getenv("PADDLE_DEVICE", "cpu").startswith("gpu"):
            from paddle.device.cuda import empty_cache

            empty_cache()
    return load_pipeline(*key)


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
    backend.get_pipeline = get_pipeline
    backend._run_predict = predict_serially
    backend.app.add_api_route("/health", backend.health, methods=["POST"])
    if os.getenv("PADDLE_VL_WARMUP") == "1":
        Thread(target=warm_model, daemon=True).start()
    uvicorn.run(backend.app, host="127.0.0.1", port=int(os.getenv("PORT", "8000")))


if __name__ == "__main__":
    main()
