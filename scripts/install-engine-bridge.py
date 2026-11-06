#!/usr/bin/env python3
"""Install the engine-bridge package through HeraMind's authenticated API."""
import argparse
import base64
import json
from pathlib import Path
import time
import urllib.error
import urllib.request
import zipfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--token-file", type=Path, required=True)
    parser.add_argument("--server", default="http://127.0.0.1:9375")
    parser.add_argument("--device", help="Bind this device's image metric to SeetaFace after installation")
    parser.add_argument("--metric", default="image")
    parser.add_argument("--min-interval-ms", type=int, default=1500, help="Minimum delay between bound camera frames")
    parser.add_argument("--fixture", type=Path, help="Verify real face detection through the installed extension")
    args = parser.parse_args()
    if not 0 <= args.min_interval_ms <= 60_000:
        parser.error("--min-interval-ms must be 0–60000")
    with zipfile.ZipFile(args.package) as package:
        manifest = json.loads(package.read("manifest.json"))
        if manifest["id"] != "engine-bridge":
            raise SystemExit("Refusing to install a package for a different extension")
    token = args.token_file.read_text().strip()

    def api(method, endpoint, payload=None):
        request = urllib.request.Request(args.server.rstrip("/") + endpoint,
            data=json.dumps(payload).encode() if payload is not None else None, method=method,
            headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(request, timeout=180) as response:
                value = json.load(response)
        except urllib.error.HTTPError as error:
            # A response may contain echoed input. Report status without leaking images or credentials.
            raise RuntimeError(f"HeraMind {method} {endpoint} returned HTTP {error.code}") from None
        if value.get("success") is False:
            raise RuntimeError(f"HeraMind rejected {endpoint}")
        return value.get("data", value)

    entries = api("GET", "/api/extensions")
    entries = entries.get("extensions", []) if isinstance(entries, dict) else entries
    previous_config = None
    if any(entry.get("id") == "engine-bridge" for entry in entries):
        previous_config = api("GET", "/api/extensions/engine-bridge/config")["current_config"]
    print("Uploading engine-bridge package through the HeraMind API...", flush=True)
    api("POST", "/api/extensions/upload/file", {
        "filename": args.package.name,
        "data": base64.b64encode(args.package.read_bytes()).decode(),
    })
    if previous_config is not None:
        restored = api("PUT", "/api/extensions/engine-bridge/config", previous_config)
        if not restored.get("hot_reloaded"):
            api("POST", "/api/extensions/engine-bridge/reload", {})
    health = api("POST", "/api/extensions/engine-bridge/command", {"command": "health", "args": {}})["data"]
    print(json.dumps({"installed": "engine-bridge", "backend": health["backend"],
        "seetaface_compiled": health["seetaface_compiled"], "qnn_available": health["qnn_available"],
        "age_estimation": health.get("age_estimation")}), flush=True)
    if args.fixture:
        result = api("POST", "/api/extensions/engine-bridge/command", {"command": "process", "args": {
            "device_id": "engine-bridge-validation", "task": "face.recognize",
            "options": {"estimate_age": health.get("age_estimation", {}).get("model_available", False)},
            "image_base64": base64.b64encode(args.fixture.read_bytes()).decode()}})
        count = len(result["result"]["faces"])
        if count != 1:
            raise RuntimeError(f"Expected one face in the fixture, got {count}")
        print(json.dumps({"installed_face_test": "passed", "faces": count, "latency_ms": result["latency_ms"]}), flush=True)
    if args.device:
        settings = api("GET", "/api/extensions/engine-bridge/config")["current_config"]
        settings["timeout_ms"] = 60000
        bindings = settings.setdefault("bindings", [])
        bindings[:] = [binding for binding in bindings if not
            (binding["device_id"] == args.device and binding.get("metric", "image") == args.metric)]
        bindings.append({"device_id": args.device, "metric": args.metric,
                "task": "face.recognize", "options": {"backend": "cpu"}, "enabled": True,
                "min_interval_ms": args.min_interval_ms})
        config = api("PUT", "/api/extensions/engine-bridge/config", settings)
        if not config.get("hot_reloaded"):
            api("POST", "/api/extensions/engine-bridge/reload", {})
        for _ in range(30):
            latest = api("POST", "/api/extensions/engine-bridge/command", {"command": "get_latest", "args": {"device_id": args.device}})["latest"]
            if latest:
                print(json.dumps({"bound_device": args.device, "task": latest["task"],
                    "faces": len(latest["result"]["faces"]), "latency_ms": latest["latency_ms"]}), flush=True)
                break
            time.sleep(1)
        else:
            print(json.dumps({"bound_device": args.device, "waiting_for_camera": True,
                "message": "Installed and configured; no new camera image arrived within 30 seconds"}), flush=True)
        status = api("POST", "/api/extensions/engine-bridge/command", {"command": "get_status", "args": {}})
        print(json.dumps({key: status[key] for key in ("processed", "failed", "dropped", "publish_failed", "last_error")}), flush=True)


if __name__ == "__main__":
    main()
