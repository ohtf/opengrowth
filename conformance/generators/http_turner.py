# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for Turner syndrome calculation and bulk-calculation (Profile B, R15, R20).

The shared cases (_http_reference.py), which are mostly refused because
Turner covers only girls' height, plus girls' height across 1 to 20 years,
an SDS sweep, gestations, and bulk.
"""

from datetime import date, timedelta

from _http_reference import generate


def extra(call, request, plausible_value):
    cases = []
    birth = date(2010, 3, 15)
    for days_old in (300, 364, 365, 366, 400, 730, 1826, 3653, 5479, 6574, 7300, 7304, 7305, 7306, 7400):
        observed = birth + timedelta(days=days_old)
        for sds in (-6.0, -3.0, -1.0, 0.0, 1.5, 3.0, 6.0):
            value = plausible_value("height", "female", birth, observed, 40, 0, sds)
            cases.append(call(f"turner girl {days_old}d {sds}", body=request(birth, observed, "height", "female", value), full=False))
    for weeks, days in ((24, 0), (30, 2), (36, 6), (42, 0)):
        for days_old in (365, 400, 800):
            observed = birth + timedelta(days=days_old)
            value = plausible_value("height", "female", birth, observed, weeks, days)
            cases.append(call(f"turner girl {weeks}+{days} at {days_old}d", body=request(birth, observed, "height", "female", value, weeks, days), full=days_old == 800 and weeks == 30))
    observed = birth + timedelta(days=2000)
    for method in ("weight", "bmi", "ofc"):
        cases.append(call(f"turner girl {method}", body=request(birth, observed, method, "female", 20.0)))
    cases.append(call("turner boy", body=request(birth, observed, "height", "male", 110.0)))
    bulk = {"measurement_method": "height", "birth_date": "2018-04-12", "sex": "female"}
    observations = [{"observation_date": "2024-06-12", "observation_value": 78}, {"observation_date": "2024-08-12", "observation_value": 80}, {"observation_date": "2040-08-12", "observation_value": 150}]
    cases.append(call("turner bulk", path="/turner/bulk-calculation", body={**bulk, "observations": observations}))
    cases.append(call("turner bulk boy", path="/turner/bulk-calculation", body={**bulk, "sex": "male", "observations": observations}))
    cases.append(call("turner bulk weight", path="/turner/bulk-calculation", body={**bulk, "measurement_method": "weight", "observations": observations}))
    return cases


generate("turners-syndrome", "Turner", "http-turner.json", "http_turner.py", extra, prefix="/turner")
