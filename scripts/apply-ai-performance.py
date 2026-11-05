#!/usr/bin/env python3
"""Install prepared performance updates after restarting a backend with a stuck extension queue."""
import base64
import json
from pathlib import Path
import runpy
import subprocess
import time
from urllib.error import HTTPError, URLError
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parent.parent
HELPERS = runpy.run_path(str(ROOT / 'scripts/create-rtsp-dashboard.py'))


def wait_ready(token, timeout=120):
    deadline = time.monotonic() + timeout
    print('Đang chờ backend và extensions sẵn sàng…', flush=True)
    while time.monotonic() < deadline:
        request = urllib.request.Request('http://127.0.0.1:9375/api/extensions',
                                         headers={'Authorization': 'Bearer ' + token})
        try:
            with urllib.request.urlopen(request, timeout=5) as response:
                body = json.load(response)
            extensions = body.get('data', body)
            if isinstance(extensions, list) and {'clothing-colors', 'face-recognition'} <= {e['id'] for e in extensions}:
                return
        except HTTPError as error:
            if error.code in (401, 403):
                raise RuntimeError('API key không hợp lệ hoặc không đủ quyền') from None
        except (URLError, OSError, ValueError):
            pass
        time.sleep(1)
    raise RuntimeError('Backend chưa sẵn sàng sau 120 giây; kiểm tra systemctl status heramind')


def main():
    token = (ROOT / '.local/usb-face-feed/token').read_text().strip()
    wait_ready(token)
    subprocess.run(['systemctl', '--user', 'stop', 'heramind-usb-face.service',
                    'heramind-rtsp-clothing.service'], check=True)
    for extension in ['clothing-colors', 'face-recognition']:
        installed = Path('/var/lib/heramind/extensions') / extension
        package = ROOT / '.local' / (extension + '-performance.nep')
        with zipfile.ZipFile(package, 'w', zipfile.ZIP_DEFLATED) as archive:
            archive.write(installed / 'manifest.json', 'manifest.json')
            binary = ROOT / 'extensions/clothing-colors/target/release/libheramind_clothing_colors.so' if extension == 'clothing-colors' else installed / 'binaries/linux_arm64/extension.so'
            archive.write(binary, 'binaries/linux_arm64/extension.so')
            frontend = 'clothing-colors.umd.cjs' if extension == 'clothing-colors' else 'face-recognition-components.umd.cjs'
            archive.write(ROOT / 'extensions' / extension / 'frontend' / frontend, 'frontend/' + frontend)
            if extension == 'clothing-colors':
                archive.write(ROOT / 'extensions/clothing-colors/worker/analyze.py', 'assets/worker/analyze.py')
        payload = json.dumps({'filename': package.name, 'data': base64.b64encode(package.read_bytes()).decode()}).encode()
        request = urllib.request.Request('http://127.0.0.1:9375/api/extensions/upload/file', data=payload,
            headers={'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json'})
        with urllib.request.urlopen(request, timeout=45) as response:
            result = json.load(response)
        if result.get('success') is False:
            raise RuntimeError('Extension installation failed')
        print('Installed:', extension, flush=True)
    HELPERS['request']('http://127.0.0.1:9375', '/devices/cam2-rtsp/metrics', token,
                       {'metric': 'image_update_interval', 'value': 1})
    subprocess.run(['systemctl', '--user', 'daemon-reload'], check=True)
    subprocess.run(['systemctl', '--user', 'restart', 'heramind-usb-face.service',
                    'heramind-rtsp-clothing-feed.service', 'heramind-rtsp-clothing.service'], check=True)


if __name__ == '__main__':
    main()
