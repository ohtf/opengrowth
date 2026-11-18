# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /who/chart-coordinates (R13, R15). See _http_charts.py."""

from _http_charts import generate

generate("who", "WHO", "http-who-charts.json", "http_who_charts.py")
