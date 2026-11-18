# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /uk-who/fictional-child-data.

Captured from the pinned server with Starlette's TestClient. Upstream adds
noise with random.uniform, so this script fixes Python's random source at
FIXED_DRAW: every draw is that fraction of the way across its range, through
upstream's own random.uniform. The opengrowth test injects the same fraction
(AppState::with_fixed_noise), so noisy series compare byte for byte too.
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
FIXED_DRAW = 0.8125
PATH = "/uk-who/fictional-child-data"

random._inst.random = lambda: FIXED_DRAW


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
    if response.status_code == 200:
        case["count"] = len(json.loads(text))
    if full:
        case["body"] = text
    return case


H, W, B, O = "height", "weight", "bmi", "ofc"
CASES = [
    # name, body, full body?
    ("defaults", {"measurement_method": H, "sex": "male"}, False),
    ("small full", {"measurement_method": H, "sex": "male", "start_chronological_age": 4, "end_age": 4.5, "measurement_interval_number": 2}, True),
    ("weight infant monthly", {"measurement_method": W, "sex": "female", "end_age": 2, "measurement_interval_number": 1}, False),
    ("ofc days", {"measurement_method": O, "sex": "male", "end_age": 0.25, "measurement_interval_type": "d", "measurement_interval_number": 10}, False),
    ("bmi weeks", {"measurement_method": B, "sex": "female", "end_age": 1, "measurement_interval_type": "w", "measurement_interval_number": 4}, False),
    ("height preterm 28+3", {"measurement_method": H, "sex": "female", "gestation_weeks": 28, "gestation_days": 3, "end_age": 1, "measurement_interval_number": 1}, False),
    ("weight preterm 24+0 weekly", {"measurement_method": W, "sex": "male", "gestation_weeks": 24, "end_age": 0.6, "measurement_interval_type": "weeks", "measurement_interval_number": 1}, False),
    ("ofc post-term 42+5", {"measurement_method": O, "sex": "female", "gestation_weeks": 42, "gestation_days": 5, "end_age": 3, "measurement_interval_number": 3}, False),
    ("start before birth", {"measurement_method": W, "sex": "male", "start_chronological_age": -0.1, "end_age": 0.1, "measurement_interval_type": "w", "measurement_interval_number": 2}, False),
    ("drift negative", {"measurement_method": B, "sex": "female", "start_chronological_age": 2, "end_age": 18, "measurement_interval_type": "y", "measurement_interval_number": 1, "start_sds": 1, "drift": True, "drift_range": -1.5}, False),
    ("drift positive", {"measurement_method": H, "sex": "male", "drift": True, "drift_range": 0.7}, False),
    ("drift default range", {"measurement_method": W, "sex": "female", "drift": True}, False),
    ("noise", {"measurement_method": W, "sex": "male", "noise": True, "noise_range": 0.05}, False),
    ("noise default range preterm", {"measurement_method": W, "sex": "female", "gestation_weeks": 30, "end_age": 1, "measurement_interval_number": 1, "noise": True}, False),
    ("noise range zero", {"measurement_method": O, "sex": "male", "end_age": 2, "measurement_interval_number": 3, "noise": True, "noise_range": 0}, False),
    ("noise and drift", {"measurement_method": H, "sex": "female", "start_sds": -1, "drift": True, "drift_range": 2, "noise": True, "noise_range": 0.02}, False),
    ("sds 7.95", {"measurement_method": H, "sex": "male", "start_sds": 7.95}, False),
    ("sds 9", {"measurement_method": H, "sex": "male", "start_sds": 9}, False),
    ("sds -9 weight", {"measurement_method": W, "sex": "male", "start_sds": -9}, False),
    ("sds -9 weight preterm", {"measurement_method": W, "sex": "male", "gestation_weeks": 24, "start_sds": -9, "end_age": 0.5, "measurement_interval_type": "w", "measurement_interval_number": 2}, False),
    ("sds 16 bmi", {"measurement_method": B, "sex": "female", "start_chronological_age": 2, "start_sds": 16}, False),
    ("ages past 20", {"measurement_method": H, "sex": "male", "start_chronological_age": 18, "end_age": 25}, True),
    ("unit day", {"measurement_method": W, "sex": "male", "start_chronological_age": 1, "end_age": 1.2, "measurement_interval_type": "day", "measurement_interval_number": 30}, False),
    ("unit days", {"measurement_method": W, "sex": "male", "start_chronological_age": 1, "end_age": 1.1, "measurement_interval_type": "days", "measurement_interval_number": 7}, False),
    ("unit w", {"measurement_method": H, "sex": "female", "start_chronological_age": 0.5, "end_age": 0.9, "measurement_interval_type": "w", "measurement_interval_number": 5}, False),
    ("unit week", {"measurement_method": H, "sex": "female", "start_chronological_age": 0.5, "end_age": 0.9, "measurement_interval_type": "week", "measurement_interval_number": 3}, False),
    ("unit m", {"measurement_method": O, "sex": "male", "start_chronological_age": 1, "end_age": 2, "measurement_interval_type": "m", "measurement_interval_number": 4}, False),
    ("unit month", {"measurement_method": O, "sex": "male", "start_chronological_age": 1, "end_age": 2, "measurement_interval_type": "month", "measurement_interval_number": 5}, False),
    ("unit y", {"measurement_method": B, "sex": "male", "start_chronological_age": 4, "end_age": 10, "measurement_interval_type": "y", "measurement_interval_number": 2}, False),
    ("unit year", {"measurement_method": B, "sex": "male", "start_chronological_age": 4, "end_age": 7, "measurement_interval_type": "year", "measurement_interval_number": 1}, False),
    ("unit years", {"measurement_method": H, "sex": "male", "start_chronological_age": 0, "end_age": 20, "measurement_interval_type": "years", "measurement_interval_number": 4}, False),
    ("lax numbers", {"measurement_method": H, "sex": "male", "start_chronological_age": "1.5", "end_age": 3, "measurement_interval_number": "6", "gestation_weeks": "38", "drift": "true", "drift_range": "0.5"}, False),
    ("interval number float", {"measurement_method": H, "sex": "male", "end_age": 2, "measurement_interval_number": 6.0}, False),
    ("start_sds null", {"measurement_method": H, "sex": "male", "start_sds": None}, True),
    ("start_sds null, gestation_days null", {"measurement_method": H, "sex": "male", "start_sds": None, "gestation_days": None}, True),
    ("gestation_days null, ages past 20", {"measurement_method": H, "sex": "male", "start_chronological_age": 21, "end_age": 25, "gestation_days": None}, True),
    ("noise_range null, ages past 20", {"measurement_method": H, "sex": "male", "start_chronological_age": 21, "end_age": 25, "noise": True, "noise_range": None}, True),
    ("drift_range null without drift", {"measurement_method": H, "sex": "male", "start_chronological_age": 4, "end_age": 4.5, "measurement_interval_number": 2, "drift_range": None}, False),
    ("error end before start", {"measurement_method": H, "sex": "male", "start_chronological_age": 2, "end_age": 1}, True),
    ("error end equals start", {"measurement_method": H, "sex": "male", "start_chronological_age": 1, "end_age": 1}, True),
    ("error interval zero", {"measurement_method": H, "sex": "male", "measurement_interval_number": 0}, True),
    ("error interval negative", {"measurement_method": H, "sex": "male", "measurement_interval_number": -3}, True),
    ("error span shorter than interval", {"measurement_method": H, "sex": "male", "end_age": 1, "measurement_interval_type": "y", "measurement_interval_number": 2}, True),
    ("error span shorter after correction", {"measurement_method": H, "sex": "male", "gestation_weeks": 23, "end_age": 1.2, "measurement_interval_type": "y", "measurement_interval_number": 1}, True),
    ("error dates beyond 9999", {"measurement_method": H, "sex": "male", "start_chronological_age": 19, "end_age": 1e7, "measurement_interval_type": "y", "measurement_interval_number": 1000}, True),
    ("error dates before year 1", {"measurement_method": H, "sex": "male", "start_chronological_age": -2000, "end_age": 1}, True),
    ("error gestation_weeks null", {"measurement_method": H, "sex": "male", "gestation_weeks": None}, True),
    ("error gestation_days null", {"measurement_method": H, "sex": "male", "gestation_days": None}, True),
    ("error gestation_days null preterm", {"measurement_method": H, "sex": "male", "gestation_weeks": 30, "gestation_days": None}, True),
    ("error start null", {"measurement_method": H, "sex": "male", "start_chronological_age": None}, True),
    ("error end null", {"measurement_method": H, "sex": "male", "end_age": None}, True),
    ("error interval type null", {"measurement_method": H, "sex": "male", "measurement_interval_type": None}, True),
    ("error interval number null", {"measurement_method": H, "sex": "male", "measurement_interval_number": None}, True),
    ("error drift without range", {"measurement_method": H, "sex": "male", "drift": True, "drift_range": None}, True),
    ("error drift without sds", {"measurement_method": H, "sex": "male", "drift": True, "start_sds": None}, True),
    ("error noise without range", {"measurement_method": H, "sex": "male", "noise": True, "noise_range": None}, True),
    ("validation gestation 21", {"measurement_method": H, "sex": "male", "gestation_weeks": 21}, True),
    ("validation gestation days 7", {"measurement_method": H, "sex": "male", "gestation_days": 7}, True),
    ("validation interval type", {"measurement_method": H, "sex": "male", "measurement_interval_type": "fortnight"}, True),
    ("validation interval number 6.5", {"measurement_method": H, "sex": "male", "measurement_interval_number": 6.5}, True),
    ("validation drift null", {"measurement_method": H, "sex": "male", "drift": None}, True),
    ("validation noise maybe", {"measurement_method": H, "sex": "male", "noise": "maybe"}, True),
    ("validation start not a number", {"measurement_method": H, "sex": "male", "start_chronological_age": "abc"}, True),
    ("validation missing fields", {}, True),
    ("validation method and sex", {"measurement_method": "length", "sex": "Male", "end_age": "x"}, True),
    ("validation not an object", [], True),
]

c = client()
cases = [call(c, name, body, full) for name, body, full in CASES]
write(
    "http-uk-who-fictional-child.json",
    "http_uk_who_fictional_child.py",
    f"FastAPI fictional-child-data responses for UK-WHO, with every random draw fixed at {FIXED_DRAW}.",
    cases,
)
