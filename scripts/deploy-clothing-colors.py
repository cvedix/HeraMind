#!/usr/bin/env python3
"""Install Clothing Colors .nep and add its YOLO/SCHP widget to a native dashboard."""
import argparse
import base64
import getpass
import json
from pathlib import Path
import runpy
import sys
import urllib.request
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlparse

COMMON = runpy.run_path(str(Path(__file__).with_name("create-rtsp-dashboard.py")))
ROOT = COMMON["ROOT"]


def components(source, top=0):
    return [{"id": "cam2-clothing-colors", "type": "yolo-clothing-colors", "title": "Cam2 · Màu áo và quần",
             "position": {"x": 0, "y": top, "w": 9, "h": 12},
             "config": {"sourceUrl": source, "maxPeople": 2, "analysisIntervalMs": 2500}},
            {"id": "cam2-clothing-guide", "type": "markdown-display", "title": "Phân tích trang phục",
             "position": {"x": 9, "y": top, "w": 3, "h": 12}, "config": {"content": (
                 "## YOLO + SCHP-ATR\n\nBấm **Bắt đầu** để mở Cam2. YOLO tìm người, SCHP tách vùng áo/quần, "
                 "HSV/Lab gom màu từ pixel trong từng vùng.\n\n"
                 "Mặc định tối đa 2 người mỗi ảnh, chu kỳ ít nhất 2,5 giây. Đây là ảnh đã phân tích, "
                 "không phải video 25 FPS. Nhãn và hình luôn thuộc cùng một khung.\n\n"
                 "Không nhìn thấy quần, vùng quá nhỏ, bị che hoặc ảnh gần như xám/hồng ngoại sẽ có thể trả **Không rõ**. "
                 "Tỷ lệ phần trăm là phần diện tích màu, không phải xác suất đúng.\n\n"
                 "Số thứ tự người chỉ có ý nghĩa trong khung hình; chưa có tracking danh tính. "
                 "Ánh sáng và cân bằng trắng có thể làm lệch màu.\n\n"
                 "Có thể thêm widget này vào dashboard khác qua thư viện component."
             ), "size": "md"}}]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", default="http://127.0.0.1:9375")
    parser.add_argument("--web-url")
    parser.add_argument("--token-file", type=Path)
    parser.add_argument("--rtsp-file", type=Path)
    parser.add_argument("--package", type=Path, default=ROOT / ".local/clothing-colors-0.1.0-linux_arm64.nep")
    parser.add_argument("--dashboard-id", help="Append to this existing dashboard instead of creating a new tab")
    parser.add_argument("--reinstall", action="store_true", help="Replace an installed clothing-colors package")
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    server = args.server.rstrip("/")
    if urlparse(server).scheme not in ("http", "https"):
        parser.error("Invalid server URL")
    if not args.package.is_file():
        parser.error("Build the package first with scripts/package-clothing-colors.py")
    if not args.apply:
        print(f"Gói cài: {args.package}\nThêm --apply để đăng nhập, cài extension và tạo widget dashboard.")
        return
    source_file = args.rtsp_file or ROOT / ".local/camera-smoke/source"
    source = source_file.read_text().strip() if source_file.exists() else getpass.getpass("URL RTSP: ")
    parsed = urlparse(source)
    if parsed.scheme not in ("rtsp", "rtsps") or not parsed.hostname or any(ord(c) < 32 for c in source):
        parser.error("Invalid RTSP URL")
    token = COMMON["authenticate"](server, args.token_file)
    api = COMMON["request"]
    extensions = api(server, "/extensions", token)
    if not any(ext["id"] == "yolo-video" for ext in extensions):
        raise ValueError("Cần cài và chạy extension yolo-video trước.")
    installed = any(ext["id"] == "clothing-colors" for ext in extensions)
    if not installed or args.reinstall:
        print("Đang tải gói model và runtime lên HeraMind…", flush=True)
        payload = json.dumps({"filename": args.package.name,
                              "data": base64.b64encode(args.package.read_bytes()).decode()}).encode()
        req = urllib.request.Request(server + "/api/extensions/upload/file", data=payload,
                                     headers={"Content-Type": "application/json", "Authorization": "Bearer " + token})
        with urllib.request.urlopen(req, timeout=240) as response:
            result = json.load(response)
        if result.get("success") is False:
            raise ValueError("HeraMind từ chối gói extension")
        print("Đã cài extension Clothing Colors.", flush=True)
    health = api(server, "/extensions/clothing-colors/command", token, {"command": "health", "args": {}})
    if health.get("ready") is not True:
        raise ValueError("Model SCHP chưa sẵn sàng")
    if args.dashboard_id:
        saved = api(server, "/dashboards/" + quote(args.dashboard_id, safe=""), token)
        if not any(c["type"] == "yolo-clothing-colors" for c in saved.get("components", [])):
            bottom = max((c.get("position", {}).get("y", 0) + c.get("position", {}).get("h", 1)
                          for c in saved.get("components", [])), default=0)
            api(server, f"/dashboards/{quote(saved['id'], safe='')}/components", token,
                {"components": components(source, bottom)})
    else:
        response = api(server, "/dashboards", token)
        dashboards = response.get("dashboards", []) if isinstance(response, dict) else response
        name = "Màu áo và quần - Cam2"
        saved = next((item for item in dashboards if item["name"] == name), None)
        if saved is None:
            saved = api(server, "/dashboards", token, {"name": name,
                "description": "YOLO person detection + SCHP ATR + HSV/Lab clothing colors",
                "layout": {"columns": 12, "rows": "auto", "breakpoints": {"lg": 1200, "md": 996, "sm": 768, "xs": 480}},
                "components": components(source)})
    print("Model sẵn sàng. Mở dashboard và bấm Bắt đầu:")
    print((args.web_url or server).rstrip("/") + "/visual-dashboard/" + saved["id"])


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nĐã dừng.")
    except HTTPError as error:
        print(f"Lỗi HTTP {error.code}. Kiểm tra đăng nhập, quyền cài extension và trạng thái máy chủ.", file=sys.stderr)
        sys.exit(1)
    except (URLError, OSError, ValueError, KeyError) as error:
        print(f"Triển khai thất bại ({type(error).__name__}). Kiểm tra log extension/model.", file=sys.stderr)
        sys.exit(1)
