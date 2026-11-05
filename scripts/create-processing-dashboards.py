#!/usr/bin/env python3
"""Create native Face Recognition and document OCR dashboards in HeraMind."""
import argparse
import json
import os
from pathlib import Path
import runpy
import sys
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse

COMMON = runpy.run_path(str(Path(__file__).with_name("create-rtsp-dashboard.py")))
LAYOUT = {"columns": 12, "rows": "auto",
          "breakpoints": {"lg": 1200, "md": 996, "sm": 768, "xs": 480}}


def metric_card(identifier, title, extension, metric, x, width):
    return {"id": identifier, "type": "value-card", "title": title,
            "position": {"x": x, "y": 0, "w": width, "h": 2},
            "data_source": {"type": "extension", "source": "extension", "mode": "latest",
                            "id": extension, "sourceId": extension, "extensionId": extension,
                            "field": f"produce:{metric}", "extensionMetric": f"produce:{metric}", "refresh": 2},
            "config": {"size": "md", "showTrend": False}}


def face_dashboard(device_id):
    # Reuse the existing face widget binding schema, not a custom component.
    combined = COMMON["dashboard"]("rtsp://unused", "unused", True, device_id)
    face = next(c for c in combined["components"] if c["type"] == "face-recognition-card")
    face["id"] = "face-camera"
    face["position"] = {"x": 0, "y": 2, "w": 8, "h": 8}
    components = [metric_card(f"face-{metric}", title, "face-recognition", metric, index * 4, 4)
                  for index, (title, metric) in enumerate([
                      ("Lượt xử lý ảnh", "total_inferences"),
                      ("Lượt nhận ra khuôn mặt", "total_recognized"),
                      ("Lượt khuôn mặt chưa biết", "total_unknown")])]
    components += [face, {"id": "face-guide", "type": "markdown-display", "title": "Đăng ký và nhận diện",
        "position": {"x": 8, "y": 2, "w": 4, "h": 8}, "config": {"content": (
            "## Nhận diện khuôn mặt Cam2\n\n"
            "1. Giữ bộ cấp ảnh `rtsp-face-feed.py` chạy để camera gửi ảnh vào thiết bị.\n"
            "2. Trong Face Recognition, đăng ký ảnh có một khuôn mặt rõ nét và nhập tên.\n"
            "3. Theo dõi kết quả khớp tên hoặc khuôn mặt chưa biết; có thể xóa ảnh mẫu và bật/tắt binding trong widget.\n\n"
            f"**Thiết bị:** `{device_id}` · **Metric ảnh:** `image`.\n\n"
            "Các thẻ trên là tổng lượt của extension trên mọi thiết bị, cập nhật theo chu kỳ metric của backend. "
            "Cùng một người có thể xuất hiện ở nhiều lượt; không phải số người duy nhất."
        ), "size": "md"}}]
    return {"name": "Nhận diện khuôn mặt - Cam2", "description": "Face Recognition trên ảnh thiết bị camera",
            "layout": LAYOUT, "components": components}


