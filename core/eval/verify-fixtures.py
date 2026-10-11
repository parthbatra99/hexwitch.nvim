#!/usr/bin/env python3
"""Offline metadata/hash gate; does not judge aesthetic or editor fidelity."""
import hashlib
import json
import re
from pathlib import Path

REQUIRED = {"file", "origin", "license", "revision", "sha256", "purpose", "expected", "limitations"}
OPTIONAL = {"license_file"}


def verify(manifest_path: Path) -> int:
    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    if set(data) != {"version", "suite", "references"} or data["version"] != 1 or data["suite"] not in {"import_fidelity", "interpretation_quality"}:
        raise ValueError("invalid fixture manifest contract")
    if not isinstance(data["references"], list) or not data["references"]:
        raise ValueError("fixture references must be a nonempty list")
    root = manifest_path.parent.resolve()
    seen = set()
    for fixture in data["references"]:
        if not isinstance(fixture, dict) or not REQUIRED <= fixture.keys() or fixture.keys() - REQUIRED - OPTIONAL:
            raise ValueError("missing or unknown fixture metadata fields")
        for key in REQUIRED - {"expected", "limitations"}:
            if not isinstance(fixture[key], str) or not fixture[key].strip():
                raise ValueError(f"invalid {key}")
        for key in ("expected", "limitations"):
            if not isinstance(fixture[key], list) or not fixture[key] or any(not isinstance(v, str) or not v.strip() for v in fixture[key]):
                raise ValueError(f"{key} needs explicit nonempty evidence/limitations")
        if not re.fullmatch(r"[0-9a-f]{64}", fixture["sha256"]):
            raise ValueError("invalid SHA-256 metadata")
        path = (root / fixture["file"]).resolve()
        if not path.is_relative_to(root) or fixture["file"] in seen or not path.is_file():
            raise ValueError("invalid, duplicate, or escaping fixture path")
        seen.add(fixture["file"])
        if hashlib.sha256(path.read_bytes()).hexdigest() != fixture["sha256"]:
            raise ValueError(f"fixture bytes changed: {fixture['file']}")
        if "license_file" in fixture:
            if not isinstance(fixture["license_file"], str):
                raise ValueError("invalid license_file")
            license_path = (root / fixture["license_file"]).resolve()
            if not license_path.is_relative_to(root) or not license_path.is_file():
                raise ValueError("missing or escaping license notice")
    return len(seen)


if __name__ == "__main__":
    root = Path(__file__).parent
    counts = {name: verify(root / name / "manifest.json") for name in ("import", "states")}
    print(f"Verified metadata and exact hashes: {counts['import']} import fixtures, {counts['states']} evaluation states")
