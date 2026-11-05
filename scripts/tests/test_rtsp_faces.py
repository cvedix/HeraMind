import runpy
from pathlib import Path
import unittest
import tempfile
from unittest.mock import patch

SCRIPTS = Path(__file__).resolve().parents[1]
FEED = runpy.run_path(str(SCRIPTS / "rtsp-face-feed.py"))
DASHBOARD = runpy.run_path(str(SCRIPTS / "create-rtsp-dashboard.py"))


class Fragments:
    def __init__(self, chunks):
        self.chunks = iter(chunks)

    def read1(self, _size):
        return next(self.chunks, b"")


class FaceIntegrationTests(unittest.TestCase):
    def test_latest_frame_queue_drops_backlog_and_keeps_final_frame(self):
        frames = [b"\xff\xd8" + str(i).encode() + b"\xff\xd9" for i in range(100)]
        output = list(FEED["latest_frames"](Fragments([b"".join(frames)])))
        self.assertEqual(output[-1], frames[-1])
        self.assertLess(len(output), len(frames))

    def test_update_interval_accepts_zero_and_fractional_seconds(self):
        for value in (0, 0.1, 0.5, 1):
            self.assertEqual(FEED["valid_interval"](value, 1), value)
        for value in (-0.1, 1.1, float("nan"), float("inf")):
            with self.assertRaises(ValueError):
                FEED["valid_interval"](value, 1)

    def test_update_interval_retains_last_value_on_invalid_control(self):
        function = FEED["read_interval"]
        for value, expected in ((0, 0), (1, 1), (None, 0.5), (-1, 0.5), ("bad", 0.5)):
            def request(*args, **kwargs):
                return {"metrics": {"image_update_interval": {"value": value}}}
            with patch.dict(function.__globals__["HELPERS"], {"request": request}):
                self.assertEqual(function("http://localhost", "token", "camera", "image_update_interval", 0.5), expected)

    def test_jpeg_frames_split_across_pipe_reads(self):
        stream = Fragments([b"\xff", b"\xd8abc\xff", b"\xd9\xff\xd8def\xff\xd9"])
        self.assertEqual(list(FEED["jpeg_frames"](stream)),
                         [b"\xff\xd8abc\xff\xd9", b"\xff\xd8def\xff\xd9"])

    def test_truncated_frame_is_not_published(self):
        self.assertEqual(list(FEED["jpeg_frames"](Fragments([b"\xff\xd8broken"]))), [])

    def test_oversized_frame_is_rejected(self):
        with self.assertRaises(ValueError):
            list(FEED["jpeg_frames"](Fragments([b"\xff\xd8" + b"x" * 100]), maximum=32))

    def test_face_widgets_extend_native_dashboard_without_replacing_widgets(self):
        base = DASHBOARD["dashboard"]("rtsp://camera/live", "test")
        extended = DASHBOARD["dashboard"]("rtsp://camera/live", "test", True, "test-camera")
        self.assertEqual(extended["components"][:len(base["components"])], base["components"])
        ids = [c["id"] for c in extended["components"]]
        self.assertEqual(len(ids), len(set(ids)))
        face = next(c for c in extended["components"] if c["type"] == "face-recognition-card")
        self.assertEqual(face["data_source"]["deviceId"], "test-camera")
        self.assertEqual(face["data_source"]["metricId"], "image")

    def test_reapplying_face_dashboard_preserves_existing_widgets(self):
        existing = DASHBOARD["dashboard"]("rtsp://camera/live", "Giám sát RTSP - Cam2", True)
        existing["id"] = "existing-dashboard"
        mutations = []

        def api(_server, path, _token=None, payload=None, method=None):
            if payload is not None and path != "/extensions/face-recognition/command":
                mutations.append((path, payload))
            if path == "/extensions":
                return [{"id": "yolo-video"}, {"id": "face-recognition"}]
            if path == "/dashboards":
                return {"dashboards": [existing]}
            if path == "/extensions/face-recognition/command":
                return {"bindings": [{"device_id": "cam2-rtsp", "metric_name": "image", "active": True}]}
            return {}

        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source"
            source.write_text("rtsp://camera/live")
            token = Path(directory) / "token"
            token.write_text("test-token")
            arguments = ["create-rtsp-dashboard.py", "--apply", "--with-faces",
                         "--rtsp-file", str(source), "--token-file", str(token)]
            with patch("sys.argv", arguments), patch.dict(DASHBOARD["main"].__globals__, {"request": api}):
                DASHBOARD["main"]()
        self.assertEqual(mutations, [])


if __name__ == "__main__":
    unittest.main()
