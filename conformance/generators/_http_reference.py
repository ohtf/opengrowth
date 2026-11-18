# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Shared generator for a reference's calculation and bulk-calculation goldens.

Used by http_uk_who.py and http_who.py: each reference gets the same cases,
plus any of its own (Profile B, R15, R20).

Run by s/goldens inside the reference-server environment (s/reference-server).
Requests go through Starlette's TestClient, so the bytes are exactly what the
application produces; transport headers such as date and server are not
recorded (Profile C is not promised).

Bodies are recorded in full for curated cases. For the R-vector corpus only
SHA-256 digests are kept, to keep the repository small.

Both `body_sha256` and `masked_sha256` are recorded. The masked digest is of
the body with every "provenance" member removed (see mask_provenance), which
is the one designed difference (spec/standard.md section 4.2).
"""

import contextlib
import hashlib
import io
import json
import os
import sys
from datetime import date, datetime, timedelta

SERVER = os.environ["OPENGROWTH_REFERENCE_SERVER_SRC"]
sys.path.insert(0, SERVER)
os.chdir(SERVER)

from fastapi.testclient import TestClient  # noqa: E402

with contextlib.redirect_stdout(io.StringIO()):
    import main  # noqa: E402
from rcpchgrowth.global_functions import measurement_from_sds  # noqa: E402
from rcpchgrowth.date_calculations import corrected_decimal_age  # noqa: E402

from _common import INDEPENDENT, RECORDED_HEADERS, mask_provenance, write  # noqa: E402

client = TestClient(main.app, raise_server_exceptions=False)


def generate(reference, label, output, generator, extra=lambda call, request, plausible_value: []):
    """Write the golden for `reference` (also its route prefix), called `label` in descriptions."""
    prefix = f"/{reference}"


    def call(name, path=None, body=None, raw=None, method="POST", headers=None, full=True):
        if path is None:
            path = f"{prefix}/calculation"
        if raw is None:
            raw = json.dumps(body, ensure_ascii=False) if body is not None else ""
        request_headers = {"content-type": "application/json", **(headers or {})}
        with contextlib.redirect_stdout(io.StringIO()):
            response = client.request(
                method, path, content=raw.encode(), headers=request_headers, follow_redirects=False
            )
        text = response.content.decode()
        case = {
            "name": name,
            "method": method,
            "path": path,
            "request_headers": headers or {},
            "request_body": raw,
            "status": response.status_code,
            "headers": {h: response.headers[h] for h in RECORDED_HEADERS if h in response.headers},
            "body_sha256": hashlib.sha256(response.content).hexdigest(),
            "masked_sha256": hashlib.sha256(mask_provenance(text).encode()).hexdigest(),
        }
        if full:
            case["body"] = text
        return case


    def plausible_value(method, sex, birth, observed, weeks, days, sds=0.5):
        """A measurement at `sds` for the corrected age, so the request passes validation."""
        try:
            age = corrected_decimal_age(birth, observed, weeks, days)
            with contextlib.redirect_stdout(io.StringIO()):
                value = measurement_from_sds(reference, sds, method, sex, age)
            return round(value, 2)
        except Exception:
            return {"height": 80.0, "weight": 10.0, "bmi": 16.0, "ofc": 45.0}[method]


    def request(birth, observed, method, sex, value, weeks=40, days=0, **extra):
        return {
            "birth_date": birth.isoformat(),
            "observation_date": observed.isoformat(),
            "observation_value": value,
            "sex": sex,
            "gestation_weeks": weeks,
            "gestation_days": days,
            "measurement_method": method,
            **extra,
        }


    cases = []
    BASE = request(date(2020, 4, 12), date(2028, 6, 12), "height", "female", 115)

    # Curated single calculations: every measurement, both sexes, across the
    # UK-WHO joins and the gestation range.
    birth = date(2020, 1, 31)
    ages_in_days = [0, 1, 6, 7, 13, 14, 15, 30, 59, 365, 729, 730, 731, 1095, 1460, 1461, 1462, 3653, 6209, 6240, 6575, 6750, 7304, 7305, 7306]
    gestations = [(22, 0), (23, 0), (24, 6), (25, 0), (27, 3), (32, 4), (36, 6), (37, 0), (39, 6), (40, 0), (40, 1), (41, 3), (42, 0), (42, 1), (43, 6), (44, 6)]
    for method in ("height", "weight", "bmi", "ofc"):
        for sex in ("male", "female"):
            for days_old in ages_in_days:
                observed = birth + timedelta(days=days_old)
                value = plausible_value(method, sex, birth, observed, 40, 0)
                cases.append(call(f"age {method} {sex} {days_old}d", body=request(birth, observed, method, sex, value), full=False))
            for weeks, days in gestations:
                for days_old in (0, 7, 14, 60, 120, 365, 730, 913):
                    observed = birth + timedelta(days=days_old)
                    value = plausible_value(method, sex, birth, observed, weeks, days)
                    cases.append(
                        call(
                            f"gestation {method} {sex} {weeks}+{days} at {days_old}d",
                            body=request(birth, observed, method, sex, value, weeks, days),
                            full=False,
                        )
                    )

    # Month ends and leap days, for calendar-age phrasing.
    for b, o in [
        (date(2020, 1, 31), date(2020, 2, 29)),
        (date(2020, 1, 31), date(2020, 3, 1)),
        (date(2020, 2, 29), date(2021, 2, 28)),
        (date(2020, 2, 29), date(2021, 3, 1)),
        (date(2019, 12, 31), date(2020, 6, 30)),
        (date(2020, 3, 31), date(2020, 4, 30)),
        (date(2020, 5, 15), date(2020, 5, 16)),
        (date(2020, 5, 15), date(2020, 5, 22)),
        (date(2020, 5, 15), date(2020, 5, 23)),
        (date(2020, 5, 15), date(2020, 6, 16)),
    ]:
        cases.append(call(f"calendar {b} {o}", body=request(b, o, "weight", "male", plausible_value("weight", "male", b, o, 40, 0))))

    # Chronological age has no data but corrected age does.
    cases.append(call("ofc chronological absent", body=request(date(2009, 1, 1), date(2026, 2, 20), "ofc", "female", 55.0, 30, 0)))

    # Passthrough fields.
    cases.append(
        call(
            "bone age and events",
            body={**BASE, "bone_age": 10, "bone_age_type": "greulich-pyle", "bone_age_sds": 2.0, "bone_age_centile": 98, "bone_age_text": "This bone age is advanced", "events_text": ["Growth hormone start", "Growth Hormone Deficiency diagnosis"]},
        )
    )
    cases.append(call("events mixed", body={**BASE, "events_text": ["a", {"b": 1.0, "a": [1, 2.5]}, 2, None, True, "kg/m² é \u2028"]}))

    # Application-level 422s.
    for name, overrides in [
        ("plus 8 sd height", {"observation_value": 300}),
        ("minus 8 sd height", {"observation_value": 30}),
        ("plus 8 sd weight", {"measurement_method": "weight", "observation_value": 200}),
        ("minus 8 sd ofc", {"measurement_method": "ofc", "observation_value": 20}),
        ("plus 15 sd bmi", {"measurement_method": "bmi", "observation_value": 90}),
        ("minus 15 sd bmi", {"measurement_method": "bmi", "observation_value": 3}),
        ("plus 8 sd at birth", {"birth_date": "2028-06-12", "measurement_method": "weight", "observation_value": 20}),
        ("bmi under two weeks", {"measurement_method": "bmi", "observation_value": 13, "birth_date": "2028-06-07"}),
        ("ofc girl over 17", {"measurement_method": "ofc", "observation_value": 55, "birth_date": "2010-06-07"}),
        ("ofc boy over 18", {"measurement_method": "ofc", "observation_value": 57, "birth_date": "2009-06-07", "sex": "male"}),
        ("over 20 years", {"birth_date": "2008-06-11"}),
        ("length below 25 weeks", {"birth_date": "2028-06-12", "gestation_weeks": 23, "observation_value": 30}),
        ("observation true", {"observation_value": True}),
        ("observation zero", {"observation_value": 0}),
        ("gestation null", {"gestation_weeks": None}),
        ("gestation days null", {"gestation_days": None}),
    ]:
        cases.append(call(f"app error {name}", body={**BASE, **overrides}))

    # Request validation (Pydantic) errors.
    missing = dict(BASE)
    del missing["sex"]
    for name, body in [
        ("missing sex", missing),
        ("missing everything", {}),
        ("sex wrong case", {**BASE, "sex": "Male"}),
        ("method unknown", {**BASE, "measurement_method": "length"}),
        ("gestation 21", {**BASE, "gestation_weeks": 21}),
        ("gestation 45", {**BASE, "gestation_weeks": 45}),
        ("gestation days 7", {**BASE, "gestation_days": 7}),
        ("gestation days -1", {**BASE, "gestation_days": -1}),
        ("gestation string", {**BASE, "gestation_weeks": "40"}),
        ("gestation float whole", {**BASE, "gestation_weeks": 40.0}),
        ("gestation float fraction", {**BASE, "gestation_weeks": 40.5}),
        ("gestation text", {**BASE, "gestation_weeks": "forty"}),
        ("gestation bool", {**BASE, "gestation_weeks": True}),
        ("observation string number", {**BASE, "observation_value": "115"}),
        ("observation text", {**BASE, "observation_value": "abc"}),
        ("observation null", {**BASE, "observation_value": None}),
        ("observation list", {**BASE, "observation_value": [1]}),
        ("birth date with time", {**BASE, "birth_date": "2020-04-12T00:00:00"}),
        ("birth date wrong format", {**BASE, "birth_date": "12/04/2020"}),
        ("birth date not string", {**BASE, "birth_date": 20200412}),
        ("birth date null", {**BASE, "birth_date": None}),
        ("observation date with time", {**BASE, "observation_date": "2028-06-12T00:00:00"}),
        ("observation date bad month", {**BASE, "observation_date": "2028-13-12"}),
        ("observation date short", {**BASE, "observation_date": "2028-06"}),
        ("observation date text", {**BASE, "observation_date": "tomorrow"}),
        ("birth after observation", {**BASE, "birth_date": "2029-01-01"}),
        ("bone age type unknown", {**BASE, "bone_age_type": "xx"}),
        ("events text not list", {**BASE, "events_text": "abc"}),
        ("extra field", {**BASE, "colour": "blue"}),
        ("two errors", {**BASE, "sex": "x", "gestation_weeks": 50}),
        ("body is a list", [1]),
        ("body is a string", "hello"),
    ]:
        cases.append(call(f"validation {name}", body=body))
    for name, raw in [("invalid json", '{"a":'), ("empty body", ""), ("trailing comma", '{"sex": "male",}')]:
        cases.append(call(f"validation {name}", raw=raw))

    # Routing.
    cases.append(call("not found", path="/nope", body=BASE))
    cases.append(call("method not allowed", method="GET", body=None))
    cases.append(call("trailing slash", path=f"{prefix}/calculation/", body=BASE))

    # Bulk.
    observation = {"observation_date": "2028-06-12", "observation_value": 115}
    bulk = {"measurement_method": "height", "birth_date": "2020-04-12", "sex": "female"}
    cases.append(call("bulk one", path=f"{prefix}/bulk-calculation", body={**bulk, "observations": [observation]}))
    cases.append(
        call(
            "bulk mixed",
            path=f"{prefix}/bulk-calculation",
            body={
                **bulk,
                "observations": [
                    observation,
                    {"observation_date": "2028-12-12", "observation_value": 300},
                    {"observation_date": "2041-12-12", "observation_value": 160},
                    {"observation_date": "2029-06-12", "observation_value": 122.5, "events_text": ["x"]},
                ],
            },
        )
    )
    cases.append(call("bulk empty", path=f"{prefix}/bulk-calculation", body={**bulk, "observations": []}))
    cases.append(call("bulk 200", path=f"{prefix}/bulk-calculation", body={**bulk, "observations": [observation] * 200}, full=False))
    cases.append(call("bulk 201", path=f"{prefix}/bulk-calculation", body={**bulk, "observations": [observation] * 201}))
    cases.append(call("bulk missing observations", path=f"{prefix}/bulk-calculation", body=bulk))
    cases.append(call("bulk observation missing date", path=f"{prefix}/bulk-calculation", body={**bulk, "observations": [{"observation_value": 115}]}))
    cases.append(call("bulk birth after first observation", path=f"{prefix}/bulk-calculation", body={**bulk, "birth_date": "2028-07-01", "observations": [observation]}))

    # Authorisation.
    main.authorization_key = "test-key"
    cases.append(call("auth missing", body=BASE))
    cases.append(call("auth wrong", body=BASE, headers={"authorization": "Bearer nope"}))
    cases.append(call("auth right", body=BASE, headers={"authorization": "Bearer test-key"}))
    main.authorization_key = None

    cases.extend(extra(call, request, plausible_value))

    curated = cases
    cases = []

    # Every R-vector child as a single calculation request.
    for index, row in enumerate(json.loads((INDEPENDENT / "uk-who-cole-r-vectors.json").read_text())):
        birth = datetime.strptime(row["birth_date"], "%d/%m/%Y").date()
        observed = datetime.strptime(row["observation_date"], "%d/%m/%Y").date()
        body = request(birth, observed, row["measurement_method"], row["sex"], float(row["observation_value"]), int(row["gestation_weeks"]), int(row["gestation_days"]))
        cases.append(call(f"r-vector {index}", body=body, full=False))

    write(output, generator, f"FastAPI digital-growth-charts-server responses for {label} requests.", {"curated": curated, "r_vectors": cases})
