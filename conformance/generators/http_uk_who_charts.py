# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /uk-who/chart-coordinates (R13, R15). See _http_charts.py."""

from _http_charts import generate

generate("uk-who", "UK-WHO", "http-uk-who-charts.json", "http_uk_who_charts.py")
