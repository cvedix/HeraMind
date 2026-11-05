#!/usr/bin/env python3
"""Create a native HeraMind dashboard using existing widgets and YOLO Video."""
import argparse
import getpass
import json
import os
from pathlib import Path
import sys
import subprocess
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent.parent


def dashboard(source, name, with_faces=False, device_id="cam2-rtsp"):
    components = []
    for index, (title, metric, unit) in enumerate([
        ("Tốc độ xử lý", "current_fps", "FPS"),
        ("Luồng đang chạy", "active_streams", "luồng"),
        ("Khung hình đã xử lý", "total_frames_processed", "khung"),
        ("Tổng lượt phát hiện", "total_detections", "lượt"),
    ]):
        components.append({
            "id": f"cam2-{metric}", "type": "value-card", "title": title,
            "position": {"x": index * 3, "y": 0, "w": 3, "h": 2},
            "data_source": {"type": "extension", "source": "extension", "mode": "latest",
                            "id": "yolo-video", "sourceId": "yolo-video", "field": f"produce:{metric}",
                            "extensionId": "yolo-video", "extensionMetric": f"produce:{metric}", "refresh": 2},
            "config": {"unit": unit, "size": "md", "showTrend": False},
        })
    components += [{
        "id": "cam2-yolo", "type": "yolo-video-card", "title": "Cam2 · YOLO Video",
        "position": {"x": 0, "y": 2, "w": 9, "h": 9},
        "config": {"sourceUrl": source, "confidenceThreshold": 0.5, "maxObjects": 50,
                   "fps": 5, "drawBoxes": True, "showStats": True, "variant": "default"},
    }, {
        "id": "cam2-guide", "type": "markdown-display", "title": "Vận hành camera",
        "position": {"x": 9, "y": 2, "w": 3, "h": 9},
        "config": {"content": (
            "## Giám sát RTSP — Cam2\n\n"
            "Bấm **Start / Bắt đầu** trong YOLO Video để chạy luồng đã cấu hình.\n\n"
            "### Người và phương tiện\n"
            "Xem nhãn phát hiện trên video. Dùng công cụ ROI để chọn vùng quan sát "
            "và lọc lớp người, xe đạp, ô tô, xe máy, xe buýt, xe tải. "
            "Vẽ đường đếm nếu cần theo dõi lượt đi qua.\n\n"
            "### Phạm vi số liệu\n"
            "Các thẻ phía trên là số liệu tổng của extension YOLO Video trên mọi luồng. "
            "Tổng lượt phát hiện có thể đếm cùng đối tượng ở nhiều khung hình; "
            "không phải số người duy nhất.\n\n"
            "Bật **Chỉnh sửa** để kéo thả, đổi kích thước và cấu hình widget."
        ), "size": "md"},
    }]
    if with_faces:
        components.append({
            "id": "cam2-faces", "type": "face-recognition-card", "title": "Cam2 · Nhận diện khuôn mặt",
            "position": {"x": 0, "y": 11, "w": 6, "h": 7},
            "data_source": {"type": "device", "source": "device", "mode": "latest",
                            "id": device_id, "sourceId": device_id, "deviceId": device_id,
                            "field": "image", "metricId": "image", "extensionId": "face-recognition"},
            "config": {"defaultCollapsed": False},
        })
        for index, (title, metric) in enumerate([
            ("Lượt nhận ra khuôn mặt", "total_recognized"),
            ("Lượt khuôn mặt chưa biết", "total_unknown"),
        ]):
            components.append({
                "id": f"cam2-face-{metric}", "type": "value-card", "title": title,
                "position": {"x": 6 + index * 3, "y": 11, "w": 3, "h": 2},
                "data_source": {"type": "extension", "source": "extension", "mode": "latest",
                                "id": "face-recognition", "sourceId": "face-recognition",
                                "extensionId": "face-recognition", "field": f"produce:{metric}",
                                "extensionMetric": f"produce:{metric}", "refresh": 2},
                "config": {"unit": "lượt", "size": "md"},
            })
    return {"name": name, "description": "Dashboard RTSP sử dụng các widget có sẵn",
            "layout": {"columns": 12, "rows": "auto",
                       "breakpoints": {"lg": 1200, "md": 996, "sm": 768, "xs": 480}},
            "components": components}


