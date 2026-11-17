# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Benchmark round 0 (spec/benchmark.md): the Python engine, in-process.

Run with the reference environment: s/bench-round0
"""

import contextlib
import io
import json
import time
from datetime import datetime
from pathlib import Path

from rcpchgrowth import Measurement
from rcpchgrowth.date_calculations import chronological_decimal_age, corrected_decimal_age
from rcpchgrowth.global_functions import sds_for_measurement

rows = json.loads((Path(__file__).resolve().parents[2] / "conformance/independent/uk-who-cole-r-vectors.json").read_text())

inputs = []
for r in rows:
    birth = datetime.strptime(r["birth_date"], "%d/%m/%Y")
    observed = datetime.strptime(r["observation_date"], "%d/%m/%Y")
    for age in (
        chronological_decimal_age(birth, observed),
        corrected_decimal_age(birth, observed, r["gestation_weeks"], r["gestation_days"]),
    ):
        inputs.append((age, r["measurement_method"], float(r["observation_value"]), r["sex"]))


def sds_loop():
    for age, method, value, sex in inputs:
        try:
            sds_for_measurement(reference="uk-who", age=age, measurement_method=method, observation_value=value, sex=sex)
        except LookupError:
            pass


def measurement_loop():
    # Upstream prints from inside percentage_median_bmi; keep that out of the timing output.
    with contextlib.redirect_stdout(io.StringIO()):
        for r in rows:
            Measurement(
                sex=r["sex"],
                birth_date=datetime.strptime(r["birth_date"], "%d/%m/%Y"),
                observation_date=datetime.strptime(r["observation_date"], "%d/%m/%Y"),
                measurement_method=r["measurement_method"],
                observation_value=float(r["observation_value"]),
                gestation_weeks=r["gestation_weeks"],
                gestation_days=r["gestation_days"],
                reference="uk-who",
            )


def best_of(fn, repeats, count):
    best = float("inf")
    for _ in range(repeats):
        start = time.perf_counter()
        fn()
        best = min(best, (time.perf_counter() - start) / count)
    return best * 1e9


ns = best_of(sds_loop, 5, len(inputs))
print(f"python sds_for_measurement: {ns:>10.1f} ns per SDS ({1e9 / ns:.0f} per second, 1 core)")
ns = best_of(measurement_loop, 3, len(rows))
print(f"python Measurement (full):  {ns:>10.1f} ns per measurement, 2 SDS + extras ({1e9 / ns:.0f} per second, 1 core)")
