#!/usr/bin/env python3
"""Feed sampled RTSP JPEGs to the native Face Recognition device binding."""
import argparse
import base64
import getpass
import math
import queue
import threading
import os
from pathlib import Path
import runpy
import subprocess
import sys
import time
from urllib.error import HTTPError, URLError
from urllib.parse import urlparse

HELPERS = runpy.run_path(str(Path(__file__).with_name("create-rtsp-dashboard.py")))


def jpeg_frames(stream, maximum=4 * 1024 * 1024):
    pending = bytearray()
    while True:
        chunk = stream.read1(16384)
        if not chunk:
            return
        pending.extend(chunk)
        if len(pending) > maximum:
            raise ValueError("Camera frame exceeds 4 MB")
        while True:
            start = pending.find(b"\xff\xd8")
            end = pending.find(b"\xff\xd9", start + 2) if start >= 0 else -1
            if end < 0:
                break
            yield bytes(pending[start:end + 2])
            del pending[:end + 2]


def latest_frames(stream):
    """Drain the pipe continuously; never queue more than the newest JPEG."""
    frames = queue.Queue(maxsize=1)
    done = threading.Event()
    errors = []

    def read():
        try:
            for frame in jpeg_frames(stream):
                try:
                    frames.get_nowait()
                except queue.Empty:
                    pass
                frames.put_nowait(frame)
        except (OSError, ValueError) as error:
            errors.append(error)
        finally:
            done.set()

    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    while not done.is_set() or not frames.empty():
        try:
            yield frames.get(timeout=0.5)
        except queue.Empty:
            continue
    if errors:
        raise errors[0]


def valid_interval(value, maximum=300):
    value = float(value)
    if not math.isfinite(value) or not 0 <= value <= maximum:
        raise ValueError("Invalid update interval")
    return value


def read_interval(server, token, device_id, metric, fallback):
    try:
        current = HELPERS["request"](server, f"/devices/{device_id}/current", token)
        value = current.get("metrics", {}).get(metric, {}).get("value")
        return fallback if value is None else valid_interval(value, 1)
    except (HTTPError, URLError, OSError, ValueError, TypeError):
        return fallback


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", default="http://127.0.0.1:9375")
    parser.add_argument("--device-id", default="cam2-rtsp")
    parser.add_argument("--rtsp-file", type=Path)
    parser.add_argument("--usb-device", type=Path, help="USB webcam, preferably /dev/v4l/by-id/...-video-index0")
    parser.add_argument("--token-file", type=Path)
    parser.add_argument("--interval", type=float, default=5, help="Seconds between sampled frames; 0 means fastest (up to 10 fps)")
    parser.add_argument("--minimum-interval", type=float, default=0.1, help="Minimum sending interval to protect inference capacity")
    parser.add_argument("--interval-metric", help="Device metric controlling USB interval (0 to 1 seconds)")
    parser.add_argument("--skip-face-binding", action="store_true", help="Feed an existing camera without enabling face recognition")
    parser.add_argument("--once", action="store_true", help="Send one frame and exit")
    args = parser.parse_args()
    try:
        valid_interval(args.interval)
        if valid_interval(args.minimum_interval) < 0.1:
            raise ValueError("Minimum must be at least 0.1 seconds")
    except ValueError:
        parser.error("--interval must be between 0 and 300 seconds")
    if args.interval_metric and not args.usb_device:
        parser.error("--interval-metric requires --usb-device")
    server = args.server.rstrip("/")
    if urlparse(server).scheme not in ("http", "https"):
        parser.error("Invalid server URL")
    if args.usb_device:
        if not str(args.usb_device).startswith("/dev/"):
            parser.error("--usb-device must be a path under /dev/")
        input_args = ["-f", "v4l2", "-input_format", "mjpeg",
                      "-video_size", "640x480", "-framerate", "10", "-i", str(args.usb_device)]
    else:
        source_file = args.rtsp_file or HELPERS["ROOT"] / ".local/camera-smoke/source"
        source = os.getenv("HERAMIND_RTSP_URL") or (source_file.read_text().strip() if source_file.exists() else getpass.getpass("URL RTSP: "))
        parsed = urlparse(source)
        if parsed.scheme not in ("rtsp", "rtsps") or not parsed.hostname or any(ord(c) < 32 for c in source):
            parser.error("Invalid RTSP URL")
        input_args = ["-rtsp_transport", "tcp", "-timeout", "10000000", "-i", source]
    token = HELPERS["authenticate"](server, args.token_file)
    if not args.skip_face_binding:
        HELPERS["setup_face_camera"](server, token, args.device_id)
    capture_interval = 0.1 if args.usb_device else max(args.interval, 0.1)
    command = ["ffmpeg", "-hide_banner", "-loglevel", "error", "-nostdin",
               *input_args,
               "-an", "-vf", f"fps=1/{capture_interval},scale=640:-2", "-threads", "2",
               "-c:v", "mjpeg", "-q:v", "4", "-f", "image2pipe", "pipe:1"]
    sent = 0
    interval = args.interval
    last_sent = float("-inf")
    last_control_read = float("-inf")
    print(f"Đang cấp ảnh cho {args.device_id}, mỗi {args.interval:g} giây. Ctrl+C để dừng.", flush=True)
    while True:
        with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                              stdin=subprocess.DEVNULL) as process:
            try:
                for frame in latest_frames(process.stdout):
                    now = time.monotonic()
                    if args.interval_metric and now - last_control_read >= 1:
                        interval = read_interval(server, token, args.device_id, args.interval_metric, interval)
                        last_control_read = now
                    if now - last_sent < max(interval, args.minimum_interval):
                        continue
                    last_sent = now
                    value = "data:image/jpeg;base64," + base64.b64encode(frame).decode()
                    HELPERS["request"](server, f"/devices/{args.device_id}/metrics", token,
                                       {"metric": "image", "value": value})
                    sent += 1
                    if sent == 1 or sent % 30 == 0:
                        print(f"Đã gửi ảnh #{sent}", flush=True)
                    if args.once:
                        return
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        if args.once:
            raise ValueError("Camera disconnected before producing a frame")
        print("Camera ngắt kết nối; thử lại sau 3 giây.", flush=True)
        time.sleep(3)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nĐã dừng cấp ảnh.")
    except HTTPError as error:
        print(f"Lỗi API HTTP {error.code}. Kiểm tra đăng nhập và trạng thái extension.", file=sys.stderr)
        sys.exit(1)
    except (URLError, OSError, ValueError, KeyError) as error:
        print(f"Không cấp được ảnh: {type(error).__name__}. Kiểm tra FFmpeg, camera và cấu hình.", file=sys.stderr)
        sys.exit(1)
