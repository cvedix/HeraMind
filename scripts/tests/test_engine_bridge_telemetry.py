#!/usr/bin/env python3
"""Verify REST telemetry → DeviceMetric → installed SeetaFace → engine.result."""
import argparse
import base64
import copy
import json
from pathlib import Path
import time
import urllib.request
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", default="http://127.0.0.1:9375")
    parser.add_argument("--token-file", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    token = args.token_file.read_text().strip()

    def api(method, path, body=None):
        request = urllib.request.Request(args.server.rstrip("/") + path, method=method,
            headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"},
            data=json.dumps(body).encode() if body is not None else None)
        with urllib.request.urlopen(request, timeout=90) as response:
            value = json.load(response)
        assert value.get("success", True), "HeraMind rejected request"
        return value.get("data", value)

    def configure(config):
        result = api("PUT", "/api/extensions/engine-bridge/config", config)
        if not result.get("hot_reloaded"):
            api("POST", "/api/extensions/engine-bridge/reload", {})

    original = api("GET", "/api/extensions/engine-bridge/config")["current_config"]
    device = "engine-bridge-test-" + uuid.uuid4().hex[:12]
    created = False
    try:
        result = api("POST", "/api/devices", {"device_id": device, "name": "Engine Bridge test fixture",
            "device_type": "rtsp-camera", "adapter_type": "http", "connection_config": {}})
        assert not result.get("updated_existing"), "Unexpected test device collision"
        created = True
        config = copy.deepcopy(original)
        config["bindings"].append({"device_id": device, "metric": "image", "task": "face.recognize"})
        configure(config)
        image = "data:image/png;base64," + base64.b64encode(args.fixture.read_bytes()).decode()
        api("POST", f"/api/devices/{device}/metrics", {"metric": "image", "value": image})
        for _ in range(60):
            latest = api("POST", "/api/extensions/engine-bridge/command", {
                "command": "get_latest", "args": {"device_id": device}})["latest"]
            if latest:
                break
            time.sleep(1)
        else:
            raise RuntimeError("No SeetaFace result from the telemetry fixture")
        assert latest["task"] == "face.recognize", latest["task"]
        assert len(latest["result"]["faces"]) == 1, "Expected exactly one face"
        status = api("POST", "/api/extensions/engine-bridge/command", {"command": "get_status", "args": {}})
        assert status["publish_failed"] == 0, "engine.result publication failed"
        report = {"telemetry_to_seetaface": "passed", "faces": 1, "backend": latest["result"]["backend"],
            "latency_ms": latest["latency_ms"], "publish_failed": status["publish_failed"]}
    finally:
        configure(original)
        if created:
            api("DELETE", f"/api/devices/{device}")
    report["test_device_removed"] = True
    report["original_bindings_restored"] = True
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
