#!/usr/bin/env python3
"""Download and verify recognition and age models used by Engine Bridge."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = "https://www.dropbox.com/s/julk1f16riu0dyp/sf6.0_models.zip?dl=1"
UPSTREAM = "https://github.com/SeetaFace6Open/index"
# Digest of the official archive retrieved on 2026-11-06. The upstream README
# does not publish a checksum; this pin detects changes on subsequent downloads.
SHA256 = "57d339895915b73f19b986f1def66eed8b51975e8b0e74eeb12c656277230800"
REQUIRED = (
    "face_detector.csta", "mask_detector.csta", "face_landmarker_pts5.csta",
    "face_recognizer.csta", "face_landmarker_mask_pts5.csta", "face_recognizer_mask.csta",
    "age_predictor.csta",
)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, default=ROOT / ".local/seetaface-downloads/sf6.0_models.zip")
    parser.add_argument("--output", type=Path, default=ROOT / ".local/seetaface-models")
    args = parser.parse_args()
    archive = args.archive.resolve()
    if not archive.exists():
        archive.parent.mkdir(parents=True, exist_ok=True)
        staging = archive.with_suffix(archive.suffix + ".part")
        with urllib.request.urlopen(SOURCE, timeout=60) as response, staging.open("wb") as output:
            shutil.copyfileobj(response, output, 1024 * 1024)
        if digest(staging) != SHA256:
            raise SystemExit("Official model archive checksum changed; inspect the upstream artifact before updating the pin")
        staging.replace(archive)
    if digest(archive) != SHA256:
        raise SystemExit("Model archive SHA256 mismatch")
    destination = args.output.resolve()
    destination.mkdir(parents=True, exist_ok=True)
    records = []
    with zipfile.ZipFile(archive) as package:
        for name in REQUIRED:
            entry = package.getinfo("sf3.0_models/" + name)
            if not 0 < entry.file_size < 256 * 1024 * 1024:
                raise SystemExit(f"Unexpected model size: {name}")
            target = destination / name
            staging = destination / (name + ".part")
            # Extract only allowlisted basenames; never unpack archive paths.
            with package.open(entry) as source, staging.open("wb") as output:
                shutil.copyfileobj(source, output, 1024 * 1024)
            if staging.stat().st_size != entry.file_size:
                raise SystemExit(f"Truncated model: {name}")
            staging.replace(target)
            records.append({"name": name, "bytes": target.stat().st_size, "sha256": digest(target)})
    provenance = {"upstream": UPSTREAM, "archive_url": SOURCE,
        "archive_sha256": SHA256, "checksum_origin": "locally pinned official download, 2026-11-06",
        "models": records}
    (destination / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    print(f"Verified {len(records)} SeetaFace6 models in {destination}")
    print(f"Total model bytes: {sum(record['bytes'] for record in records)}")


if __name__ == "__main__":
    main()
