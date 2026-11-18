# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for POST /turner/fictional-child-data. See _http_fictional_child.py.

The shared cases are mostly refused, since Turner covers only girls' height,
so girls' height series are added.
"""

from _http_fictional_child import generate

G = {"measurement_method": "height", "sex": "female"}
EXTRA = [
    ("turner girl defaults", G, False),
    ("turner girl from 1 year", {**G, "start_chronological_age": 1}, True),
    ("turner girl yearly drift", {**G, "start_chronological_age": 1, "measurement_interval_type": "y", "measurement_interval_number": 1, "drift": True, "drift_range": -1.0, "start_sds": 0.5}, False),
    ("turner girl noise", {**G, "start_chronological_age": 2, "noise": True, "noise_range": 0.02}, False),
    ("turner girl preterm", {**G, "gestation_weeks": 30, "start_chronological_age": 1.5, "end_age": 4}, False),
    ("turner girl past 20", {**G, "start_chronological_age": 18, "end_age": 25}, False),
    ("turner girl sds 9", {**G, "start_chronological_age": 2, "start_sds": 9}, False),
    ("turner girl weight", {**G, "measurement_method": "weight"}, True),
]

generate("turners-syndrome", "Turner", "http-turner-fictional-child.json", "http_turner_fictional_child.py", EXTRA, prefix="/turner")
