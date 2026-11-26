"""Black-box tests for the real engine worker; no Python imaging dependencies."""
import base64
import json
import os
import selectors
import struct
import subprocess
import unittest
import zlib


def png(width=2, height=3):
    def chunk(kind, data):
        return struct.pack("!I", len(data)) + kind + data + struct.pack("!I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    return base64.b64encode(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack("!IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress((b"\x00" + b"\xff\x00\x00" * width) * height))
        + chunk(b"IEND", b"")
    ).decode()


class WorkerTests(unittest.TestCase):
    def setUp(self):
        self.worker = subprocess.Popen(
            [os.environ["HERAMIND_TEST_ENGINE_WORKER"]],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            env={**os.environ, "OPENCV_IO_MAX_IMAGE_PIXELS": "16777216"},
        )
        self.sequence = 0

    def tearDown(self):
        if not self.worker.stdin.closed:
            self.worker.stdin.close()
        if self.worker.poll() is None:
            try:
                self.worker.wait(timeout=3)
            except subprocess.TimeoutExpired:
                self.worker.kill()
                self.worker.wait(timeout=3)
        self.worker.stdout.close()

    def read_exact(self, length):
        # Tests must fail rather than hang if the pipeline stops replying.
        data = b""
        with selectors.DefaultSelector() as selector:
            selector.register(self.worker.stdout, selectors.EVENT_READ)
            while len(data) < length:
                self.assertTrue(selector.select(5), "Worker response timed out")
                part = os.read(self.worker.stdout.fileno(), length - len(data))
                self.assertTrue(part, "Worker exited while reading a response")
                data += part
        return data

    def call(self, task, args=None):
        self.sequence += 1
        request_id = f"test-camera-{self.sequence}"
        request = json.dumps({"protocol": 1, "request_id": request_id, "task": task, "args": args or {}}).encode()
        self.worker.stdin.write(struct.pack("!I", len(request)) + request)
        self.worker.stdin.flush()
        length, = struct.unpack("!I", self.read_exact(4))
        self.assertLessEqual(length, 2 * 1024 * 1024)
        result = json.loads(self.read_exact(length))
        self.assertEqual(result["request_id"], request_id)
        return result

    def test_actual_core_handles_different_frame_dimensions(self):
        for width, height in [(2, 3), (32, 18), (1, 1), (2, 3)]:
            result = self.call("image.inspect", {"image_base64": png(width, height)})
            self.assertTrue(result["ok"], result)
            self.assertEqual(result["data"]["width"], width)
            self.assertEqual(result["data"]["height"], height)
            self.assertEqual(result["data"]["mean_bgr"], [0.0, 0.0, 255.0])

    def test_preprocessing_and_validation_leave_worker_usable(self):
        args = {"image_base64": "data:image/png;base64," + png(),
                "options": {"width": 16, "height": 12, "format": "gray"}}
        result = self.call("image.preprocess", args)
        self.assertTrue(result["ok"], result)
        self.assertEqual(result["data"]["output_channels"], 1)
        jpeg = base64.b64decode(result["data"]["image_base64"].split(",", 1)[1])
        self.assertTrue(jpeg.startswith(b"\xff\xd8"))
        self.assertTrue(jpeg.endswith(b"\xff\xd9"))
        for options in [{"width": 0}, {"width": 2049}, {"format": "unknown"}, {"backend": "qnn"}]:
            self.assertFalse(self.call("image.preprocess", {"image_base64": png(), "options": options})["ok"])
        self.assertTrue(self.call("health")["ok"])

    def test_rejects_invalid_inputs(self):
        for value in ["", "AAAA!AAA", "AA=A", "====", "data:text/plain;base64,AAAA", "aGVsbG8="]:
            self.assertFalse(self.call("image.inspect", {"image_base64": value})["ok"])
        self.assertFalse(self.call("unknown.task")["ok"])
        self.assertTrue(self.call("image.inspect", {"image_base64": png()})["ok"])

    def test_no_silent_npu_fallback(self):
        health = self.call("health")["data"]
        self.assertFalse(health["qnn_available"])
        self.assertFalse(self.call("image.inspect", {"image_base64": png(), "options": {"backend": "qnn"}})["ok"])
        if not health["seetaface_compiled"]:
            response = self.call("face.recognize", {"image_base64": png()})
            self.assertFalse(response["ok"])
            self.assertIn("SeetaFace was not built", response["error"])

    def test_oversized_frame_header_exits_without_reading_payload(self):
        self.worker.stdin.write(struct.pack("!I", 8 * 1024 * 1024 + 1))
        self.worker.stdin.flush()
        self.assertEqual(self.worker.wait(timeout=3), 3)


if __name__ == "__main__":
    unittest.main()
