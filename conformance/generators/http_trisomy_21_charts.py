# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /trisomy-21/chart-coordinates. See _http_charts.py."""

from _http_charts import generate

generate("trisomy-21", "Trisomy 21", "http-trisomy-21-charts.json", "http_trisomy_21_charts.py")
