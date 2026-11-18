# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens from the FastAPI reference server for CDC (Profile B, R15, R20).

The UK-WHO cases (_http_reference.py), sent to the CDC routes, plus cases for
what CDC does differently: its joins at 2 years, 3 years for head
circumference, and 20 years; the extended BMI method above the 95th centile;
no data before term; and the rule that treats 37 to 42 weeks as term and
stops correcting at 2 years.
"""

from datetime import date, timedelta

from _http_reference import generate

METHODS = ("height", "weight", "bmi", "ofc")


def extra(call, request, plausible_value):
    cases = []
    birth = date(2020, 1, 31)
    for method in METHODS:
        for sex in ("male", "female"):
            for days_old in (729, 730, 731, 1095, 1096, 1097, 1100, 7304, 7305, 7306):
                observed = birth + timedelta(days=days_old)
                value = plausible_value(method, sex, birth, observed, 40, 0)
                cases.append(call(f"cdc age {method} {sex} {days_old}d", body=request(birth, observed, method, sex, value), full=False))
            for days_old in (60, 800, 2200, 5000, 7000):
                observed = birth + timedelta(days=days_old)
                for sds in (-5.0, -3.0, 1.6, 1.7, 2.5, 3.5, 5.0, 7.0):
                    value = plausible_value(method, sex, birth, observed, 40, 0, sds)
                    cases.append(call(f"cdc sds {method} {sex} {days_old}d {sds}", body=request(birth, observed, method, sex, value), full=False))
    for weeks, days in ((24, 0), (30, 2), (36, 6), (37, 0), (41, 6), (42, 0), (42, 1), (43, 0)):
        for days_old in (20, 100, 729, 730, 800, 1500):
            observed = birth + timedelta(days=days_old)
            value = plausible_value("weight", "male", birth, observed, weeks, days)
            full = (weeks, days_old) in ((30, 20), (30, 800))
            cases.append(call(f"cdc correction {weeks}+{days} at {days_old}d", body=request(birth, observed, "weight", "male", value, weeks, days), full=full))
    return cases


generate("cdc", "CDC", "http-cdc.json", "http_cdc.py", extra)
