# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""HTTP goldens for AAP Trisomy 21 calculation and bulk-calculation (Profile B, R15, R20).

The shared cases (_http_reference.py), whose bone-age case checks that the
AAP routes drop bone age, plus the reference's joins: height and head
circumference from 1 month, BMI from 2 years, the infant table to 3 years,
and 20 years; an SDS sweep; and preterm gestations.
"""

from datetime import date, timedelta

from _http_reference import generate

METHODS = ("height", "weight", "bmi", "ofc")


def extra(call, request, plausible_value):
    cases = []
    birth = date(2004, 5, 1)
    for method in METHODS:
        for sex in ("male", "female"):
            for days_old in (0, 20, 30, 31, 32, 729, 730, 731, 1095, 1096, 1097, 1200, 7304, 7305, 7306):
                observed = birth + timedelta(days=days_old)
                value = plausible_value(method, sex, birth, observed, 40, 0)
                cases.append(call(f"aap age {method} {sex} {days_old}d", body=request(birth, observed, method, sex, value), full=False))
            for days_old in (60, 1200, 5000):
                observed = birth + timedelta(days=days_old)
                for sds in (-6.0, -3.0, 2.0, 4.0, 6.0):
                    value = plausible_value(method, sex, birth, observed, 40, 0, sds)
                    cases.append(call(f"aap sds {method} {sex} {days_old}d {sds}", body=request(birth, observed, method, sex, value), full=False))
    for weeks, days in ((24, 0), (30, 2), (36, 6), (42, 0)):
        for days_old in (20, 100, 800):
            observed = birth + timedelta(days=days_old)
            value = plausible_value("weight", "female", birth, observed, weeks, days)
            cases.append(call(f"aap gestation {weeks}+{days} at {days_old}d", body=request(birth, observed, "weight", "female", value, weeks, days), full=(weeks, days_old) == (30, 100)))
    return cases


generate("trisomy-21-aap", "AAP Trisomy 21", "http-trisomy-21-aap.json", "http_trisomy_21_aap.py", extra)
