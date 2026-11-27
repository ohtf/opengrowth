# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""How the pinned server reads a request body: which Content-Type values it
reads as JSON, and how it decodes the bytes (spec/queries.md Q16).

FastAPI reads a body as JSON only when Content-Type is absent or empty, or
names application/json or another application/*+json type as Python's
email.message parses it. It hands any other body to the request model
undecoded, and the model refuses it. This is FastAPI's fix for
CVE-2021-32677. A JSON body is decoded as json.loads decodes bytes: UTF-8,
UTF-16, or UTF-32, chosen by a byte-order mark or by where the zero bytes are.

Each case records exactly the headers sent, so a replay must send those and
no others. A body that is not UTF-8 is recorded as hex.
"""

import contextlib
import hashlib
import io
import json
import os
import sys

SERVER = os.environ["OPENGROWTH_REFERENCE_SERVER_SRC"]
sys.path.insert(0, SERVER)
os.chdir(SERVER)

from fastapi.testclient import TestClient  # noqa: E402

with contextlib.redirect_stdout(io.StringIO()):
    import main  # noqa: E402

from _common import RECORDED_HEADERS, mask_provenance, write  # noqa: E402

client = TestClient(main.app, raise_server_exceptions=False)

CALCULATION = {"birth_date": "2020-04-12", "observation_date": "2028-06-12", "observation_value": 115, "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height"}
BULK = {"birth_date": "2020-04-12", "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height", "observations": [{"observation_date": "2028-06-12", "observation_value": 115}]}
# Custom centiles, which upstream calculates; it serves named formats from stored files.
CHART = {"sex": "female", "measurement_method": "height", "centile_format": [2, 50, 98]}
FICTIONAL_CHILD = {"measurement_method": "height", "sex": "female", "start_chronological_age": 0, "end_age": 0.1, "gestation_weeks": 40, "gestation_days": 0, "measurement_interval_type": "days", "measurement_interval_number": 30, "start_sds": 0, "drift": False, "noise": False}
MID_PARENTAL_HEIGHT = {"height_paternal": 180, "height_maternal": 165, "sex": "male"}

# A calculation whose echoed events_text shows how the body was decoded.
ACCENTED = json.dumps({**CALCULATION, "events_text": ["café"]}, ensure_ascii=False)

# Content-Type values for a valid calculation: what reads as JSON, and what does not.
LABELS = [
    None,
    "",
    "application/json",
    "application/json; charset=utf-8",
    "application/json;charset=UTF-8",
    "Application/JSON",
    "application/json ; charset=utf-8",
    "application/json;",
    "application/vnd.api+json",
    "application/geo+json",
    "application/merge-patch+json",
    "application/+json",
    "application/jsonx",
    "application/json5",
    "application/*",
    "application / json",
    "application/json/extra",
    "json",
    "*/*",
    "text/json",
    "text/plain",
    "text/plain;charset=UTF-8",
    "application/x-www-form-urlencoded",
    "multipart/form-data; boundary=x",
    "application/xml",
]

# Routes, each with a body it accepts, and whether to keep the full response
# body (charts and fictional children are long; their digests are kept).
ROUTES = [
    ("/uk-who/calculation", CALCULATION, True),
    ("/who/calculation", CALCULATION, True),
    ("/cdc/calculation", CALCULATION, True),
    ("/turner/calculation", CALCULATION, True),
    ("/trisomy-21/calculation", CALCULATION, True),
    ("/trisomy-21-aap/calculation", CALCULATION, True),
    ("/uk-who/bulk-calculation", BULK, True),
    ("/uk-who/chart-coordinates", CHART, False),
    ("/uk-who/fictional-child-data", FICTIONAL_CHILD, False),
    ("/utilities/mid-parental-height", MID_PARENTAL_HEIGHT, True),
]


def label(content_type):
    return "no Content-Type" if content_type is None else f"Content-Type {content_type!r}"


def call(name, path, raw, content_type, full=True):
    headers = {} if content_type is None else {"content-type": content_type}
    with contextlib.redirect_stdout(io.StringIO()):
        response = client.request("POST", path, content=raw, headers=headers, follow_redirects=False)
    text = response.content.decode()
    case = {"name": name, "method": "POST", "path": path, "request_headers": headers}
    try:
        case["request_body"] = raw.decode("utf-8")
    except UnicodeDecodeError:
        case["request_body"] = ""
        case["request_body_hex"] = raw.hex()
    case.update({
        "status": response.status_code,
        "headers": {h: response.headers[h] for h in RECORDED_HEADERS if h in response.headers},
        "body_sha256": hashlib.sha256(response.content).hexdigest(),
        "masked_sha256": hashlib.sha256(mask_provenance(text).encode()).hexdigest(),
    })
    if full or response.status_code != 200:
        case["body"] = text
    return case


def cases():
    out = []
    valid = json.dumps(CALCULATION).encode()
    for content_type in LABELS:
        out.append(call(f"calculation, {label(content_type)}", "/uk-who/calculation", valid, content_type))

    for path, body, full in ROUTES:
        raw = json.dumps(body).encode()
        for content_type in (None, "application/json", "text/plain", "application/x-www-form-urlencoded"):
            out.append(call(f"{path}, {label(content_type)}", path, raw, content_type, full))

    bodies = [
        ("empty body", b""),
        ("an empty object", b"{}"),
        ("malformed JSON", b'{"birth_date": '),
        ("a form-encoded body", b"birth_date=2020-04-12&sex=female"),
        # A raw tab and newline inside a string, which JSON does not allow.
        ("non-ASCII and control characters", (ACCENTED[:-1] + ', "note": "tab\there\nline"}').encode()),
    ]
    for content_type in ("text/plain", "application/json"):
        for what, raw in bodies:
            out.append(call(f"calculation, {what}, {label(content_type)}", "/uk-who/calculation", raw, content_type))

    encodings = [
        ("UTF-8", ACCENTED.encode("utf-8")),
        ("UTF-8 with a byte-order mark", ACCENTED.encode("utf-8-sig")),
        ("UTF-16 with a byte-order mark", ACCENTED.encode("utf-16")),
        ("UTF-16LE without a byte-order mark", ACCENTED.encode("utf-16-le")),
        ("UTF-16BE without a byte-order mark", ACCENTED.encode("utf-16-be")),
        ("UTF-32 with a byte-order mark", ACCENTED.encode("utf-32")),
        ("UTF-32LE without a byte-order mark", ACCENTED.encode("utf-32-le")),
        ("Latin-1, which is not UTF-8", ACCENTED.encode("latin-1")),
    ]
    for content_type in (None, "application/json", "text/plain"):
        for what, raw in encodings:
            out.append(call(f"calculation, {what}, {label(content_type)}", "/uk-who/calculation", raw, content_type))
    return out


write(
    "http-request-bodies.json",
    "http_request_bodies.py",
    "How the pinned server reads a request body: the Content-Type values it reads as JSON, "
    "and how it decodes the bytes. Each case's request_headers are exactly the headers sent, "
    "and request_body_hex, where present, is the body.",
    cases(),
)
