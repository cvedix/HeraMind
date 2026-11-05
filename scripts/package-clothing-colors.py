#!/usr/bin/env python3
"""Build the Linux ARM64 / Python 3.12 clothing-colors extension package."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parent.parent
MODEL_REVISION = "61f4ff3c508b899904e495cefbbaff9cacadeb45"
MODEL_NAME = "schp-atr-18-int8-static.onnx"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-deps", action="store_true", help="Reuse the prepared dependency directory")
    args = parser.parse_args()
    if platform.machine() != "aarch64" or sys.version_info[:2] != (3, 12):
        parser.error("This package targets the current Linux ARM64 / Python 3.12 host")
    extension = ROOT / "extensions/clothing-colors"
    subprocess.run(["cargo", "build", "--manifest-path", str(extension / "Cargo.toml"), "--release"], check=True)
    stage = ROOT / ".local/clothing-stage"
    for directory in ["binaries/linux_arm64", "frontend", "assets/worker", "models", "assets/python/site-packages"]:
        (stage / directory).mkdir(parents=True, exist_ok=True)
    shutil.copyfile(extension / "target/release/libheramind_clothing_colors.so", stage / "binaries/linux_arm64/extension.so")
    for relative in ["manifest.json", "frontend/clothing-colors.umd.cjs"]:
        shutil.copyfile(extension / relative, stage / relative)
    shutil.copyfile(extension / "worker/analyze.py", stage / "assets/worker/analyze.py")
    # Migrate a locally prepared dependency directory from early development.
    if (stage / "runtime/site-packages/onnxruntime").exists():
        shutil.rmtree(stage / "assets/python/site-packages")
        shutil.move(str(stage / "runtime/site-packages"), str(stage / "assets/python/site-packages"))
        shutil.rmtree(stage / "runtime")
    if (stage / "worker").exists():
        shutil.rmtree(stage / "worker")
    model = ROOT / ".local/clothing-research" / MODEL_NAME
    if not model.exists():
        model.parent.mkdir(parents=True, exist_ok=True)
        url = f"https://huggingface.co/pirocheto/schp-atr-18/resolve/{MODEL_REVISION}/onnx/{MODEL_NAME}"
        temporary = model.with_suffix(".download")
        with urllib.request.urlopen(url, timeout=60) as source, temporary.open("wb") as output:
            shutil.copyfileobj(source, output)
        temporary.replace(model)
    shutil.copyfile(model, stage / "models" / MODEL_NAME)
    person_model = Path("/var/lib/heramind/extensions/yolo-video/models/yolo11n.onnx")
    if not person_model.is_file():
        parser.error("USB analysis requires the installed YOLO11n person model")
    shutil.copyfile(person_model, stage / "models/yolo11n.onnx")
    with model.open("rb") as model_file:
        checksum = hashlib.file_digest(model_file, "sha256").hexdigest()
    provenance = {"model": "pirocheto/schp-atr-18", "revision": MODEL_REVISION,
                  "file": MODEL_NAME, "sha256": checksum,
                  "target": "Linux aarch64 / CPython 3.12", "python_dependencies": (extension / "requirements.txt").read_text()}
    (stage / "assets/model-provenance.json").write_text(json.dumps(provenance, indent=2))
    if not args.skip_deps:
        subprocess.run([sys.executable, "-m", "pip", "install", "--only-binary=:all:",
                        "--target", str(stage / "assets/python/site-packages"),
                        "-r", str(extension / "requirements.txt")], check=True)
    if not (stage / "assets/python/site-packages/onnxruntime").is_dir():
        parser.error("Prepare dependencies before using --skip-deps")
    package = ROOT / ".local/clothing-colors-0.1.0-linux_arm64.nep"
    with zipfile.ZipFile(package, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=3) as archive:
        for path in sorted(stage.rglob("*")):
            if path.is_file() and "__pycache__" not in path.parts and path.suffix != ".pyc":
                archive.write(path, path.relative_to(stage).as_posix())
    print(f"Package: {package}\nSize: {package.stat().st_size / (1024 * 1024):.1f} MiB")


if __name__ == "__main__":
    main()
