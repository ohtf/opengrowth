# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""The pinned server's OpenAPI document, as its `/` returns it (R49).

opengrowth's own document (`opengrowth openapi`, src/server/openapi.rs) takes
its schemas from this one, since opengrowth implements the same contract byte
for byte, and tests/openapi.rs checks that the two differ only where listed.
"""

import contextlib
import io
import os
import sys

SERVER = os.environ["OPENGROWTH_REFERENCE_SERVER_SRC"]
sys.path.insert(0, SERVER)
os.chdir(SERVER)

from fastapi.testclient import TestClient  # noqa: E402

with contextlib.redirect_stdout(io.StringIO()):
    import main  # noqa: E402

from _common import write  # noqa: E402

response = TestClient(main.app).get("/")
assert response.status_code == 200, response.status_code
write(
    "http-openapi.json",
    "http_openapi.py",
    "The pinned server's OpenAPI document, as GET / returns it.",
    {"document": response.json()},
)
