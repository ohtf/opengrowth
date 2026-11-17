# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""SciPy normal CDF and PPF over a grid spanning ordinary values and the extreme tails (R5, Q4, Q5)."""

import numpy as np
from scipy import stats

from _common import write

z_values = sorted(
    set(np.round(np.arange(-40.0, 40.0001, 0.01), 10).tolist())
    | set(np.linspace(-8.5, 8.5, 4001).tolist())
    | {-37.5, -15.0, -8.0, -4.0, -0.0, 0.0, 4.0, 8.0, 15.0}
)
cdf_cases = [{"z": z, "cdf": float(stats.norm.cdf(z))} for z in z_values]

p_values = sorted(
    set(np.linspace(1e-6, 1 - 1e-6, 4001).tolist())
    | {10.0 ** -k for k in range(1, 308)}
    | {1 - 10.0 ** -k for k in range(1, 16)}
    | {0.004, 0.02, 0.09, 0.25, 0.5, 0.75, 0.91, 0.98, 0.996}
)
ppf_cases = [{"p": p, "ppf": float(stats.norm.ppf(p))} for p in p_values]

write(
    "normal-grid.json",
    "normal_grid.py",
    "scipy.stats.norm.cdf at each z and scipy.stats.norm.ppf at each p.",
    {"cdf": cdf_cases, "ppf": ppf_cases},
)
