# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""CPython round(x, n) for tie cases, near-ties, and a reproducible random sample."""

import random

from _common import write

values = [2.675, 0.125, 0.375, 2.5, 3.5, -0.5, -2.5, 1.00005, 12.34565, 0.038329911, 1e-5, -1e-5, 123456.78905]
values += [k / 2 ** n for n in range(1, 12) for k in range(-9, 10, 2)]
values += [k / 10 ** 5 + 0.000005 for k in range(0, 2000, 7)]
rng = random.Random(20261117)
values += [rng.uniform(-50, 50) for _ in range(800)]
values += [rng.uniform(-0.5, 25) for _ in range(800)]

cases = [{"x": x, "ndigits": n, "rounded": round(x, n)} for x in values for n in (0, 1, 2, 3, 4)]
write("python-round.json", "python_round.py", "CPython round(x, ndigits).", cases)
