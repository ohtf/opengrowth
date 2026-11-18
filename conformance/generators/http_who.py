# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens from the FastAPI reference server for WHO (Profile B, R15, R20).

The UK-WHO cases (_http_reference.py), sent to the WHO routes, plus cases for
what WHO does differently: its joins at 1856 days, 10 years for weight, and
19 years; its restricted LMS method beyond 3 SD; and its gestation rules,
which treat 37 to 42 weeks as term and stop correcting at 2 years.
"""

from datetime import date, timedelta

from _http_reference import generate

METHODS = ("height", "weight", "bmi", "ofc")


def extra(call, request, plausible_value):
    cases = []
    birth = date(2020, 1, 31)
    for method in METHODS:
        for sex in ("male", "female"):
            for days_old in (1855, 1856, 1857, 1858, 1880, 3652, 3653, 3654, 6939, 6940, 6941):
                observed = birth + timedelta(days=days_old)
                value = plausible_value(method, sex, birth, observed, 40, 0)
                cases.append(call(f"who age {method} {sex} {days_old}d", body=request(birth, observed, method, sex, value), full=False))
            for days_old in (30, 400, 1200, 2200, 3300, 5000):
                observed = birth + timedelta(days=days_old)
                for sds in (-6.5, -4.2, -3.3, -3.0, 3.0, 3.3, 4.2, 6.5):
                    value = plausible_value(method, sex, birth, observed, 40, 0, sds)
                    cases.append(call(f"who sds {method} {sex} {days_old}d {sds}", body=request(birth, observed, method, sex, value), full=False))
    for weeks, days in ((30, 2), (36, 6), (37, 0), (41, 6), (42, 0), (42, 1), (43, 0)):
        for days_old in (100, 729, 730, 800, 1500):
            observed = birth + timedelta(days=days_old)
            value = plausible_value("weight", "female", birth, observed, weeks, days)
            full = weeks == 30 and days_old == 800
            cases.append(call(f"who correction {weeks}+{days} at {days_old}d", body=request(birth, observed, "weight", "female", value, weeks, days), full=full))
    return cases


generate("who", "WHO", "http-who.json", "http_who.py", extra)
