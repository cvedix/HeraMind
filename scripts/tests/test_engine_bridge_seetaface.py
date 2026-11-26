#!/usr/bin/env python3
"""Run real SeetaFace detection/enrollment/persistence/deletion in a disposable DB."""
import argparse
import base64
import json
import os
from pathlib import Path
import selectors
import struct
import subprocess
import tempfile
import time
import zlib


def blank_image(width, height):
    def chunk(kind, data):
        return struct.pack("!I", len(data)) + kind + data + struct.pack("!I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    return base64.b64encode(b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack("!IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress((b"\0" + b"\0\0\0" * width) * height))
        + chunk(b"IEND", b"")).decode()


class Worker:
    def __init__(self, package, workdir):
        self.sequence = 0
        self.log_path = workdir / "worker.log"
        self.log = self.log_path.open("ab")
        self.proc = subprocess.Popen([str(package / "assets/engine-worker")], cwd=workdir,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.log,
            env={**os.environ, "LD_LIBRARY_PATH": str(package / "assets/lib") + ":" + os.environ.get("LD_LIBRARY_PATH", "")})

    def read_exact(self, length):
        data = b""
        with selectors.DefaultSelector() as selector:
            selector.register(self.proc.stdout, selectors.EVENT_READ)
            while len(data) < length:
                if not selector.select(60):
                    raise RuntimeError("SeetaFace inference timed out")
                part = os.read(self.proc.stdout.fileno(), length - len(data))
                if not part:
                    raise RuntimeError(f"SeetaFace worker exited: {self.log_path.read_text(errors='replace')[-2000:]}")
                data += part
        return data

    def call(self, task, args, expected_error=None):
        self.sequence += 1
        request_id = f"seetaface-smoke-{self.sequence}"
        body = json.dumps({"protocol": 1, "request_id": request_id, "task": task, "args": args}).encode()
        self.proc.stdin.write(struct.pack("!I", len(body)) + body)
        self.proc.stdin.flush()
        size, = struct.unpack("!I", self.read_exact(4))
        assert 0 < size < 2 * 1024 * 1024, size
        response = json.loads(self.read_exact(size))
        assert response["request_id"] == request_id, response
        if expected_error is not None:
            assert not response["ok"] and expected_error in response.get("error", ""), response
            return response["error"]
        assert response["ok"], response
        return response["data"]

    def close(self):
        self.proc.stdin.close()
        try:
            self.proc.wait(timeout=10)
        finally:
            if self.proc.poll() is None:
                self.proc.kill()
                self.proc.wait(timeout=5)
            self.proc.stdout.close()
            self.log.close()
        assert self.proc.returncode == 0, self.proc.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--extension-root", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    package = args.extension_root.resolve()
    image = base64.b64encode(args.fixture.read_bytes()).decode()
    label = "Kiểm thử SeetaFace"
    report = {"backend": "cpu", "fixture": str(args.fixture), "model": "seetaface6"}
    with tempfile.TemporaryDirectory(prefix="heramind-seetaface-smoke-") as temp:
        workdir = Path(temp)
        (workdir / "models").symlink_to(package / "models", target_is_directory=True)
        worker = Worker(package, workdir)
        try:
            capabilities = worker.call("health", {})
            assert capabilities["seetaface_compiled"]
            assert capabilities["age_estimation"]["model_available"]
            start = time.perf_counter()
            initial = worker.call("face.recognize", {"image_base64": image})
            report["first_inference_ms"] = (time.perf_counter() - start) * 1000
            assert len(initial["faces"]) == 1, initial
            assert "age_estimate" not in initial["faces"][0], "Age inference should be opt-in"
            aged = worker.call("face.recognize", {"image_base64": image, "options": {"estimate_age": True}})
            assert aged["age_estimation_enabled"] is True
            assert aged["faces"][0]["age_estimation_status"] == "estimated", aged
            assert isinstance(aged["faces"][0]["age_estimate"], int) and aged["faces"][0]["age_estimate"] >= 0, aged
            report["age_estimation"] = "passed (model estimate, not verified age)"
            worker.call("face.recognize", {"image_base64": blank_image(1, 1)}, expected_error="at least 40x40")
            assert worker.call("face.recognize", {"image_base64": blank_image(64, 64), "options": {"estimate_age": True}})["faces"] == []
            registered = worker.call("face.register", {"image_base64": image, "name": label})
            start = time.perf_counter()
            matched = worker.call("face.recognize", {"image_base64": image})
            report["warm_inference_ms"] = (time.perf_counter() - start) * 1000
            assert matched["faces"][0]["identity"] == label, matched
            report["self_similarity"] = matched["faces"][0]["similarity"]
        finally:
            worker.close()
        worker = Worker(package, workdir)
        try:
            restored = worker.call("face.recognize", {"image_base64": image})
            assert restored["faces"][0]["identity"] == label, restored
            worker.call("face.delete", {"face_id": registered["face_id"]})
            deleted = worker.call("face.recognize", {"image_base64": image})
            assert deleted["faces"][0]["identity"] in ("", "Unknown"), deleted
        finally:
            worker.close()
        worker = Worker(package, workdir)
        try:
            deleted_persisted = worker.call("face.recognize", {"image_base64": image})
            assert deleted_persisted["faces"][0]["identity"] in ("", "Unknown"), deleted_persisted
        finally:
            worker.close()
    with tempfile.TemporaryDirectory(prefix="heramind-missing-age-") as temp:
        workdir = Path(temp)
        models = workdir / "models/seetaface6"
        models.mkdir(parents=True)
        for source in (package / "models/seetaface6").glob("*.csta"):
            if source.name != "age_predictor.csta":
                (models / source.name).symlink_to(source)
        worker = Worker(package, workdir)
        try:
            assert not worker.call("health", {})["age_estimation"]["model_available"]
            worker.call("face.recognize", {"image_base64": image, "options": {"estimate_age": True}},
                expected_error="Missing SeetaFace age model")
            assert len(worker.call("face.recognize", {"image_base64": image})["faces"]) == 1
        finally:
            worker.close()
    report["missing_age_model"] = "explicit error; normal recognition remains usable"
    report["checks"] = ["face detection", "registration", "matching", "UTF-8 name persistence", "deletion", "deletion persistence", "age estimate", "no face", "missing age model"]
    if args.report:
        args.report.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(report, ensure_ascii=False))


if __name__ == "__main__":
    main()
