# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Shared helpers for golden generators. Run only through s/goldens."""

import importlib.metadata
import json
import os
import platform
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXPECTED = ROOT / "conformance" / "expected"
INDEPENDENT = ROOT / "conformance" / "independent"


def environment() -> dict:
    """Versions that determine the golden values, recorded with every golden."""
    versions = {}
    for package in ("rcpchgrowth", "scipy", "numpy", "fastapi", "pydantic", "starlette"):
        try:
            versions[package] = importlib.metadata.version(package)
        except importlib.metadata.PackageNotFoundError:
            versions[package] = None
    versions["python"] = platform.python_version()
    server = os.environ.get("OPENGROWTH_REFERENCE_SERVER_SRC")
    if server:
        versions["digital-growth-charts-server"] = subprocess.run(
            ["git", "-C", server, "rev-parse", "HEAD"], capture_output=True, text=True, check=True
        ).stdout.strip()
    versions["platform"] = f"{platform.system()} {platform.machine()}"
    return versions


def write(name: str, generator: str, description: str, cases: list) -> None:
    """Write a golden file. Floats are written with repr precision, so they round-trip exactly."""
    document = {
        "generator": f"conformance/generators/{generator}",
        "command": "s/goldens",
        "description": description,
        "environment": environment(),
        "cases": cases,
    }
    path = EXPECTED / name
    path.write_text(json.dumps(document, indent=1, allow_nan=False) + "\n")
    count = sum(len(v) for v in cases.values()) if isinstance(cases, dict) else len(cases)
    print(f"wrote {path.relative_to(ROOT)} ({count} cases)", file=sys.stderr)


RECORDED_HEADERS = ("content-type", "allow", "location", "x-git-revision")


def mask_provenance(body: str) -> str:
    """Remove every `"provenance": <value>` member, and its separating comma."""
    decoder = json.JSONDecoder()
    key = '"provenance":'
    out = body
    start = 0
    while (i := out.find(key, start)) != -1:
        if i == 0 or out[i - 1] not in "{,":
            start = i + len(key)
            continue
        _, end = decoder.raw_decode(out, i + len(key))
        if end < len(out) and out[end] == ",":
            out = out[:i] + out[end + 1 :]
        elif out[i - 1] == ",":
            out = out[: i - 1] + out[end:]
        else:
            out = out[:i] + out[end:]
        start = max(i - 1, 0)
    return out
