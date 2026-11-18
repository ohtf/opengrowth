# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /trisomy-21-aap/chart-coordinates. See _http_charts.py."""

from _http_charts import generate

generate("trisomy-21-aap", "AAP Trisomy 21", "http-trisomy-21-aap-charts.json", "http_trisomy_21_aap_charts.py")