def ocr_dashboard():
    return {"name": "OCR tài liệu", "description": "Trích xuất văn bản từ ảnh tài liệu bằng PaddleOCR v6",
            "layout": LAYOUT, "components": [
                metric_card("ocr-model", "Mô hình đã nạp", "paddle-ocr-v6", "loaded", 0, 6),
                metric_card("ocr-tier", "Phiên bản mô hình đang dùng", "paddle-ocr-v6", "tier", 6, 6),
                {"id": "document-ocr", "type": "paddle-ocr-v6-card", "title": "OCR · Ảnh tài liệu",
                 "position": {"x": 0, "y": 2, "w": 8, "h": 11},
                 "config": {"drawBoxes": True, "showConfidence": True, "tier": "tiny"}},
                {"id": "ocr-guide", "type": "markdown-display", "title": "Xử lý tài liệu",
                 "position": {"x": 8, "y": 2, "w": 4, "h": 11}, "config": {"content": (
                     "## OCR tài liệu\n\n"
                     "1. Kéo thả, dán hoặc tải ảnh JPG/PNG/WebP vào widget PaddleOCR.\n"
                     "2. Bấm **Recognize** để đọc chữ.\n"
                     "3. Xem vùng chữ, độ tin cậy và toàn bộ văn bản; dùng nút sao chép kết quả trong widget.\n\n"
                     "**Ảnh đầu vào:** chụp thẳng, đủ sáng, chữ rõ nét. Với PDF, xuất mỗi trang thành ảnh trước khi tải lên. "
                     "Widget hiện xử lý từng ảnh, không nhập PDF trực tiếp hay xử lý hàng loạt.\n\n"
                     "Mặc định dùng tier **tiny** đã có trên máy. Các tier khác có thể cần tải mô hình. "
                     "Tier tiny hiện chưa có đủ bộ ký tự tiếng Việt có dấu; không bảo đảm đọc đúng toàn bộ dấu. "
                     "Kiểm tra lại văn bản và số quan trọng trong kết quả.\n\n"
                     "Các thẻ mô hình phản ánh trạng thái chung của extension, không phải tiến độ từng tài liệu."
                 ), "size": "md"}},
            ]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", default="http://127.0.0.1:9375")
    parser.add_argument("--web-url", help="Frontend origin, e.g. http://192.168.1.110:5173")
    parser.add_argument("--token-file", type=Path)
    parser.add_argument("--device-id", default="cam2-rtsp")
    parser.add_argument("--only", choices=("face", "ocr", "all"), default="all")
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    server = args.server.rstrip("/")
    if urlparse(server).scheme not in ("http", "https"):
        parser.error("Invalid server URL")
    if not args.device_id or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-" for c in args.device_id):
        parser.error("Invalid camera device ID")
    plans = []
    if args.only in ("face", "all"):
        plans.append(("face-recognition", face_dashboard(args.device_id)))
    if args.only in ("ocr", "all"):
        plans.append(("paddle-ocr-v6", ocr_dashboard()))
    if not args.apply:
        output = COMMON["ROOT"] / ".local/processing-dashboards.json"
        output.parent.mkdir(exist_ok=True)
        fd = os.open(output, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd, "w") as stream:
            json.dump([plan for _, plan in plans], stream, ensure_ascii=False, indent=2)
        print(f"Đã chuẩn bị {len(plans)} dashboard: {output}\nThêm --apply để đăng nhập và tạo dashboard.")
        return
    token = COMMON["authenticate"](server, args.token_file)
    request = COMMON["request"]
    available = {extension["id"] for extension in request(server, "/extensions", token)}
    missing = [extension for extension, _ in plans if extension not in available]
    if missing:
        raise ValueError("Thiếu extension: " + ", ".join(missing))
    if args.only in ("face", "all"):
        COMMON["setup_face_camera"](server, token, args.device_id)
    response = request(server, "/dashboards", token)
    existing = response.get("dashboards", []) if isinstance(response, dict) else response
    frontend = (args.web_url or server).rstrip("/")
    for _, plan in plans:
        saved = next((item for item in existing if item["name"] == plan["name"]), None)
        if saved is None:
            saved = request(server, "/dashboards", token, plan)
            existing.append(saved)
            print("Đã tạo:", plan["name"])
        else:
            print("Đã có, giữ nguyên cấu hình:", plan["name"])
        print(f"{frontend}/visual-dashboard/{saved['id']}")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nĐã dừng.")
    except HTTPError as error:
        print(f"Lỗi API HTTP {error.code}. Kiểm tra tài khoản và quyền truy cập.", file=sys.stderr)
        sys.exit(1)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
    except (URLError, OSError, KeyError) as error:
        print(f"Không tạo được dashboard: {type(error).__name__}. Kiểm tra máy chủ và extension.", file=sys.stderr)
        sys.exit(1)
