from pathlib import Path
import runpy
import unittest
from unittest.mock import patch

MODULE = runpy.run_path(str(Path(__file__).resolve().parents[1] / "create-processing-dashboards.py"))


class ProcessingDashboardTests(unittest.TestCase):
    def run_apply(self, api, only="all", setup=None):
        common = dict(MODULE["COMMON"], request=api, authenticate=lambda *_: "test-token",
                      setup_face_camera=setup or (lambda *_: None))
        with patch.dict(MODULE["main"].__globals__, {"COMMON": common}), \
                patch("sys.argv", ["create-processing-dashboards.py", "--apply", "--only", only]):
            MODULE["main"]()

    def test_existing_dashboards_are_not_overwritten(self):
        existing = [{"name": "Nhận diện khuôn mặt - Cam2", "id": "face"},
                    {"name": "OCR tài liệu", "id": "ocr"}]
        mutations = []

        def api(_server, path, _token, payload=None):
            if payload is not None:
                mutations.append(payload)
            return ([{"id": "face-recognition"}, {"id": "paddle-ocr-v6"}]
                    if path == "/extensions" else {"dashboards": existing})

        self.run_apply(api)
        self.assertEqual(mutations, [])

    def test_ocr_only_does_not_require_face_extension_or_camera(self):
        created = []

        def api(_server, path, _token, payload=None):
            if path == "/extensions":
                return [{"id": "paddle-ocr-v6"}]
            if payload is None:
                return {"dashboards": []}
            created.append(payload)
            return {"id": "new-ocr"}

        self.run_apply(api, "ocr", setup=lambda *_: self.fail("OCR must not configure a camera"))
        self.assertEqual(len(created), 1)
        self.assertIn("paddle-ocr-v6-card", [c["type"] for c in created[0]["components"]])

    def test_missing_extension_fails_before_writing_dashboards(self):
        def api(_server, path, _token, payload=None):
            self.assertEqual(path, "/extensions")
            self.assertIsNone(payload)
            return []

        with self.assertRaisesRegex(ValueError, "Thiếu extension"):
            self.run_apply(api)
