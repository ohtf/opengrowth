# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens from the FastAPI reference server for UK-WHO (Profile B, R15, R20).

The cases are in _http_reference.py, shared with the other references.
"""

from _http_reference import generate

generate("uk-who", "UK-WHO", "http-uk-who.json", "http_uk_who.py")
