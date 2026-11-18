# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /trisomy-21-aap/fictional-child-data. See _http_fictional_child.py."""

from _http_fictional_child import generate

generate("trisomy-21-aap", "AAP Trisomy 21", "http-trisomy-21-aap-fictional-child.json", "http_trisomy_21_aap_fictional_child.py")
