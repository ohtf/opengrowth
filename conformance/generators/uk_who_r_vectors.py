# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run the Python reference engine over the independent R vectors.

Uses the full Measurement pipeline, as rcpchgrowth's own
test_uk_who_reference_integration does, and records what Python computes for
each case. The R values themselves are not copied here; they stay in
conformance/independent/ as the independent oracle.
"""

import json
from datetime import datetime

from rcpchgrowth import Measurement

from _common import INDEPENDENT, write

rows = json.loads((INDEPENDENT / "uk-who-cole-r-vectors.json").read_text())
cases = []
for index, row in enumerate(rows):
    measurement = Measurement(
        sex=row["sex"],
        birth_date=datetime.strptime(row["birth_date"], "%d/%m/%Y"),
        observation_date=datetime.strptime(row["observation_date"], "%d/%m/%Y"),
        measurement_method=row["measurement_method"],
        observation_value=float(row["observation_value"]),
        gestation_weeks=int(row["gestation_weeks"]),
        gestation_days=int(row["gestation_days"]),
        reference="uk-who",
    ).measurement
    dates = measurement["measurement_dates"]
    values = measurement["measurement_calculated_values"]
    cases.append(
        {
            "index": index,
            "chronological_decimal_age": dates["chronological_decimal_age"],
            "corrected_decimal_age": dates["corrected_decimal_age"],
            "chronological_sds": values["chronological_sds"],
            "corrected_sds": values["corrected_sds"],
            "chronological_centile": values["chronological_centile"],
            "corrected_centile": values["corrected_centile"],
            "chronological_measurement_error": values["chronological_measurement_error"],
            "corrected_measurement_error": values["corrected_measurement_error"],
        }
    )

write(
    "uk-who-r-vector-inputs.json",
    "uk_who_r_vectors.py",
    "rcpchgrowth Measurement output for every input in independent/uk-who-cole-r-vectors.json, by index.",
    cases,
)
