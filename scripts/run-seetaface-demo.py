#!/usr/bin/env python3
"""Start a visible SeetaFace example inside the running HeraMind platform."""
import argparse
import base64
import json
from pathlib import Path
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
DEVICE = "seetaface-demo"
NAME = "SeetaFace · Engine Demo"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", default="http://127.0.0.1:9375")
    parser.add_argument("--web-url")
    parser.add_argument("--token-file", type=Path, required=True)
    args = parser.parse_args()
    token = args.token_file.read_text().strip()

    def api(method, endpoint, payload=None):
        request = urllib.request.Request(args.server.rstrip("/") + "/api" + endpoint, method=method,
            headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"},
            data=json.dumps(payload).encode() if payload is not None else None)
        with urllib.request.urlopen(request, timeout=90) as response:
            value = json.load(response)
        if value.get("success") is False:
            raise RuntimeError("HeraMind rejected " + endpoint)
        return value.get("data", value)

    def command(name, values):
        return api("POST", "/extensions/engine-bridge/command", {"command": name, "args": values})

    health = command("health", {})["data"]
    if not health.get("seetaface_compiled"):
        raise SystemExit("Install the SeetaFace build of engine-bridge first")
    try:
        existing = api("GET", "/devices/" + DEVICE)
        existing = existing.get("device", existing)
        if existing.get("name") != NAME:
            raise SystemExit("Demo device ID is already in use by another device")
    except urllib.error.HTTPError as error:
        if error.code != 404:
            raise
        api("POST", "/devices", {"device_id": DEVICE, "device_type": "rtsp-camera", "name": NAME,
            "adapter_type": "http", "connection_config": {}})
    config = api("GET", "/extensions/engine-bridge/config")["current_config"]
    bindings = config.setdefault("bindings", [])
    owned = next((b for b in bindings if b["device_id"] == DEVICE and b.get("metric", "image") == "image"), None)
    desired = {"device_id": DEVICE, "metric": "image", "task": "face.recognize", "options": {"backend": "cpu", "estimate_age": True}, "enabled": True, "min_interval_ms": 0}
    if owned != desired:
        if owned is not None:
            bindings.remove(owned)
        bindings.append(desired)
        config["timeout_ms"] = 60000
        result = api("PUT", "/extensions/engine-bridge/config", config)
        if not result.get("hot_reloaded"):
            api("POST", "/extensions/engine-bridge/reload", {})
    previous = command("get_latest", {"device_id": DEVICE})["latest"]
    fixture = ROOT / "thirdparty/engine/third_party/seetaface6/FaceRecognizer6/example/1.png"
    image = "data:image/png;base64," + base64.b64encode(fixture.read_bytes()).decode()
    api("POST", f"/devices/{DEVICE}/metrics", {"metric": "image", "value": image})
    for _ in range(60):
        latest = command("get_latest", {"device_id": DEVICE})["latest"]
        if latest and (not previous or latest["request_id"] != previous["request_id"]):
            break
        time.sleep(1)
    else:
        raise RuntimeError("No result from the demo image")
    assert len(latest["result"]["faces"]) == 1, "Expected one face in the supplied SeetaFace example"
    components = []
    for index, (title, metric) in enumerate([("Yêu cầu đã xử lý · toàn engine", "processed"),
            ("Lỗi xử lý · toàn engine", "failed"), ("Lỗi phát sự kiện · toàn engine", "publish_failed")]):
        components.append({"id": "engine-demo-" + metric, "type": "value-card", "title": title,
            "position": {"x": index * 4, "y": 0, "w": 4, "h": 2},
            "data_source": {"type": "extension", "source": "extension", "mode": "latest", "id": "engine-bridge",
                "sourceId": "engine-bridge", "extensionId": "engine-bridge", "field": "produce:" + metric,
                "extensionMetric": "produce:" + metric, "refresh": 3},
            "config": {"unit": "lượt", "showTrend": False}})
    components.extend([
        {"id": "engine-demo-face", "type": "engine-seetaface-demo", "title": "SeetaFace · Chạy mô hình qua Engine",
            "position": {"x": 0, "y": 2, "w": 8, "h": 12}, "config": {"deviceId": DEVICE}},
        {"id": "engine-demo-guide", "type": "markdown-display", "title": "Cách chạy ví dụ",
            "position": {"x": 8, "y": 2, "w": 4, "h": 12}, "config": {"content":
                "## SeetaFace đang chạy\n\nẢnh mẫu đã được đưa vào HeraMind và xử lý bằng engine thật.\n\n"
                "1. Bấm **Chạy ảnh mẫu** để nhận diện lại.\n2. Có thể **Chọn ảnh khác** (JPEG/PNG/WebP tối đa 600 KB).\n"
                "3. Muốn thử đối chiếu danh tính, nhập nhãn rồi bấm **Đăng ký ảnh này**.\n"
                "4. Dùng **Xóa đăng ký vừa tạo** để dọn nhãn thử.\n\n"
                "**Khung xanh:** khuôn mặt. **Chấm vàng:** 5 mốc căn chỉnh.\n\n"
                "Ví dụ này dùng **CPU**, chưa dùng Qualcomm NPU. Đây là ảnh mẫu, không phải camera trực tiếp.\n\n"
                "Luồng: ảnh → telemetry HeraMind → engine SeetaFace → kết quả và sự kiện engine.result.", "size": "md"}}
    ])
    dashboards = api("GET", "/dashboards")
    dashboards = dashboards.get("dashboards", []) if isinstance(dashboards, dict) else dashboards
    existing = next((d for d in dashboards if d["name"] == NAME), None)
    if existing:
        dashboard = existing
    else:
        dashboard = api("POST", "/dashboards", {"name": NAME, "description": "Ví dụ chạy SeetaFace thật qua CVEDIX engine và HeraMind",
            "layout": {"columns": 12, "rows": "auto", "breakpoints": {"lg": 1200, "md": 996, "sm": 768, "xs": 480}},
            "components": components})
    url = (args.web_url or args.server).rstrip("/") + "/visual-dashboard/" + dashboard["id"]
    report = {"dashboard_url": url, "dashboard_id": dashboard["id"], "device_id": DEVICE,
        "backend": latest["result"]["backend"], "faces": len(latest["result"]["faces"]),
        "latency_ms": latest["latency_ms"], "request_id": latest["request_id"], "source": "bundled SeetaFace sample image",
        "age_estimation_status": latest["result"]["faces"][0].get("age_estimation_status")}
    output = ROOT / ".local/seetaface-demo.json"
    output.parent.mkdir(exist_ok=True)
    output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    print(json.dumps(report, ensure_ascii=False))


if __name__ == "__main__":
    main()
