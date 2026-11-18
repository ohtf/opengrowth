# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Engine conformance cases from rcpchgrowth 4.6.5, for any implementation.

One file per growth reference, engine-<reference>.json, with two sections:

- sds: sds_for_measurement and centile at a decimal age that a whole number
  of days gives, as real dates do. Each case also names the data file the
  SDS came from, or carries upstream's message where there is no data.
- measurement: the Measurement class, from dates and gestation, as upstream
  returns it (birth_data, measurement_dates, child_observation_value,
  measurement_calculated_values), with the data file each SDS came from.

engine-centile-bands.json holds centile_band_for_centile either side of
every band edge, for every centile format and measurement method.

Cases sit at every boundary each reference's rules define: every join
between tables, every age or gestation where data stop, and every gestation
rule, at the boundary and one day either side, for every measurement and
sex. Sweeps cross the SDS ranges where a reference changes method (WHO
beyond 3 SD, CDC BMI above the 95th centile) and the observation check at
8 SD (15 for BMI). Boundaries come from upstream's own constants where it
has them.

Expected values are rcpchgrowth's own, at full precision. They record what
rcpchgrowth does, not what is clinically right; the independent R vectors
are the clinical-accuracy gate (spec/standard.md section 8).
"""

import contextlib
import io
import json
import math
import numbers
from datetime import date, timedelta
from pathlib import Path

import rcpchgrowth
from rcpchgrowth import Measurement, centile, centile_band_for_centile, measurement_from_sds, sds_for_measurement
from rcpchgrowth import constants as C
from rcpchgrowth.centile_bands import generate_centile_band_ranges
from rcpchgrowth.global_functions import lms_value_array_for_measurement_for_reference

from _common import write

METHODS = ("height", "weight", "ofc", "bmi")
SEXES = ("male", "female")
TYPICAL = {"height": 100.0, "weight": 15.0, "ofc": 45.0, "bmi": 16.0}

# (description, decimal age in years) for every rule boundary, from upstream's
# constants where it has one.
BOUNDARIES = {
    "uk-who": [
        ("23 weeks' gestation, where UK90 preterm data start", C.UK90_REFERENCE_LOWER_THRESHOLD),
        ("25 weeks' gestation, where length data start", C.TWENTY_FIVE_WEEKS_GESTATION),
        ("term", C.ZERO_YEARS),
        ("42 weeks' gestation, from UK90 preterm to WHO 2006, where BMI data start", C.UK_WHO_INFANT_LOWER_THRESHOLD),
        ("2 years, from the WHO 2006 infant table to the child table", C.WHO_CHILD_LOWER_THRESHOLD),
        ("4 years, from WHO 2006 to UK90", C.WHO_CHILDREN_UPPER_THRESHOLD),
        ("17 years, where girls' head circumference data stop", C.SEVENTEEN_YEARS),
        ("18 years, where boys' head circumference data stop", C.EIGHTEEN_YEARS),
        ("20 years, where UK90 data stop", C.UK90_UPPER_THRESHOLD),
    ],
    "who": [
        ("term, where WHO data start", C.ZERO_YEARS),
        ("2 years, from the WHO 2006 infant table to the child table", C.WHO_CHILD_LOWER_THRESHOLD),
        ("5 years, where the restricted LMS method stops for height and head circumference", C.FIVE_YEARS),
        ("1856 days, from WHO 2006 to WHO 2007, where head circumference data stop", C.WHO_2006_REFERENCE_UPPER_THRESHOLD),
        ("10 years, where weight data stop", C.TEN_YEARS),
        ("19 years, where WHO data stop", C.WHO_2007_REFERENCE_UPPER_THRESHOLD),
    ],
    "cdc": [
        ("22 weeks' gestation, the start of the empty Fenton segment", C.FENTON_LOWER_THRESHOLD),
        ("term, below which CDC has no data", C.ZERO_YEARS),
        ("2 years, from WHO 2006 to CDC 2000, where BMI data start", C.CDC_LOWER_THRESHOLD),
        ("3 years, where CDC infant head circumference data stop", C.THREE_YEARS),
        ("20 years, where CDC data stop", C.CDC_UPPER_THRESHOLD),
    ],
    "turners-syndrome": [
        ("1 year, where Turner data start (turner.py: age < 1)", 1.0),
        ("20 years, where Turner data stop", C.TWENTY_YEARS),
    ],
    "trisomy-21": [
        ("term, where Trisomy 21 data start", C.ZERO_YEARS),
        ("18 years, where head circumference data stop", C.EIGHTEEN_YEARS),
        ("18.82 years, where BMI data stop (trisomy_21.py: age > 18.82)", 18.82),
        ("20 years, where Trisomy 21 data stop", C.TWENTY_YEARS),
    ],
    "trisomy-21-aap": [
        ("term, where AAP Trisomy 21 data start", C.ZERO_YEARS),
        ("1 month, where length and head circumference data start", C.TRISOMY_21_AAP_INFANT_AGES[0]),
        ("2 years, where BMI data start", C.TWO_YEARS),
        ("3 years, from the infant table to the child table", C.THREE_YEARS),
        ("20 years, where AAP Trisomy 21 data stop", C.TWENTY_YEARS),
    ],
}

INTERIOR_DAYS = [-90, -30, 7, 60, 180, 365, 548, 1000, 1300, 1700, 2200, 3000, 4000, 5000, 5800, 6400, 7000]
SWEEP_DAYS = {
    "uk-who": [-60, 100, 1000, 3000, 6000],
    "who": [30, 400, 1000, 1500, 1826, 1827, 2200, 3300, 5000, 6500],
    "cdc": [100, 1000, 3000, 6000],
    "turners-syndrome": [1000, 4000],
    "trisomy-21": [100, 2000, 5000],
    "trisomy-21-aap": [100, 2000, 5000],
}
SWEEP_SDS = [-7.0, -5.0, -4.0, -3.5, -3.01, -3.0, -2.99, 2.99, 3.0, 3.01, 3.5, 4.0, 5.0, 7.0]
# Either side of the 95th centile, where CDC BMI changes method.
CDC_BMI_SDS = [1.6, 1.64, 1.645, 1.65, 1.7, 2.0, 2.5]

# Gestations either side of every gestation rule: 22, 23, and 25 weeks, the
# 32-week and 37-week comment thresholds, term, and 42 weeks.
GESTATIONS = [(22, 0), (22, 6), (23, 0), (24, 6), (25, 0), (27, 3), (30, 2), (31, 6), (32, 0), (34, 6), (36, 6), (37, 0), (39, 6), (40, 0), (40, 1), (41, 6), (42, 0), (42, 1), (43, 6), (44, 6)]
SYNDROME_GESTATIONS = [(24, 0), (31, 6), (32, 0), (36, 6), (37, 0), (40, 0), (41, 6), (42, 1)]
MATRIX = {
    "uk-who": ("weight", "female", GESTATIONS),
    "who": ("weight", "male", GESTATIONS),
    "cdc": ("weight", "male", GESTATIONS),
    "turners-syndrome": ("height", "female", SYNDROME_GESTATIONS),
    "trisomy-21": ("weight", "female", SYNDROME_GESTATIONS),
    "trisomy-21-aap": ("weight", "male", SYNDROME_GESTATIONS),
}
BIRTH = date(2020, 1, 31)

TABLES = Path(rcpchgrowth.__file__).parent / "data_tables"
FILES = {p.relative_to(TABLES).as_posix(): json.loads(p.read_text()) for p in sorted(TABLES.rglob("*.json")) if "correlation" not in p.as_posix()}
_table_cache = {}


def quiet(f, *args, **kwargs):
    with contextlib.redirect_stdout(io.StringIO()):
        return f(*args, **kwargs)


def table_for(reference, age, method, sex):
    """The data file upstream's SDS calculation reads at this age."""
    rows = quiet(lms_value_array_for_measurement_for_reference, reference, age, method, sex, default_youngest_reference=False)
    if id(rows) not in _table_cache:
        found = [rel for rel, d in FILES.items() if ((d.get("measurement") or {}).get(method) or {}).get(sex) == rows]
        assert len(found) == 1, (reference, age, method, sex, found)
        _table_cache[id(rows)] = found[0]
    return _table_cache[id(rows)]


