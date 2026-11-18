# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /utilities/mid-parental-height.

Captured from the pinned server with Starlette's TestClient: parents across
the plausible range and at the +/-8 SD limits, every combination of limit
errors (their order is part of the response), a seeded sample of parent
pairs for rounding, and validation. The cdc and who references are captured
too; the opengrowth test lists them as pending until those references exist.
"""

import contextlib
import hashlib
import importlib
import io
import json
import os
import random
import sys
from pathlib import Path

from _common import RECORDED_HEADERS, mask_provenance, write

SERVER = Path(os.environ["OPENGROWTH_REFERENCE_SERVER_SRC"])
PATH = "/utilities/mid-parental-height"


def client():
    from fastapi.testclient import TestClient

    sys.path.insert(0, str(SERVER))
    os.chdir(SERVER)
    with contextlib.redirect_stdout(io.StringIO()):
        main = importlib.import_module("main")
    return TestClient(main.app, raise_server_exceptions=False)


def call(c, name, body, full):
    raw = json.dumps(body, ensure_ascii=False)
    with contextlib.redirect_stdout(io.StringIO()):
        response = c.post(PATH, content=raw.encode(), headers={"content-type": "application/json"})
    text = response.content.decode()
    case = {
        "name": name,
        "method": "POST",
        "path": PATH,
        "request_headers": {},
        "request_body": raw,
        "status": response.status_code,
        "headers": {h: response.headers[h] for h in RECORDED_HEADERS if h in response.headers},
        "body_sha256": hashlib.sha256(response.content).hexdigest(),
        "masked_sha256": hashlib.sha256(mask_provenance(text).encode()).hexdigest(),
    }
    if full:
        case["body"] = text
    return case


def parents(paternal, maternal, sex="male", **extra):
    return {"height_paternal": paternal, "height_maternal": maternal, "sex": sex, **extra}


CASES = [
    ("typical boy", parents(180, 165), True),
    ("typical girl", parents(180, 165, "female"), False),
    ("explicit uk-who", parents(175.5, 160.2, "female", reference="uk-who"), False),
    ("tall parents", parents(200, 185), False),
    ("very tall parents", parents(230, 210, "female"), False),
    ("short parents", parents(150, 140), False),
    ("parents just inside -8 SD", parents(122.3, 115.5, "female"), False),
    ("error paternal below -8", parents(120, 165), True),
    ("error maternal below -8", parents(180, 110), True),
    ("error paternal above 8", parents(235, 165), True),
    ("error maternal above 8", parents(180, 215), True),
    ("error paternal low, maternal high", parents(120, 215), True),
    ("error paternal high, maternal low", parents(235, 110), True),
    ("error both low", parents(100, 100), True),
    ("error both high", parents(300, 300), True),
    ("error zero height", parents(0, 165), True),
    ("error negative heights", parents(-10, -10), True),
    ("lax numbers", parents("180.5", "165.2", "female"), False),
    ("pending cdc", parents(180, 165, reference="cdc"), False),
    ("pending who", parents(180, 165, "female", reference="who"), False),
    ("validation missing fields", {}, True),
    ("validation missing maternal", {"height_paternal": 180, "sex": "male"}, True),
    ("validation sex", parents(180, 165, "Male"), True),
    ("validation reference", parents(180, 165, reference="uk90"), True),
    ("validation reference null", parents(180, 165, reference=None), True),
    ("validation height not a number", parents("abc", 165), True),
    ("validation height null", parents(None, 165), True),
    ("validation not an object", [], True),
]
rng = random.Random(20261118)
for i in range(40):
    paternal = round(rng.uniform(140, 215), rng.choice([0, 1, 2]))
    maternal = round(rng.uniform(130, 200), rng.choice([0, 1, 2]))
    CASES.append((f"sample {i}", parents(paternal, maternal, rng.choice(["male", "female"])), False))

c = client()
cases = [call(c, name, body, full) for name, body, full in CASES]
write("http-utilities-mid-parental-height.json", "http_utilities_mid_parental_height.py", "FastAPI mid-parental-height responses.", cases)
