# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""CORS behaviour of the pinned server, as pinned and with its contradiction resolved (Q8).

As pinned, the server allows every origin together with credentials. The CORS
protocol forbids a credentialed response to say "*", so Starlette answers
preflights by echoing the caller's origin. Its authorisation middleware also
runs before CORS, so with a key set every preflight is refused with a 403.

"Resolved" is the same application with the same Starlette CORSMiddleware,
configured with allow_credentials=False and placed in front of authorisation.
opengrowth must match it exactly; the test checks that the pinned server
differs from it only in the ways spec/queries.md Q8 records.

Each configuration runs in its own process, because the server reads
AUTHORIZATION_KEY when it is imported.
"""

import contextlib
import hashlib
import io
import json
import os
import subprocess
import sys
from pathlib import Path

ORIGIN = "http://localhost:3000"
KEY = "test-key"
VALID = json.dumps({"birth_date": "2020-04-12", "observation_date": "2028-06-12", "observation_value": 115, "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height"})
RECORDED = ("access-control-allow-origin", "access-control-allow-credentials", "access-control-allow-methods", "access-control-allow-headers", "access-control-max-age", "access-control-expose-headers", "vary", "content-type", "allow")


def cases(keyed):
    auth = {"authorization": f"Bearer {KEY}"} if keyed else {}
    preflight = {"origin": ORIGIN, "access-control-request-method": "POST"}
    browser = {"origin": ORIGIN, **auth}
    out = [
        ("preflight, demo client", "OPTIONS", "/uk-who/calculation", {**preflight, "access-control-request-headers": "content-type"}, ""),
        ("preflight, demo client with a subscription key", "OPTIONS", "/uk-who/calculation", {**preflight, "access-control-request-headers": "content-type,subscription-key"}, ""),
        ("preflight, bearer token", "OPTIONS", "/uk-who/calculation", {**preflight, "access-control-request-headers": "authorization,content-type"}, ""),
        ("preflight, no request headers", "OPTIONS", "/uk-who/calculation", preflight, ""),
        ("preflight, unsupported method", "OPTIONS", "/uk-who/calculation", {"origin": ORIGIN, "access-control-request-method": "PROPFIND"}, ""),
        ("preflight, unknown path", "OPTIONS", "/nowhere", preflight, ""),
        ("preflight, root", "OPTIONS", "/", {"origin": ORIGIN, "access-control-request-method": "GET"}, ""),
        ("browser POST", "POST", "/uk-who/calculation", browser, VALID),
        ("browser POST with a cookie", "POST", "/uk-who/calculation", {**browser, "cookie": "a=b"}, VALID),
        ("browser POST, validation error", "POST", "/uk-who/calculation", browser, "{}"),
        ("browser GET /", "GET", "/", {"origin": ORIGIN}, ""),
        ("browser OPTIONS, not a preflight", "OPTIONS", "/uk-who/calculation", {"origin": ORIGIN}, ""),
        ("POST without Origin", "POST", "/uk-who/calculation", auth, VALID),
        ("OPTIONS without Origin", "OPTIONS", "/uk-who/calculation", {"access-control-request-method": "POST"}, ""),
    ]
    if keyed:
        out.append(("browser POST without the key", "POST", "/uk-who/calculation", {"origin": ORIGIN}, VALID))
    return out


def capture(resolved, keyed):
    """Run every case against one configuration; print the observations as JSON."""
    server = Path(os.environ["OPENGROWTH_REFERENCE_SERVER_SRC"])
    sys.path.insert(0, str(server))
    os.chdir(server)
    with contextlib.redirect_stdout(io.StringIO()):
        import main
        from fastapi.middleware.cors import CORSMiddleware
        from fastapi.testclient import TestClient

        if resolved:
            main.app.user_middleware = [m for m in main.app.user_middleware if m.cls is not CORSMiddleware]
            main.app.add_middleware(CORSMiddleware, allow_origins=["*"], allow_credentials=False, allow_methods=["*"], allow_headers=["*"])
        client = TestClient(main.app, raise_server_exceptions=False)
        observed = []
        for name, method, path, headers, body in cases(keyed):
            request_headers = {**headers, "content-type": "application/json"} if body else headers
            r = client.request(method, path, headers=request_headers, content=body.encode())
            entry = {"status": r.status_code, "headers": {h: r.headers[h] for h in RECORDED if h in r.headers}}
            if len(r.content) < 400:
                entry["body"] = r.content.decode()
            else:
                entry["body_sha256"] = hashlib.sha256(r.content).hexdigest()
            observed.append(entry)
    print(json.dumps(observed))


def run(resolved, keyed):
    env = {k: v for k, v in os.environ.items() if k != "AUTHORIZATION_KEY"}
    if keyed:
        env["AUTHORIZATION_KEY"] = KEY
    out = subprocess.run([sys.executable, __file__, "--capture", str(int(resolved)), str(int(keyed))], env=env, capture_output=True, text=True, check=True)
    return json.loads(out.stdout.strip().splitlines()[-1])


if __name__ == "__main__" and sys.argv[1:2] == ["--capture"]:
    capture(resolved=sys.argv[2] == "1", keyed=sys.argv[3] == "1")
    sys.exit(0)

from _common import write  # noqa: E402

golden = []
for keyed in (False, True):
    pinned, resolved = run(False, keyed), run(True, keyed)
    for (name, method, path, headers, body), p, r in zip(cases(keyed), pinned, resolved):
        golden.append({"name": name, "key": KEY if keyed else None, "method": method, "path": path, "request_headers": headers, "request_body": body, "pinned": p, "resolved": r})
write("http-cors.json", "http_cors.py", "CORS responses from the pinned server, as pinned and with allow_credentials=False in front of authorisation (Q8).", golden)
