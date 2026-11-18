# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /turner/chart-coordinates. See _http_charts.py."""

from _http_charts import generate

generate("turners-syndrome", "Turner's syndrome", "http-turner-charts.json", "http_turner_charts.py", prefix="/turner")
