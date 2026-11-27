# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Benchmark round 0 (spec/benchmark.md): the Python engine, in-process.

The same work as examples/round0.rs: for every R-vector child, the UK-WHO SDS
at chronological and corrected age (`sds_for_measurement`), and the whole
calculation (`Measurement`), with ages and dates prepared beforehand. Each
figure is the best of several passes. `--json` prints them as JSON, for
s/bench.

Run with the reference environment: s/bench-round0
"""

import contextlib
import importlib.metadata
import io
import json
import sys
import time
from datetime import datetime
from pathlib import Path

from rcpchgrowth import Measurement
from rcpchgrowth.date_calculations import chronological_decimal_age, corrected_decimal_age
from rcpchgrowth.global_functions import sds_for_measurement

rows = json.loads((Path(__file__).resolve().parents[2] / "conformance/independent/uk-who-cole-r-vectors.json").read_text())

children = [
    {
        "sex": r["sex"],
        "birth_date": datetime.strptime(r["birth_date"], "%d/%m/%Y"),
        "observation_date": datetime.strptime(r["observation_date"], "%d/%m/%Y"),
        "measurement_method": r["measurement_method"],
        "observation_value": float(r["observation_value"]),
        "gestation_weeks": r["gestation_weeks"],
        "gestation_days": r["gestation_days"],
        "reference": "uk-who",
    }
    for r in rows
]

inputs = []
for c in children:
    for age in (
        chronological_decimal_age(c["birth_date"], c["observation_date"]),
        corrected_decimal_age(c["birth_date"], c["observation_date"], c["gestation_weeks"], c["gestation_days"]),
    ):
        inputs.append((age, c["measurement_method"], c["observation_value"], c["sex"]))


def sds_loop():
    for age, method, value, sex in inputs:
        try:
            sds_for_measurement(reference="uk-who", age=age, measurement_method=method, observation_value=value, sex=sex)
        except LookupError:
            pass


def measurement_loop():
    # Upstream prints from inside percentage_median_bmi; keep that out of the timing output.
    with contextlib.redirect_stdout(io.StringIO()):
        for child in children:
            Measurement(**child)


def best_of(fn, repeats, count):
    best = float("inf")
    for _ in range(repeats):
        start = time.perf_counter()
        fn()
        best = min(best, (time.perf_counter() - start) / count)
    return best * 1e9


sds_ns = best_of(sds_loop, 5, len(inputs))
measurement_ns = best_of(measurement_loop, 3, len(children))
if "--json" in sys.argv[1:]:
    print(json.dumps({
        "engine": "rcpchgrowth",
        "version": importlib.metadata.version("rcpchgrowth"),
        "python": sys.version.split()[0],
        "children": len(children),
        "sds_ns": sds_ns,
        "measurement_ns": measurement_ns,
    }))
else:
    print(f"python sds_for_measurement: {sds_ns:>10.1f} ns per SDS ({1e9 / sds_ns:.0f} per second, 1 core)")
    print(f"python Measurement (full):  {measurement_ns:>10.1f} ns per measurement, 2 SDS + extras ({1e9 / measurement_ns:.0f} per second, 1 core)")