def request(server, path, token=None, payload=None, method=None):
    headers = {"Content-Type": "application/json"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = Request(server + "/api" + path, headers=headers, method=method,
                  data=json.dumps(payload).encode() if payload is not None else None)
    with urlopen(req, timeout=30) as response:
        result = json.load(response)
    return result.get("data", result)


def authenticate(server, token_file=None):
    token = token_file.read_text().strip() if token_file else os.getenv("HERAMIND_API_KEY")
    if token:
        return token
    username = input("Tài khoản HeraMind: ").strip()
    password = getpass.getpass("Mật khẩu HeraMind (không hiển thị): ")
    return request(server, "/auth/login", payload={"username": username, "password": password})["token"]


def setup_face_camera(server, token, device_id):
    if not device_id or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-" for c in device_id):
        raise ValueError("Invalid camera device ID")
    try:
        request(server, f"/devices/{device_id}", token)
    except HTTPError as error:
        if error.code != 404:
            raise
        try:
            request(server, "/device-types/rtsp-camera", token)
        except HTTPError as type_error:
            if type_error.code != 404:
                raise
            request(server, "/device-types", token, {
                "device_type": "rtsp-camera", "name": "RTSP camera",
                "store_raw": False,
                "metrics": [{"name": "image", "display_name": "Camera image", "data_type": "string", "unit": "image"}],
            })
        request(server, "/devices", token, {"device_type": "rtsp-camera", "device_id": device_id,
                "name": "Cam2 RTSP", "adapter_type": "http", "connection_config": {}})
    bindings = request(server, "/extensions/face-recognition/command", token,
                       {"command": "get_bindings", "args": {}})
    existing = next((item for item in bindings.get("bindings", []) if item["device_id"] == device_id), None)
    if existing:
        if existing.get("metric_name") != "image":
            raise ValueError("Camera is already bound to another image metric")
        if not existing.get("active"):
            result = request(server, "/extensions/face-recognition/command", token, {
                "command": "toggle_binding", "args": {"device_id": device_id, "active": True}})
            if result.get("success") is False:
                raise ValueError("Could not activate face recognition binding")
        return
    result = request(server, "/extensions/face-recognition/command", token, {
        "command": "bind_device", "args": {"device_id": device_id, "metric_name": "image"},
    })
    if result.get("success") is False:
        raise ValueError("Face Recognition could not bind the camera")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", default="http://127.0.0.1:9375")
    parser.add_argument("--name", default="Giám sát RTSP - Cam2")
    parser.add_argument("--web-url", help="Frontend origin if different from the API server")
    parser.add_argument("--rtsp-file", type=Path)
    parser.add_argument("--token-file", type=Path, help="Use an existing login token")
    parser.add_argument("--apply", action="store_true", help="Log in and create the dashboard")
    parser.add_argument("--with-faces", action="store_true", help="Add Face Recognition to the existing dashboard")
    parser.add_argument("--device-id", default="cam2-rtsp")
    parser.add_argument("--face-feed", action="store_true", help="Start RTSP image feed after adding face widgets")
    args = parser.parse_args()
    if args.face_feed and not args.with_faces:
        parser.error("--face-feed requires --with-faces")
    server = args.server.rstrip("/")
    if urlparse(server).scheme not in ("http", "https"):
        parser.error("--server must use http:// or https://")
    source_file = args.rtsp_file or ROOT / ".local/camera-smoke/source"
    source = source_file.read_text().strip() if source_file.exists() else getpass.getpass("URL RTSP: ")
    parsed = urlparse(source)
    if parsed.scheme not in ("rtsp", "rtsps") or not parsed.hostname or any(ord(c) < 32 for c in source):
        parser.error("URL RTSP không hợp lệ")
    payload = dashboard(source, args.name, args.with_faces, args.device_id)
    if not args.apply:
        output = ROOT / ".local/rtsp-dashboard.request.json"
        output.parent.mkdir(exist_ok=True)
        fd = os.open(output, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd, "w") as stream:
            json.dump(payload, stream, ensure_ascii=False, indent=2)
        output.chmod(0o600)
        print(f"Đã chuẩn bị {len(payload['components'])} widget: {output}\nChạy lại với --apply để tạo dashboard.")
        return
    token = authenticate(server, args.token_file)
    extensions = request(server, "/extensions", token)
    if not any(ext["id"] == "yolo-video" for ext in extensions):
        raise ValueError("Hãy cài extension yolo-video trước khi tạo dashboard.")
    if args.with_faces:
        if not any(ext["id"] == "face-recognition" for ext in extensions):
            raise ValueError("Hãy cài extension face-recognition trước khi thêm nhận diện khuôn mặt.")
        setup_face_camera(server, token, args.device_id)
    existing = request(server, "/dashboards", token)
    dashboards = existing.get("dashboards", []) if isinstance(existing, dict) else existing
    match = next((item for item in dashboards if item["name"] == args.name), None)
    if match:
        # Repair only the template's legacy metric bindings, preserving layout/config edits.
        for component in match.get("components", []):
            expected = next((item for item in components_with_sources(payload)
                             if item["id"] == component["id"]), None)
            old = component.get("data_source") or {}
            raw_field = expected["data_source"]["field"].split(":", 1)[-1] if expected else None
            if expected and old.get("extensionId") == "yolo-video" and old.get("field") in (None, raw_field):
                request(server, f"/dashboards/{match['id']}/components/{component['id']}", token,
                        {"set": {"data_source": expected["data_source"]}}, method="PATCH")
        print("Dashboard cùng tên đã tồn tại; giữ bố cục và cấu hình, kiểm tra liên kết số liệu.")
        if args.with_faces:
            existing_ids = {component["id"] for component in match.get("components", [])}
            extra = [component for component in payload["components"]
                     if component["id"].startswith("cam2-face") and component["id"] not in existing_ids]
            if extra:
                # Append below existing widgets; preserve all user layout edits.
                bottom = max((c.get("position", {}).get("y", 0) + c.get("position", {}).get("h", 1)
                              for c in match.get("components", [])), default=11)
                for component in extra:
                    component["position"]["y"] = max(11, bottom)
                request(server, f"/dashboards/{match['id']}/components", token, {"components": extra})
                print(f"Đã thêm {len(extra)} widget nhận diện khuôn mặt.")
        result = match
    else:
        result = request(server, "/dashboards", token, payload)
        print(f"Đã tạo dashboard với {len(payload['components'])} widget có sẵn.")
    frontend = (args.web_url or server).rstrip("/")
    print(f"Mở: {frontend}/visual-dashboard/{result['id']}")
    if args.with_faces:
        print("Chạy scripts/rtsp-face-feed.py để cấp ảnh RTSP; đăng ký ảnh khuôn mặt trong widget Face Recognition.")
    if args.face_feed:
        env = dict(os.environ, HERAMIND_API_KEY=token, HERAMIND_RTSP_URL=source)
        completed = subprocess.run([sys.executable, str(ROOT / "scripts/rtsp-face-feed.py"),
                                    "--server", server, "--device-id", args.device_id], env=env)
        if completed.returncode:
            sys.exit(completed.returncode)


def components_with_sources(payload):
    return [item for item in payload["components"] if "data_source" in item]


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nĐã dừng cấp ảnh.")
    except HTTPError as error:
        print(f"Không tạo được dashboard: HTTP {error.code}. Kiểm tra tài khoản và quyền truy cập.", file=sys.stderr)
        sys.exit(1)
    except (URLError, OSError, ValueError, KeyError) as error:
        print(f"Không tạo được dashboard: {type(error).__name__}. Kiểm tra máy chủ và cấu hình đầu vào.", file=sys.stderr)
        sys.exit(1)
