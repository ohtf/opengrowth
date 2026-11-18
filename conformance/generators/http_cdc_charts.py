# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /cdc/chart-coordinates. See _http_charts.py."""

from _http_charts import generate

generate("cdc", "CDC", "http-cdc-charts.json", "http_cdc_charts.py")
