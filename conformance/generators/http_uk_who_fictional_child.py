# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /uk-who/fictional-child-data. See _http_fictional_child.py."""

from _http_fictional_child import generate

generate("uk-who", "UK-WHO", "http-uk-who-fictional-child.json", "http_uk_who_fictional_child.py")
