# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""CPython `date + timedelta(days=x)` for float x, as fictional child data uses it.

CPython rounds the float to whole microseconds, half to even, and the date
then moves by the whole days only, so the result is not simply floor(x).
Cases are the steps the generator takes, values either side of whole days and
of half-microsecond ties, and the overflow limits. `result` is null where
CPython raises.
"""

import random
from datetime import date, timedelta

from _common import write

BIRTH = date(1759, 4, 11)
UNITS = {"days": 365.25, "weeks": 52, "months": 12, "years": 1}
HALF_US = 0.5 / 86_400_000_000

values = []
for unit, per_year in UNITS.items():
    for n in list(range(1, 60)) + [100, 365, 1000]:
        values.append(n / per_year * 365.25)
rng = random.Random(20261118)
values += [rng.uniform(-0.33, 20.5) * 365.25 for _ in range(400)]
for k in [0, 1, 2, 7, 608, 7305, -1, -2, -608]:
    for offset in [0.0, 1e-13, -1e-13, 1e-12, -1e-12, HALF_US, -HALF_US, 2 * HALF_US, -2 * HALF_US, 0.5, -0.5, 0.999999, -0.999999]:
        values.append(k + offset)
values += [k * 0.25 for k in range(-12, 13)]
values += [-642_000.0, -643_000.0, 3_010_000.0, 3_020_000.0, 999_999_999.0, 1_000_000_000.0, -999_999_999.5, 1e15, -1e15, 1e300]

cases = []
for x in values:
    try:
        result = (BIRTH + timedelta(days=x)).isoformat()
    except (OverflowError, ValueError):
        result = None
    cases.append({"days": x, "result": result})

write("python-timedelta.json", "python_timedelta.py", "CPython date(1759, 4, 11) + timedelta(days=x) for float x; null where CPython raises.", cases)
