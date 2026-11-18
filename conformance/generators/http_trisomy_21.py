# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for UK Trisomy 21 calculation and bulk-calculation (Profile B, R15, R20).

The shared cases (_http_reference.py), plus the reference's own limits: head
circumference to 18 years, BMI to 18.82 (two table rows beyond it are
unreachable), and 20 years; an SDS sweep; and preterm gestations.
"""

from datetime import date, timedelta

from _http_reference import generate

METHODS = ("height", "weight", "bmi", "ofc")


def extra(call, request, plausible_value):
    cases = []
    birth = date(2004, 5, 1)
    for method in METHODS:
        for sex in ("male", "female"):
            for days_old in (0, 30, 400, 6574, 6575, 6576, 6870, 6873, 6874, 6875, 6911, 7304, 7305, 7306):
                observed = birth + timedelta(days=days_old)
                value = plausible_value(method, sex, birth, observed, 40, 0)
                cases.append(call(f"t21 age {method} {sex} {days_old}d", body=request(birth, observed, method, sex, value), full=False))
            for days_old in (60, 1200, 5000):
                observed = birth + timedelta(days=days_old)
                for sds in (-6.0, -3.0, 2.0, 4.0, 6.0):
                    value = plausible_value(method, sex, birth, observed, 40, 0, sds)
                    cases.append(call(f"t21 sds {method} {sex} {days_old}d {sds}", body=request(birth, observed, method, sex, value), full=False))
    for weeks, days in ((24, 0), (30, 2), (36, 6), (42, 0)):
        for days_old in (20, 100, 800):
            observed = birth + timedelta(days=days_old)
            value = plausible_value("weight", "male", birth, observed, weeks, days)
            cases.append(call(f"t21 gestation {weeks}+{days} at {days_old}d", body=request(birth, observed, "weight", "male", value, weeks, days), full=(weeks, days_old) == (30, 20)))
    return cases


generate("trisomy-21", "Trisomy 21", "http-trisomy-21.json", "http_trisomy_21.py", extra)