def days_around(years):
    """Whole days either side of an age: the last day before, and the first on or after, each one day further out too."""
    x = years * 365.25
    lo, hi = math.floor(x + 1e-9), math.ceil(x - 1e-9)
    return sorted({lo - 1, lo, hi, hi + 1})


def value_at(reference, sds, method, sex, age, places):
    """A measurement at this SDS, or None where upstream has none or it is not positive."""
    try:
        value = quiet(measurement_from_sds, reference, sds, method, sex, age)
    except Exception:
        return None
    if value is None or isinstance(value, complex) or not math.isfinite(value) or value <= 0:
        return None
    return round(float(value), places)


def plain(value):
    """Upstream's values as JSON: dates as ISO strings, NumPy floats as floats."""
    if isinstance(value, dict):
        return {k: plain(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [plain(v) for v in value]
    if isinstance(value, date):
        return value.isoformat()
    if isinstance(value, bool) or value is None or isinstance(value, str):
        return value
    if isinstance(value, numbers.Integral):
        return int(value)
    if isinstance(value, float):
        return float(value)
    return value


def sds_case(reference, method, sex, days, value, purpose, level):
    age = days / 365.25
    case = {
        "name": f"{purpose} | {method} {sex} | {days} days | {f'SDS {level:+}' if level is not None else 'value ' + repr(value)}",
        "purpose": purpose,
        "input": {"reference": reference, "measurement_method": method, "sex": sex, "age": age, "observation_value": value},
    }
    try:
        sds = quiet(sds_for_measurement, reference, age, method, value, sex)
    except LookupError as error:
        case["expected"] = {"error": str(error)}
        return case
    if sds is None or isinstance(sds, complex) or not math.isfinite(sds):
        return None
    case["expected"] = {"sds": float(sds), "centile": float(quiet(centile, sds)), "table": table_for(reference, age, method, sex)}
    return case


def sds_cases(reference):
    cases, seen = [], set()

    def add(method, sex, days, levels, purpose, places):
        age = days / 365.25
        values = [(level, value_at(reference, level, method, sex, age, places)) for level in levels]
        present = [(level, v) for level, v in values if v is not None]
        if not present:
            if purpose.startswith("boundary"):
                present = [(None, TYPICAL[method])]
            else:
                return
        for level, value in present:
            key = (method, sex, days, value)
            if key in seen:
                continue
            seen.add(key)
            case = sds_case(reference, method, sex, days, value, purpose, level)
            if case is not None:
                cases.append(case)

    for label, years in BOUNDARIES[reference]:
        for days in days_around(years):
            for method in METHODS:
                for sex in SEXES:
                    add(method, sex, days, [-2.2, 1.3], f"boundary: {label}", 2)
    for days in INTERIOR_DAYS:
        for method in METHODS:
            for sex in SEXES:
                add(method, sex, days, [-2.2, 1.3], "interior", 2)
    for days in SWEEP_DAYS[reference]:
        for method in METHODS:
            for sex in SEXES:
                add(method, sex, days, SWEEP_SDS, "SDS sweep", 3)
                if reference == "cdc" and method == "bmi":
                    add(method, sex, days, CDC_BMI_SDS, "SDS sweep: CDC BMI either side of the 95th centile", 3)
    return cases


def measurement_case(reference, birth, observed, weeks, days, sex, method, value, purpose):
    inputs = {
        "reference": reference,
        "birth_date": birth.isoformat(),
        "observation_date": observed.isoformat(),
        "gestation_weeks": weeks,
        "gestation_days": days,
        "sex": sex,
        "measurement_method": method,
        "observation_value": value,
    }
    case = {
        "name": f"{purpose} | {method} {sex} | born {birth} at {weeks}+{days} | measured {observed} | {value!r}",
        "purpose": purpose,
        "input": inputs,
    }
    try:
        m = quiet(Measurement, birth_date=birth, observation_date=observed, measurement_method=method, observation_value=value, reference=reference, sex=sex, gestation_weeks=weeks, gestation_days=days).measurement
    except Exception as error:
        case["expected"] = {"error": str(error)}
        return case
    expected = {k: plain(m[k]) for k in ("birth_data", "measurement_dates", "child_observation_value", "measurement_calculated_values")}
    values = expected["measurement_calculated_values"]
    ages = expected["measurement_dates"]
    expected["tables"] = {
        branch: table_for(reference, ages[f"{branch}_decimal_age"], method, sex) if values[f"{branch}_sds"] is not None else None
        for branch in ("chronological", "corrected")
    }
    case["expected"] = expected
    return case


def plausible(reference, method, sex, age, sds, places=2):
    """A measurement at this SDS for the age, or a typical one where there is no data."""
    return value_at(reference, sds, method, sex, age, places) or TYPICAL[method]


def measurement_cases(reference):
    cases = []
    method, sex, gestations = MATRIX[reference]
    for weeks, days in gestations:
        to_edd = 280 - (weeks * 7 + days)
        offsets = {0, 1, 729, 730, 731} | {to_edd + d for d in (-1, 0, 1, 13, 14, 15, 365, 730, 731, 1461)}
        for offset in sorted(o for o in offsets if o >= 0):
            observed = BIRTH + timedelta(days=offset)
            value = plausible(reference, method, sex, (offset - to_edd) / 365.25, 0.8)
            cases.append(measurement_case(reference, BIRTH, observed, weeks, days, sex, method, value, f"gestation {weeks}+{days}, {offset} days after birth ({offset - to_edd:+} from the due date)"))
    for method_ in METHODS:
        limit = C.MAXIMUM_BMI_ERROR_SDS if method_ == "bmi" else C.MAXIMUM_HEIGHT_WEIGHT_OFC_ERROR_SDS
        for sex_ in SEXES:
            for days_old in (1000, 3000) if reference == "turners-syndrome" else (400, 3000):
                for level in (-limit - 0.01, -limit + 0.01, limit - 0.01, limit + 0.01):
                    value = value_at(reference, level, method_, sex_, days_old / 365.25, 3)
                    if value is not None:
                        observed = BIRTH + timedelta(days=days_old)
                        cases.append(measurement_case(reference, BIRTH, observed, 40, 0, sex_, method_, value, f"observation check, SDS {level:+.2f}"))
            for days_old in (15, 731, 3653):
                value = value_at(reference, 1.3, method_, sex_, days_old / 365.25, 2)
                if value is not None and method_ == "bmi":
                    cases.append(measurement_case(reference, BIRTH, BIRTH + timedelta(days=days_old), 40, 0, sex_, method_, value, "BMI as a percentage of the median"))
    cases.append(measurement_case(reference, BIRTH, BIRTH - timedelta(days=1), 40, 0, sex, method, TYPICAL[method], "observation before birth"))
    if reference == "uk-who":
        for birth, observed in [
            (date(2020, 2, 29), date(2021, 2, 28)), (date(2020, 2, 29), date(2021, 3, 1)), (date(2020, 2, 29), date(2024, 2, 28)), (date(2020, 2, 29), date(2024, 2, 29)),
            (date(2020, 1, 31), date(2020, 2, 29)), (date(2020, 1, 31), date(2020, 3, 1)), (date(2020, 1, 31), date(2020, 3, 31)), (date(2020, 1, 31), date(2020, 4, 30)),
            (date(2019, 12, 31), date(2020, 12, 31)), (date(2019, 12, 31), date(2021, 1, 1)),
        ]:
            age = (observed - birth).days / 365.25
            cases.append(measurement_case(reference, birth, observed, 40, 0, "female", "weight", plausible(reference, "weight", "female", age, 0.8), "calendar age across month ends and leap days"))
    names = [c["name"] for c in cases]
    assert len(names) == len(set(names)), "duplicate measurement case names"
    return cases


def band_cases():
    formats = {
        C.COLE_TWO_THIRDS_SDS_NINE_CENTILES: C.COLE_TWO_THIRDS_SDS_NINE_CENTILE_COLLECTION,
        C.THREE_PERCENT_CENTILES: C.THREE_PERCENT_CENTILE_COLLECTION,
        C.FIVE_PERCENT_CENTILES: C.FIVE_PERCENT_CENTILE_COLLECTION,
        C.EIGHTY_FIVE_PERCENT_CENTILES: C.EIGHTY_FIVE_PERCENT_CENTILE_COLLECTION,
    }
    cases = []
    for name, collection in formats.items():
        ranges = generate_centile_band_ranges(collection)
        edges = sorted({e for pair in ranges for e in pair})
        limits = [C.MINIMUM_HEIGHT_WEIGHT_OFC_ADVISORY_SDS, C.MAXIMUM_HEIGHT_WEIGHT_OFC_ADVISORY_SDS, C.MINIMUM_BMI_ADVISORY_SDS, C.MAXIMUM_BMI_ADVISORY_SDS]
        points = set()
        for e in edges + [float(x) for x in limits]:
            points |= {math.nextafter(e, -math.inf), e, math.nextafter(e, math.inf)}
        points |= {(a + b) / 2 for a, b in ranges}
        points |= {0.0, -10.0, 10.0}
        for method in METHODS:
            for sds in sorted(points):
                band = quiet(centile_band_for_centile, sds, method, name)
                cases.append({
                    "name": f"{name} | {method} | SDS {sds!r}",
                    "input": {"sds": sds, "measurement_method": method, "centile_format": name},
                    "expected": {"centile_band": band},
                })
    return cases


DESCRIPTION = "rcpchgrowth 4.6.5 at every boundary {reference}'s rules define: sds_for_measurement and centile at whole-day ages (sds), and the Measurement class from dates and gestation (measurement)."

for reference in BOUNDARIES:
    write(f"engine-{reference}.json", "engine_cases.py", DESCRIPTION.format(reference=reference), {"sds": sds_cases(reference), "measurement": measurement_cases(reference)})
write("engine-centile-bands.json", "engine_cases.py", "rcpchgrowth 4.6.5 centile_band_for_centile either side of every band edge and advisory limit, for every centile format and measurement method.", band_cases())
