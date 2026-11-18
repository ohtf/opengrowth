# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /cdc/fictional-child-data. See _http_fictional_child.py."""

from _http_fictional_child import generate

generate("cdc", "CDC", "http-cdc-fictional-child.json", "http_cdc_fictional_child.py")
