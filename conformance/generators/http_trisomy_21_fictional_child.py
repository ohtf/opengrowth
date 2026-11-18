# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /trisomy-21/fictional-child-data. See _http_fictional_child.py."""

from _http_fictional_child import generate

generate("trisomy-21", "Trisomy 21", "http-trisomy-21-fictional-child.json", "http_trisomy_21_fictional_child.py")
