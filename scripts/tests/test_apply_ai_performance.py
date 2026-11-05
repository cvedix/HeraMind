import io
import json
from pathlib import Path
import runpy
import unittest
from unittest.mock import patch
from urllib.error import HTTPError, URLError

INSTALL = runpy.run_path(str(Path(__file__).resolve().parents[1] / 'apply-ai-performance.py'))


class BackendReadinessTests(unittest.TestCase):
    def test_waits_for_connection_and_both_extensions(self):
        wait = INSTALL['wait_ready']
        responses = [URLError('connection refused'), io.BytesIO(b'{"data": []}'),
                     io.BytesIO(json.dumps({'data': [{'id': 'clothing-colors'},
                                                    {'id': 'face-recognition'}]}).encode())]
        with patch('urllib.request.urlopen', side_effect=responses) as open_url, patch('time.sleep'):
            wait('test-token')
        self.assertEqual(open_url.call_count, 3)

    def test_auth_failure_is_not_retried(self):
        error = HTTPError('http://localhost', 401, 'Unauthorized', None, None)
        with patch('urllib.request.urlopen', side_effect=error) as open_url, patch('time.sleep'):
            with self.assertRaisesRegex(RuntimeError, 'API key'):
                INSTALL['wait_ready']('test-token')
        self.assertEqual(open_url.call_count, 1)

    def test_wait_has_deadline(self):
        with patch('time.monotonic', side_effect=[0, 3]), patch('time.sleep'):
            with self.assertRaisesRegex(RuntimeError, 'chưa sẵn sàng'):
                INSTALL['wait_ready']('test-token', timeout=2)


if __name__ == '__main__':
    unittest.main()
