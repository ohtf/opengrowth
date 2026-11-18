# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Python round() on a NumPy float64, and round(x) with no ndigits.

Upstream rounds SciPy results, which are NumPy float64 values, and NumPy's
rounding differs from CPython's where scaling lands on a tie (src/round.rs).
Five-decimal values like table medians are where that happens most.
"""

import random

import numpy as np

from _common import write

values = [2.675, 0.125, 0.375, 2.5, 3.5, -0.5, -2.5, 1.00005, 12.34565, 0.038329911, 1e-5, -1e-5, 123456.78905, 42.34725]
values += [k / 2 ** n for n in range(1, 12) for k in range(-9, 10, 2)]
rng = random.Random(20261118)
values += [round(rng.uniform(0, 200), 5) for _ in range(700)]
values += [rng.uniform(-50, 50) for _ in range(500)]
values += [rng.uniform(-0.01, 0.01) for _ in range(150)]

cases = []
for i, x in enumerate(values):
    for n in range(9) if i < 250 else range(5):
        rounded = round(np.float64(x), n)
        assert type(rounded) is np.float64
        cases.append({"x": x, "ndigits": n, "rounded": float(rounded)})
    # With no ndigits, a Python float and a float64 both round to an int.
    builtin, numpy = round(x), round(np.float64(x))
    assert type(builtin) is int and type(numpy) is int and builtin == numpy
    cases.append({"x": x, "ndigits": None, "rounded": float(builtin)})

write("numpy-round.json", "numpy_round.py", "Python round(x, ndigits) on a NumPy float64; round(x) with ndigits null.", cases)
