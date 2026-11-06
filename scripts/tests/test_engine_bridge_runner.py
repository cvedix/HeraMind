#!/usr/bin/env python3
"""Exercise the installed runner ABI and native event capability bridge."""
import argparse
import base64
import json
import os
from pathlib import Path
import selectors
import struct
import subprocess
import time
import zlib


def image():
    def chunk(kind, data):
        return struct.pack("!I", len(data)) + kind + data + struct.pack("!I", zlib.crc32(kind + data) & 0xFFFFFFFF)
    png = (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack("!IIBBBBB", 2, 3, 8, 2, 0, 0, 0))
           + chunk(b"IDAT", zlib.compress((b"\x00" + b"\xff\x00\x00" * 2) * 3)) + chunk(b"IEND", b""))
    return "data:image/png;base64," + base64.b64encode(png).decode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runner", type=Path, required=True)
    parser.add_argument("--extension-root", type=Path, required=True)
    args = parser.parse_args()
    root = args.extension_root.resolve()
    extension = root / "binaries/linux_arm64/extension.so"
    proc = subprocess.Popen([str(args.runner.resolve()), "--extension-path", str(extension)],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        env={**os.environ, "HERAMIND_EXTENSION_DIR": str(root), "NEOMIND_EXTENSION_DIR": str(root)})

    def send(message):
        body = json.dumps(message).encode()
        proc.stdin.write(struct.pack("<I", len(body)) + body)
        proc.stdin.flush()

    def read_exact(length):
        result = b""
        with selectors.DefaultSelector() as selector:
            selector.register(proc.stdout, selectors.EVENT_READ)
            while len(result) < length:
                if not selector.select(15):
                    raise RuntimeError("Runner response timed out")
                part = os.read(proc.stdout.fileno(), length - len(result))
                if not part:
                    raise RuntimeError("Runner exited: " + proc.stderr.read().decode(errors="replace")[-2000:])
                result += part
        return result

    def receive():
        length, = struct.unpack("<I", read_exact(4))
        if not 0 < length <= 4 * 1024 * 1024:
            raise RuntimeError(f"Invalid runner frame size: {length}")
        return json.loads(read_exact(length))

    try:
        send({"Init": {"config": {"timeout_ms": 5000, "bindings": [
            {"device_id": "bridge-smoke-camera", "metric": "image", "task": "image.inspect"}]}}})
        ready = receive()
        assert "Ready" in ready, ready
        send({"ExecuteCommand": {"request_id": 1, "command": "health", "args": {}}})
        health = receive()
        assert health["Success"]["data"]["data"]["backend"] == "cpu", health
        send({"ExecuteCommand": {"request_id": 2, "command": "process", "args": {
            "device_id": "bridge-smoke-camera", "task": "image.inspect", "image_base64": image()}}})
        processed = receive()
        assert processed["Success"]["data"]["result"]["width"] == 2, processed
        # Exercise event delivery AND the reverse native capability IPC path.
        send({"EventPush": {"event_type": "DeviceMetric", "timestamp": int(time.time() * 1000),
            "payload": {"device_id": "bridge-smoke-camera", "metric": "image", "value": {"String": image()}}}})
        capability = receive()["CapabilityRequest"]
        assert capability["capability"] == "event_publish", capability
        assert capability["params"]["event_type"] == "engine.result", capability
        assert capability["params"]["payload"]["device_id"] == "bridge-smoke-camera", capability
        send({"CapabilityResult": {"request_id": capability["request_id"], "result": {"success": True}, "error": None}})
        send({"ExecuteCommand": {"request_id": 3, "command": "get_status", "args": {}}})
        status = receive()
        assert status["Success"]["data"]["publish_failed"] == 0, status
        send("Shutdown")
        assert receive() == "ShutdownAck"
        proc.wait(timeout=5)
        assert proc.returncode == 0, proc.returncode
        print("Runner ABI, engine processing, DeviceMetric and engine.result capability: OK")
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait(timeout=5)
        proc.stdin.close()
        proc.stdout.close()
        proc.stderr.close()


if __name__ == "__main__":
    main()
