# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Shared generator for a reference's chart-coordinates goldens (R13, R15).

Used by http_uk_who_charts.py and http_who_charts.py.

The pinned server does not calculate named-format charts: it serves JSON files
committed to its repository (chart-data/), last regenerated in January 2025,
which differ from what rcpchgrowth 4.6.5 calculates. opengrowth calculates
its curves with the same engine as its measurements, so the target is the
server's response shape and serialisation with curves from 4.6.5.

To capture that, this script copies the pinned server to a temporary
directory, regenerates its UK-WHO chart files with 4.6.5 exactly as the
server's own generate_and_store_chart_data does, and calls the endpoint with
TestClient. For each named-format case it also records the digest the
unmodified server returns, so the stale-file difference stays measured.
"""

import contextlib
import hashlib
import importlib
import io
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path

from rcpchgrowth import chart_functions

from _common import RECORDED_HEADERS, mask_provenance, write

SERVER = Path(os.environ["OPENGROWTH_REFERENCE_SERVER_SRC"])
FORMATS = ["cole-nine-centiles", "three-percent-centiles", "five-percent-centiles", "eighty-five-percent-centiles"]
SEXES = ["male", "female"]
METHODS = ["height", "weight", "ofc", "bmi"]


def client_for(src: Path):
    from fastapi.testclient import TestClient

    sys.path.insert(0, str(src))
    os.chdir(src)
    for name in [m for m in sys.modules if m == "main" or m.startswith(("routers", "schemas", "server_metadata"))]:
        del sys.modules[name]
    with contextlib.redirect_stdout(io.StringIO()):
        main = importlib.import_module("main")
    sys.path.remove(str(src))
    return TestClient(main.app, raise_server_exceptions=False)


def call(client, name, body=None, raw=None, full=True, path="/uk-who/chart-coordinates"):
    if raw is None:
        raw = json.dumps(body, ensure_ascii=False)
    with contextlib.redirect_stdout(io.StringIO()):
        response = client.post(path, content=raw.encode(), headers={"content-type": "application/json"})
    text = response.content.decode()
    case = {
        "name": name,
        "method": "POST",
        "path": path,
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


def generate(reference, label, output, generator):
    """Write the chart golden for `reference` (also its route prefix), called `label`."""
    path = f"/{reference}/chart-coordinates"

    def route(client, name, body=None, raw=None, full=True):
        return call(client, name, body, raw, full, path)

    # The server exactly as pinned, serving its committed files.
    stale = client_for(SERVER)
    stale_digests = {}
    for fmt in FORMATS:
        for sex in SEXES:
            for method in METHODS:
                case = route(stale, "stale", {"sex": sex, "measurement_method": method, "centile_format": fmt}, full=False)
                stale_digests[(fmt, sex, method)] = case["masked_sha256"]

    # A copy with the UK-WHO chart files regenerated from rcpchgrowth 4.6.5.
    work = Path(tempfile.mkdtemp(prefix="opengrowth-server-"))
    copy = work / "src"
    shutil.copytree(SERVER, copy, ignore=shutil.ignore_patterns(".git", "__pycache__"))
    for fmt in FORMATS:
        for sex in SEXES:
            for method in METHODS:
                with contextlib.redirect_stdout(io.StringIO()):
                    data = chart_functions.create_chart(reference, centile_format=fmt, measurement_method=method, sex=sex)
                (copy / "chart-data" / f"{fmt}-{reference}-{sex}-{method}.json").write_text(json.dumps(data, indent=4))
    fresh = client_for(copy)

    cases = []
    for fmt in FORMATS:
        for sex in SEXES:
            for method in METHODS:
                body = {"sex": sex, "measurement_method": method, "centile_format": fmt}
                full = fmt == "cole-nine-centiles" and sex == "female" and method == "height"
                case = route(fresh, f"named {fmt} {sex} {method}", body, full=full)
                case["stale_masked_sha256"] = stale_digests[(fmt, sex, method)]
                cases.append(case)
    cases.append(route(fresh, "named default format", {"sex": "male", "measurement_method": "weight"}, full=False))

    for name, body, full in [
        ("custom one centile", {"sex": "male", "measurement_method": "bmi", "centile_format": [50]}, True),
        ("custom cole values, exact sds", {"sex": "female", "measurement_method": "height", "centile_format": [0.4, 2, 9, 25, 50, 75, 91, 98, 99.6]}, False),
        ("custom two centiles", {"sex": "male", "measurement_method": "weight", "centile_format": [3, 97]}, False),
        ("custom extreme centiles", {"sex": "female", "measurement_method": "ofc", "centile_format": [0.01, 99.99]}, False),
        ("custom string numbers", {"sex": "female", "measurement_method": "ofc", "centile_format": ["50", 25.5]}, False),
        ("custom just below median", {"sex": "female", "measurement_method": "weight", "centile_format": [49.9]}, False),
        ("sds one", {"sex": "male", "measurement_method": "bmi", "centile_format": [1.5], "is_sds": True}, True),
        ("sds three", {"sex": "female", "measurement_method": "ofc", "centile_format": [-2, 0, 2.5], "is_sds": True}, False),
        ("sds extreme", {"sex": "male", "measurement_method": "height", "centile_format": [-8, 8], "is_sds": True}, False),
        ("sds is_sds string", {"sex": "male", "measurement_method": "height", "centile_format": [1], "is_sds": "true"}, False),
        ("sds negative allowed", {"sex": "male", "measurement_method": "weight", "centile_format": [-1], "is_sds": True}, False),
        ("error centile 0", {"sex": "male", "measurement_method": "bmi", "centile_format": [0]}, True),
        ("error centile 100", {"sex": "male", "measurement_method": "bmi", "centile_format": [100]}, True),
        ("error centile 150", {"sex": "male", "measurement_method": "bmi", "centile_format": [50, 150]}, True),
        ("error sds overflow", {"sex": "male", "measurement_method": "bmi", "centile_format": [1e307], "is_sds": True}, True),
        ("error extended who", {"sex": "male", "measurement_method": "bmi", "centile_format": "extended-who-centiles"}, True),
        ("error null format", {"sex": "male", "measurement_method": "bmi", "centile_format": None}, True),
        ("validation empty", {"sex": "male", "measurement_method": "bmi", "centile_format": []}, True),
        ("validation negative", {"sex": "male", "measurement_method": "bmi", "centile_format": [-1]}, True),
        ("validation sixteen", {"sex": "male", "measurement_method": "bmi", "centile_format": list(range(1, 17))}, True),
        ("validation unknown name", {"sex": "male", "measurement_method": "bmi", "centile_format": "nope"}, True),
        ("validation bad item", {"sex": "male", "measurement_method": "bmi", "centile_format": [50, "x"]}, True),
        ("validation number", {"sex": "male", "measurement_method": "bmi", "centile_format": 5}, True),
        ("validation is_sds", {"sex": "male", "measurement_method": "bmi", "is_sds": "maybe"}, True),
        ("validation missing sex", {"measurement_method": "bmi"}, True),
        ("validation method", {"sex": "male", "measurement_method": "length"}, True),
        ("validation sex and format", {"sex": "Male", "measurement_method": "bmi", "centile_format": "x"}, True),
    ]:
        cases.append(route(fresh, name, body, full=full))

    shutil.rmtree(work)
    write(output, generator, f"FastAPI chart-coordinates responses for {label}, with named-format curves regenerated from rcpchgrowth 4.6.5.", cases)
