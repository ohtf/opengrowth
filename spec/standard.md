# opengrowth - specification

Normative design for opengrowth, an Open Health Technology Foundation reimplementation in Rust of the growth calculation engine published by RCPCH in Python as [`rcpchgrowth`](https://github.com/rcpch/rcpchgrowth-python) and served over HTTP by [`digital-growth-charts-server`](https://github.com/rcpch/digital-growth-charts-server).

## 1. Purpose

One canonical Rust engine that computes SDS, centiles, centile bands, and chart coordinates against the UK-WHO, WHO, CDC, Trisomy 21, and Turner syndrome growth references, and drives several surfaces from that single implementation and a single binary: a library, a command-line tool, an MCP server, an HTTP API, and later a Python FFI binding.

The motivations are:

- **Consolidation.** The equivalent functionality is spread across six repositories: the Python package, the API server, a CLI wrapper, the React component library, a demo client, and the documentation site. A change touching the engine can need coordinated releases in five of them. The target is two: this repository (engine, CLI, MCP, API, Python bindings, docs) and one React repository (chart component and demo client). Fewer places for a calculation to drift, and a workload one maintainer can carry with help from other OHTF contributors.
- **Agent access.** A self-contained binary with a CLI and MCP server gives AI agents and scripts the calculations directly, without a Python environment or a network call.
- **Cost and throughput.** A compiled server should serve more requests per core at lower latency than the Python service. This is an expectation to be measured (`R29`), not a premise.
- **New references.** Growth references that `rcpchgrowth` does not include can be added as they are academically published, without changing the engine's methods (section 7.8). Few appear, and with agentic coding the code for one is quick to write; the licence and the evidence are the work. The achondroplasia references are the first candidates (`R47`).

This is a reimplementation, not a rewrite of intent. The clinical behaviour is already defined by the published methods, the independent R vectors, the Python engine, its reference data, and its test fixtures. Those are the specification of record for *what the numbers are*; this document specifies *how faithfully Rust must reproduce them* and *where it is permitted to differ*.

## 2. Non-goals

- **Not a new clinical method.** opengrowth devises no references and no statistical methods of its own, and does not reinterpret existing references. It can add references that others have published, implementing the method each publication specifies (section 7.8). Changes to clinical behaviour go through this project's clinical safety process (section 9), and are raised with the upstream Python project where they also apply there.
- **Not an RCPCH project.** opengrowth is not affiliated with or endorsed by RCPCH, does not use RCPCH branding, and is not part of the RCPCH Digital Growth Charts medical device. See section 3.1.
- **Not a `clincalc` module.** opengrowth is a separate OHTF project. `clincalc` or any other project may consume it as a library under the terms in section 3, which cannot relicense the reference data.
- **Not a replacement for `growth-references`.** That repository remains the provenance record for source datasets. This project consumes reference data; it does not become its home.
- **Not a re-derivation of reference data.** Tables are transcribed, not recomputed. Any transformation must be reproducible and evidenced.
- **Not initially a drop-in Python import.** FFI is a later phase (`R18`) and is explicitly out of scope for the first release.

## 3. Ownership, licensing, and data redistribution

### 3.1 Ownership and attribution

opengrowth is an Open Health Technology Foundation project from its first commit, hosted at `github.com/ohtf/opengrowth`. It was never an RCPCH project. It is an independent derivative work of `rcpchgrowth`, which RCPCH publishes under AGPL-3.0-or-later; the AGPL permits this, subject to the conditions below.

Copyright in new work is held by its contributors, recorded as `Open Health Technology Foundation contributors`. Files that port logic from `rcpchgrowth` also preserve the upstream copyright:

```rust
// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health
// SPDX-License-Identifier: AGPL-3.0-or-later
```

The upstream repository carries no explicit copyright statement beyond its licence, so the RCPCH line omits a year rather than guess one.

Original files that contain no ported logic carry only the OHTF line. When in doubt, include both: over-attribution costs nothing, under-attribution breaches the licence.

AGPL section 5 requires modified versions to carry prominent notices that they were modified, with a relevant date. A root `NOTICE.md` records that opengrowth is a Rust reimplementation derived from `rcpchgrowth`, names the upstream version and commit the port was taken from, and gives the date (`R25`).

RCPCH's name is used only nominatively: to state API compatibility and to give attribution. It does not appear in crate names, binary names, product names, or logos, and nothing may imply RCPCH endorsement.

Contributions are signed off under the [OHTF Developer Certificate of Origin](https://openhealthtechnology.org/community/dco/).

### 3.2 Code licence

**AGPL-3.0-or-later.** This is not a free choice. The Python engine is licensed AGPL-3.0-or-later, and a port is a derivative work, so the Rust engine inherits that licence. Network use of a modified hosted server triggers AGPL section 13: anyone running a modified opengrowth server must offer its source to users. The licence must not be applied by analogy to `growth-references`, which is a separate work with separate terms.

### 3.3 Reference data carries its own terms, tracked per table

The datasets do not share one set of terms. [`growth-references`](https://github.com/rcpch/growth-references) records that each dataset remains subject to its original source, terms, and licensing restrictions: WHO and CDC data are published as open data; the UK90 reference was produced under the Medical Research Council and, since the MRC's functions passed to UK Research and Innovation in 2018, is owned by UKRI. The Trisomy 21 and Turner tables have their own sources, whose terms are recorded in `R2`.

UK90 matters most because UK-WHO depends on it: UK-WHO uses UK90 for preterm infants and from 4 years to 20 years, with WHO 2006 in between. Without UK90 there is no UK-WHO.

A licence application to UKRI has been made and is pending (`Q1`). **Maintainer decision, 2026-11-18:** the UK90 tables are included in this repository and embedded in the binary for pre-alpha research and development, because they are already publicly distributed in the `rcpchgrowth` package. No release is for production use until UKRI has granted a licence. If UKRI refuses, the `embedded-data` feature is turned off for public artefacts and the tables are removed, and users with their own licence import them at install time (section 7.2); the design already supports this.

**Maintainer decision, 2026-11-18:** every reference table is embedded, including those whose terms are not yet recorded, unless its owner requires its removal. `R2` records each table's terms; until then, its file in `LICENSES/` says what is known.

Requirements:

- Reference data lives in `data/`, separate from the code, with its own `LicenseRef` terms in `LICENSES/` and `REUSE.toml`. The `embedded-data` feature compiles it in; without the feature, the binary contains the manifest but no values.
- Each table carries machine-readable provenance and licence metadata: source, publication, version or retrieval date, transformation applied, terms, and a content checksum.
- A registry test asserts every table, embedded or importable, has non-empty provenance, a licence identifier, and a checksum. A table without a terms entry fails the build. This mirrors the `clincalc` pattern where each calculator must record its own licence, and it satisfies the change-control rule `growth-references` imposes on itself.
- Every result names the tables it read and whether each was embedded or imported, so a change of data is visible in the output.
- No table is added, replaced, or regenerated without recording source, licence, transformation, and validation evidence in the same change.

### 3.4 Third-party material already identified as restricted

The Python repository retains material that is **not** cleanly redistributable and must not be copied into this project without review:

- `GrowthchartLMSmethod07.pdf` (Fenton and Sauve 2007) carries a Nature Publishing Group "all rights reserved" notice.
- `huiqi_cole_methods.bas` is LMSgrowth 2.74, copyright 2002-11 Medical Research Council UK (now UKRI).
- Fenton reference data is absent from the Python engine specifically because a licence had not been secured.

Algorithms may be reimplemented from published method descriptions; source files under restrictive terms must not be vendored.

## 4. Compatibility profiles

"Functionally identical" and "byte-identical" are different contracts with very different costs. This specification defines three named profiles so that a claim of compatibility is always testable and unambiguous. Every conformance test, issue, and release note states which profile it concerns.

### 4.1 Profile A - `semantic` (required)

The Rust implementation is semantically compatible when, for the same input:

- HTTP status codes match, where a server is involved.
- Parsed JSON structures match: same properties, same nesting, same array ordering.
- Properties that the Python implementation emits as explicit `null` are present and `null`, not omitted.
- Strings match exactly: error messages, centile bands, calendar-age phrases, clinical comments, date formats.
- Numeric values match within the tolerances in section 5.

Object key ordering, whitespace, and the lexical spelling of numbers are **not** part of this profile. This is the level the existing Python server test suite actually enforces, since its tests compare parsed JSON and its fixtures are inconsistently formatted.

Profile A is the release gate. Nothing ships without it.

### 4.2 Profile B - `wire-body` (target, separately costed)

Everything in Profile A, plus the serialised response body is byte-identical for a fixed corpus. This additionally requires reproducing:

- Pydantic response-model field ordering and Python dictionary insertion ordering.
- Compact JSON separators and absence of a trailing newline.
- Python float-to-string spelling, including `115.0` versus `115`, scientific-notation thresholds, and negative zero.
- Exact ordering and text of validation error arrays.

Profile B is the **goal for every operation** (maintainer decision, 2026-11-16: the API should be as byte-identical with the FastAPI server as it is possible to get). It remains a target rather than the release gate, because some differences may prove impossible or not worth their cost; each one that remains is enumerated in the migration notes with its reason, never waived in bulk. Byte identity lets a proxy, replay harness, or the benchmark's correctness check (`benchmark.md`) diff old and new responses directly, and lets integrators switch servers without noticing.

**One designed exception:** the `provenance` object can never be byte-identical, because opengrowth must report its own engine name, version, and commit, and adds the `reference_data` list (`provenance.md`). Profile B therefore means byte-identical *outside* `provenance.calculation_engine` and `provenance.reference_data`, and the replay harness masks exactly those paths and no others.

Measured so far (`conformance/README.md`):

- **UK-WHO `calculation` and `bulk-calculation` are byte-identical outside provenance** for 5,284 of 5,286 requests captured from the FastAPI server at a pinned commit: every R-vector child, an age and gestation grid across every UK-WHO join, calendar edge cases, pass-through fields, application errors, Pydantic validation errors, malformed JSON, routing, authorisation, and bulk limits. The other two are the enumerated Category 3 fixes in section 6.3.
- **UK-WHO `chart-coordinates` is byte-identical outside provenance** for all 60 captured requests: every named collection for each sex and measurement, custom centile and SDS lists, the error paths, and validation. Named-format curves are compared with the pinned server's responses after its stored chart files are regenerated with rcpchgrowth 4.6.5 (section 6.3).
- **UK-WHO `fictional-child-data` is byte-identical outside provenance** for 65 of 66 captured requests: every interval unit, preterm and post-term gestation, drift, noise, extreme SDS, every way a `null` field fails or does not, and validation. Noise is random in both servers, so the goldens fix Python's random draw and opengrowth is given the same one (`AppState::with_fixed_noise`); a separate test checks real noise stays within its range. The other request is the enumerated Category 3 fix in section 6.3.
- **The WHO operations are byte-identical outside provenance**: `calculation` and `bulk-calculation` for 5,791 of 5,793 captured requests (every UK-WHO case sent to WHO, every R-vector child's inputs, the WHO joins at 1856 days, 10 and 19 years, a sweep beyond ±3 SD for WHO's restricted LMS method, and gestations either side of WHO's 37-to-42-week term rule; the other two are the same enumerated `birth_date` fixes), `chart-coordinates` for 60 of 60, and `fictional-child-data` for 66 of 66.
- **The CDC operations are byte-identical outside provenance**: `calculation` and `bulk-calculation` for 5,731 of 5,734 captured requests (the UK-WHO cases, every R-vector child's inputs, CDC's joins at 2, 3, and 20 years, an SDS sweep across the extended BMI method above the 95th centile, and gestations before and around CDC's term rule; the other three are enumerated Category 3 fixes), `chart-coordinates` for 60 of 60, and `fictional-child-data` for 65 of 66 (the other enumerated).
- **The Turner, UK Trisomy 21, and AAP Trisomy 21 operations are byte-identical outside provenance**: calculation and bulk for 5,408 of 5,410, 5,528 of 5,530, and 5,536 of 5,538 captured requests (the shared cases, the R-vector inputs, and each reference's own joins and limits; the other two each are the `birth_date` fixes), charts for all 60 each (Trisomy 21's three enumerated), and fictional children for all 74, 66, and 66. For these three, upstream's stored chart files are current, and opengrowth matches the unmodified server's named-format responses too.
- **`utilities/mid-parental-height` is byte-identical outside provenance** for 117 of 118 captured requests, for all three of its references (the other is enumerated). For UK-WHO, all 66 requests match: parents across the range and at the ±8 SD limits, every combination and order of limit errors, a sample of 40 parent pairs, and validation. The centile is rounded twice with NumPy's rounding, as upstream rounds SciPy's float. WHO and CDC requests match too.
- **SDS and decimal ages are bit-identical** to Python for all 3,984 UK-WHO R-vector children on Linux.
- **Unrounded centiles are bit-identical** to Python for all 7,721 UK-WHO R-vector results on Linux, because the normal CDF and its inverse are SciPy's own `ndtr` and `ndtri`, translated (`R32`). Both agree with SciPy to the bit at all 16,299 points of the SciPy grid, deep tails included.
- Floats are written with CPython `repr` spelling (`115.0`, `1e-05`, `1e+16`, ties to even), matching CPython on 10,484 values (`R31`).
- Request validation reproduces Pydantic 2.11's lax coercions, error types, messages, contexts, and field order, and malformed JSON reports CPython's message and character position. Coverage is what the captured cases exercise; untested Pydantic corners (unusual date strings, very large integers) may differ in message but not in status.

### 4.3 Profile C - `wire-http` (explicitly not promised)

Byte-identical whole HTTP responses including headers, ordering, `Content-Length`, and framing. This is not achievable across a Uvicorn and a Rust HTTP stack and **will not be promised**. Where transport-level comparison is needed, compare status, an explicit allowlist of headers, and the body, excluding `Date`, `Server`, connection, and framing details.

### 4.4 Backwards compatibility is a separate, stronger promise

Independent of profile, the API contract must not regress. Existing fields keep their names, types, nullability, and meaning. New fields are additive. A client written against the Python server must continue to work against the Rust server without modification. This is the promise made to integrators and it outranks byte-level ambition: if a byte-compatibility shortcut would require changing a field, the field wins.

## 5. Numerical equivalence

### 5.1 The problem

Exact bit-level agreement with the Python engine cannot be assumed, because the Python engine's numerical behaviour is partly inherited from SciPy and NumPy:

- `scipy.stats.norm.cdf` and `norm.ppf` use specific approximations. A Rust normal distribution implementation will differ in the final bits, and the difference is largest in the extreme tails, which is exactly where clinically abnormal measurements live. opengrowth avoids this by translating SciPy's own `ndtr` and `ndtri` (`R32`), which agree with SciPy to the bit on Linux.
- `scipy.interpolate.interp1d` returns NumPy scalars, whose coercion behaviour is not identical to `f64`. In practice the difference is rounding: Python's `round()` on a NumPy `float64`, which is also what SciPy returns, uses NumPy's algorithm (scale, round half to even, scale back), and that differs from CPython's at scaled ties such as `round(42.34725, 4)`. opengrowth reproduces it wherever upstream's value is a NumPy float (`round::PyFloat`), which chart coordinates need.
- Python's `round()` is round-half-to-even over binary floating point. Rust's `f64::round()` is half-away-from-zero. These differ at exact ties, and the API rounds centiles to four decimal places, so a tie lands on a user-visible field.
- Floating-point operation order matters in the four-point Lagrange cubic interpolation. Compiler autovectorisation or fused multiply-add can change the result in the last bits.

### 5.2 Policy

1. **Implement Python-compatible rounding explicitly.** A shared helper implements round-half-to-even at a given decimal place. Never use `f64::round()` on a value that reaches an output field. This is a correctness requirement, not an optimisation detail.
2. **Preserve operation order in interpolation and LMS arithmetic.** Transcribe the Python expression structure rather than algebraically simplifying it. Do not enable fast-math-style transformations.
3. **Hold two oracles to two different tolerances.** They answer different questions and both must pass:
   - **Against the independent R vectors: 1e-3 absolute SDS.** About 4,000 fictional children generated by Tim Cole's R implementation (`sds_age_validation_2021_refactored_2026.json`, 3,984 UK-WHO cases) are the clinical-accuracy gate. They are independent of all RCPCH code, so they catch an error Python and Rust share, such as a defect inherited through the port. 1e-3 is the tolerance the Python engine is already accepted at, agreed with the statistician, because R's and Python's interpolation and normal-distribution implementations never agree exactly. opengrowth meets the same gate the Python engine does; it is not tightened or loosened.
   - **Against Python goldens: the tight per-field tolerances below.** These are the regression gate. They catch small drift, and string, shape, and rounding differences, that the R vectors do not exercise.
4. **Define tolerance per field class against the Python goldens, and enforce it in the conformance harness.** Proposed opening values, to be confirmed by measurement in `R5` and recorded in `Q4`:
   - SDS: agree to within `1e-9` absolute, since SDS is emitted unrounded by the API and is the field most exposed to normal-distribution differences.
   - Centiles: agree **exactly** after four-decimal rounding, except for an explicitly enumerated list of rounding-boundary cases, each individually reviewed and accepted under point 5. An unenumerated centile difference is a failure.
   - Chart coordinates: agree exactly at the four-decimal rounding already applied by the Python chart builder.
   - Ages, measurements, and derived values: agree to within `1e-9` absolute unless a specific field is documented as rounded.
5. **Rounding-boundary flips are enumerated, never waived.** If a centile differs after rounding because the unrounded value sits within a representable step of a `.00005` boundary, that case is recorded individually in a checked-in exceptions list with its inputs and both values, reviewed clinically, and either accepted with reasons or fixed. The list is expected to be short and to shrink; a growing list is evidence of algorithmic divergence rather than floating-point noise. A blanket "centiles may differ by 1 in the fourth decimal" allowance is not acceptable.
6. **Tolerance is a property of the harness, not of the engine.** The engine does not round to hide differences.

### 5.3 Normal distribution implementation

The choice of Rust normal CDF/PPF implementation is a numerical-accuracy decision, not a dependency-convenience decision, and it must be made against measured agreement with SciPy across the full range including tails beyond ±8 SD. `Q5` records this as an open decision with a required experiment: evaluate candidate implementations against a SciPy-generated grid before choosing.

## 6. Defect policy

The Python implementation contains behaviour that is observable but not intended. "Functionally identical" cannot mean "reproduce every defect", because several current defects are crashes. The port classifies each into one of three categories and records the decision.

### 6.1 Category 1 - reproduce exactly

Clinically observable output that a consumer may depend on, even if it looks odd. Includes ordinary field values, string wording, and rounding behaviour.

### 6.2 Category 2 - reproduce, but flag for separate clinical decision

Behaviour that is probably wrong but changes clinical output, so it cannot be silently corrected during a port. Each needs a clinical decision and, if changed, a version bump and a documented behaviour change. Identified so far:

- ~~`chronological_percentage_median_bmi` is computed using the corrected age.~~ Not so in 4.6.5, the pinned version: it uses the chronological age (`measurement.py:365-370`), and opengrowth does the same. Kept here, struck through, so the earlier analysis is not repeated.
- UK Trisomy 21 BMI data extend to 18.92 years (228 points per sex), but validation rejects ages above 18.82 (`rcpchgrowth/trisomy_21.py:54-55`), so the final two points per sex, at 18.83 and 18.92, are unreachable.
- `WHO_2006_OVER_TWOS_AGES` (37 entries) contains `3.083333333` twice, at index 13 and again at index 26 immediately after `4.0`, so the chart age grid is unsorted at that point (`constants/reference_constants.py:117`). Note this array is distinct from the clean `UK_WHO_2006_OVER_TWOS_AGES`, whose name contains it as a substring; a careless grep matches the wrong one.
- Several chart builders initialise `cole_method=False`, so a request for the Cole nine-centile collection is converted through the exact normal inverse rather than the two-thirds-SDS rounding used for UK-WHO. The same named collection therefore means different things per reference.
- WHO measurement-to-SDS forces linear interpolation while WHO SDS-to-measurement does not, so the two directions are not exact inverses at non-table ages.

### 6.3 Category 3 - do not reproduce

Crash-class and contract-violating behaviour. Reproducing these would be actively harmful, and no reasonable client depends on a 500. Each is fixed in Rust and recorded as an intentional, documented difference:

- Several routes `return HTTPException(...)` instead of `raise`, so a runtime failure becomes a response-validation `500` rather than the intended `422`. Rust returns the intended `422`.
- ~~Chart `Data.y` is declared non-nullable while 68 stored chart points contain `null`, so requests traversing those points fail with `500`.~~ Not so at the pinned server (29bb4e9): `y` is `Optional[float] = None`, so the points serialise as `null`, and opengrowth does the same. The 68 points are in CDC female weight and Trisomy 21 male BMI files, none in UK-WHO. Kept, struck through, so the earlier analysis is visible; see `Q6`.
- ~~`extended-who-centiles` is accepted by the chart request schema but no such asset is generated, producing a `500`.~~ At the pinned server it produces a 422, `Item not found: chart-data/extended-who-centiles-uk-who-{sex}-{method}.json`, which opengrowth reproduces byte for byte while `Q7` is open.
- The pinned server does not calculate named-format charts: it serves `chart-data/` files last regenerated on 2025-01-11. For UK-WHO, WHO, and CDC these no longer match its own engine: all 32 files of each differ from what rcpchgrowth 4.6.5 calculates, and points in the UK-WHO female Cole nine-centile height chart, for example, differ by up to 0.0183 cm. The Turner and Trisomy 21 files match 4.6.5. opengrowth calculates every chart with the 4.6.5 engine, so its curves agree with its own SDS calculations. The chart goldens therefore regenerate the files with 4.6.5 in a copy of the server (`conformance/generators/_http_charts.py`), and each chart test checks that the stored files still differ for UK-WHO, WHO, and CDC, and still match for the other three.
- `fictional-child-data` for UK-WHO with a start age before birth: upstream's `Measurement` gives the measurements dated before birth no corrected age, and the response model, which declares it non-nullable, rejects them, so the request fails with a 500. opengrowth returns the route's own 422, `Not possible to create UK-WHO fictional child data.` Enumerated in the UK-WHO fictional-child test.
- `fictional-child-data` for CDC BMI at an SDS whose centile rounds to 100 (above about 8.3 SD): the extended BMI method's inverse normal of 1 is infinite, upstream builds measurements from it, and the response then fails with a 500. opengrowth returns the route's own 422. Enumerated in the CDC fictional-child test.
- `mid-parental-height` for CDC with a negative parental height: CDC height L is not 1 at adult age, so upstream's SDS is a complex number, and comparing it with -8 raises a TypeError, which escapes as a 500. opengrowth reports the height as below -8 SD, as it does for zero. Enumerated in the mid-parental height test.
- `chart-coordinates` for UK Trisomy 21 with a centile of 0, 100, or more: upstream's Trisomy 21 builder rounds the infinite SDS without failing, so the response carries an infinity and fails to serialise with a 500. opengrowth returns the route's 422, as the other references do. Enumerated in the Trisomy 21 chart test.
- CORS (`Q8`): with an authorisation key set, upstream refuses every preflight with a 403, so no browser can call it from another origin. opengrowth answers preflights before authorisation, never sends `access-control-allow-credentials`, and answers preflights with `*` rather than by echoing the caller's origin. `tests/http_cors.rs` checks that upstream differs from opengrowth in exactly these ways (`pinned_from_resolved`).
- Turner chart requests with an invalid sex or measurement method return a bare string from an endpoint declared to return a chart object, producing a `500`. Rust returns `422`, consistent with the Turner calculation and fictional-child endpoints.
- `uk_who_reference()` returns a `ValueError` object rather than raising it on out-of-domain input, so the type error surfaces later at an unrelated place. Rust returns a typed error at the point of failure.
- `create_thrive_lines()` calls `plt.plot` while the matplotlib import is commented out, so it raises `NameError` on any call. Rust does not port a plotting call into a calculation library.

- LMS lookup at an age between the last row of one table and the first row of the next (for example between 1.9986 and 2.0 years in the WHO infant table) raises an `IndexError`, and an age below a table's first row raises a SciPy bounds `ValueError`. Neither is reachable from whole-day ages through UK-WHO selection. Rust returns `TableError::AgeOutOfRange`.
- A `Measurement` dated before birth has no corrected age, so upstream's observation check compares `None` with its SDS limit and records the Python `TypeError` text ("'<' not supported between instances of 'NoneType' and 'float'", or `'int'`, depending on the reference) as `observation_value_error`. opengrowth records no error. Engine only: the API rejects such a request before calculating. Enumerated in `tests/engine_cases.rs`.
- A zero or negative observation makes `z_score` raise or return a complex number. Rust returns `CalcError::InvalidObservation`, checked after the age rules so that the reported error matches upstream wherever upstream reports one. Through the API, upstream returns the "less than -8 SD" 422 when L is a whole number (UK90 height, L = 1) and a 500 otherwise; opengrowth always returns the 422. The CDC case, a height of zero where CDC's height L is negative, is enumerated in the CDC calculation test.
- A `birth_date` that is not a string (a number, `null`) raises a `TypeError` in the upstream `strptime` validator, which escapes as a bare `500 Internal Server Error`. opengrowth returns a 422 `value_error` carrying the same `TypeError` text. Enumerated in every reference's calculation test.
- CPython's JSON parser accepts `NaN`, `Infinity`, and lone surrogates; upstream then fails later or returns them. opengrowth rejects them as `json_invalid`. A numeric string `"nan"` is rejected as `float_parsing`, where upstream accepts it and then fails with a 500.
- `gestation_weeks` or `gestation_days` sent as `null` is accepted by the upstream model and then reaches Python arithmetic, producing a 422 whose message is a Python `TypeError` ("unsupported operand type(s) for *: 'NoneType' and 'int'"). opengrowth reproduces this exactly, because it is a 422 rather than a crash, but the message should be replaced with a clear one; that is a contract change for review (`Q13`).

Every Category 3 fix is a behaviour change relative to Python and must appear in the migration notes, even though each replaces a failure with a defined response.

### 6.4 Quality findings with no behavioural effect

The port takes the function and feature set, not the code. These upstream problems are left behind without any change to results, and are listed so that reviewers can see nothing clinical was altered:

- Debug `print()` calls on calculation paths, in `percentage_median_bmi` and in `Measurement`'s BMI block, which write to the server's stdout on ordinary requests (every BMI below two weeks of age prints the absence message).
- `measurement_for_z` prints `"measurement_for_z error: {e}"` literally, because the f-string prefix is missing.
- Commented-out alternatives (CubicSpline, splrep, the WHO beyond-3-SD measurement rule, height prediction fields) left in calculation modules.
- A sigma branch inside cubic interpolation that can never run, because cubic interpolation is skipped whenever a table has sigma.
- `corrected_decimal_age`'s docstring says babies of 37 to 42 weeks get a decimal age of 0.0; the code does not do that, and should not.
- `nearest_lowest_index` scans the whole table on every lookup and re-rounds every age; Rust binary-searches precomputed rounded ages, with a test proving equivalence to the upstream loop.
- Bare `except:` clauses that turn any error, including programming errors, into a generic message.
- Data: `uk90_preterm.json` contains BMI rows whose L, M, and S are all empty strings. Upstream never reads them because UK-WHO has no BMI data below 42 weeks. Rust treats an all-placeholder table as absent and rejects a partly empty one.
- Deployment: the server's Dockerfile runs `uvicorn --reload`, a development setting, while `s/start-prod-server` uses gunicorn workers. The benchmark uses the latter.

## 7. Architecture

**One crate**, `opengrowth`, with feature flags, following `clincalc`. A workspace of separate crates was the original plan; it was dropped on 2026-11-18 because nothing gained from it (enforced boundaries, separate publishing) needs it: features enforce the boundary, and a published crate cannot depend on unpublished siblings anyway. Split a part out only when someone else needs to reuse it on its own (house-style `library-extraction.md`).

```text
opengrowth/
|-- src/
|   |-- lib.rs, advice.rs, dates.rs, lms.rs, measurement.rs, normal.rs, ...
|   |                        the engine: a leaf, always built
|   |-- references/          one definition per growth reference (section 7.8)
|   |-- store/               table manifest, verification, embedding, import
|   |-- server/              HTTP API (feature `server`)
|   |-- cli/                 command-line tool (feature `cli`)
|   |-- main.rs, bin/og.rs   the `opengrowth` and `og` binaries
|-- data/                   reference tables (feature `embedded-data`)
|-- tests/                  conformance suites and server tests
|-- conformance/            cases, independent vectors, and goldens
|-- docs/                   user documentation site (R30)
|-- qms/                    quality management system
|-- spec/
`-- s/                      repeated-process scripts
```

| Feature | Adds | Default |
| --- | --- | --- |
| *(none)* | The engine and `store`: `serde`, `serde_json`, `jiff`, `sha2`, `thiserror` | always |
| `embedded-data` | The tables in `data/`, compiled in | yes |
| `server` | The HTTP API: `axum`, `tokio` | via `cli` |
| `cli` | The binaries: `clap`, `anyhow`, and `server` | yes |

Later surfaces are features of the same crate: `mcp` (`R26`), `python` (`R18`, built with maturin), and `wasm` (`R38`). CI builds and tests both the default set and `--no-default-features`, so the engine cannot quietly acquire a dependency on a surface.

The `opengrowth` binary exposes every surface as subcommands (`opengrowth serve`, later `opengrowth mcp`), so there is one artefact to build, version, sign, and deploy.

### 7.1 The engine

Every module outside `store`, `server`, and `cli`. Pure calculation: no clock, no I/O, no global state, no async runtime, no HTTP types. Timestamps and I/O are the caller's responsibility. This is what makes the engine embeddable in every surface and testable deterministically. Tables reach it through the `ReferenceData` trait, so tests can inject synthetic ones.

Errors are typed. `anyhow` belongs in the CLI and server, not in the engine, because callers need to match on failure modes such as out-of-domain age, unsupported measurement method for a reference, and unavailable reference data.

### 7.2 `store`: reference tables

A table reaches the engine from one of two places, behind the same `ReferenceData` interface:

- **Embedded.** The files in `data/`, compiled in by the `embedded-data` feature and verified against the manifest when first loaded. Every table is embedded unless its owner requires its removal (section 3.3), and CI checks that each file is byte-identical to `rcpchgrowth` 4.6.5's own copy on PyPI (`s/reference-tables`).
- **Imported.** `opengrowth data import <path>` verifies source files against the checked-in checksum manifest and copies them into the platform data directory. A verified imported file replaces the embedded table with the same identity, so a deployment can pin or update a table without a new binary. This is also the route for users who hold their own licence if a table can no longer be embedded.

Rules:

- An unverified, partial, or mismatched table is never used. A bad file fails loudly; it does not fall back or partially load.
- A calculation needing an absent table returns a typed `DataError::Unavailable` naming the table and how to import it. It never silently substitutes another reference, such as WHO for UK-WHO.
- Provenance records each table read, its checksum, and whether it was embedded or imported (see [provenance.md](provenance.md)).
- `opengrowth data status` lists every table, which copy is in use, and the state of any file in the data directory.
- The data directory location follows house-style deterministic configuration discovery and can be overridden by flag or environment variable, so containers and CI can mount it.

**JSON floats must be parsed exactly.** `serde_json` is built with `float_roundtrip`; without it, parsing can be off by one unit in the last place, which made 12% of ages and SDS differ from Python by 1 ulp until it was enabled. This applies equally to reference tables, goldens, and API requests.

Integrity checks run on every table, embedded or imported: the checksum, then the table invariants (finite values, positive M and S, strictly ascending ages, consistent sigma). These are the defence against a silent transcription error, which is the highest-consequence failure mode in this project.

### 7.3 `cli`

`clap` derive, thin `main.rs` delegating to a reusable library, `anyhow::Result`, global `--format text|json`, data on stdout and everything else on stderr, SIGPIPE reset on Unix. A bare invocation prints a useful command list rather than an error. Full conventions in house-style [`rust-cli.md`](https://github.com/pacharanero/house-style/blob/main/rust-cli.md).

The CLI covers the command surface of [`rcpchgrowth-python-cli`](https://github.com/rcpch/rcpchgrowth-python-cli), so that surface is an input to the design and its users are a migration audience. Unlike the HTTP API, the CLI requires sex and gestation explicitly rather than defaulting them, because agents are among its main users and a silently defaulted gestation changes the result.

### 7.4 `server`

Reimplements the current HTTP contract: six reference families, each with `calculation`, `bulk-calculation`, `chart-coordinates`, and `fictional-child-data`, plus `utilities/mid-parental-height`. Twenty-five POST operations in total.

Behaviour that must be preserved exactly:

- Bulk requests accept 1 to 200 observations. Structural failures reject the whole request with `422`; domain and calculation failures become inline error objects while the response stays `200`. Result ordering and count match the input.
- Route errors carry `loc`, `msg`, `type`, `input`, in that order. Inline bulk errors are re-validated upstream as `APIErrorDetail`, so they carry `type`, `loc`, `msg`, `input`, `ctx`, `url` (measured; see `conformance/expected/http-uk-who.json`).
- Optional response fields serialise as explicit `null`, never omitted.
- Only the four centile fields carrying the rounding validator are rounded, to four decimal places. SDS and ages are not rounded.
- Mid-parental height applies its own rounding sequence, including a double rounding of the centile.

Behaviour that must be reconsidered rather than copied:

- The current server generates chart assets during module import. A Rust server must not mutate checked-in files at startup. Chart assets become build-time artefacts.
- Authorisation is a bearer-token middleware configured by environment variable, exempting only the exact path `/`, and is not represented as an OpenAPI security scheme. Reproduce the behaviour; document it in the schema.
- `/` returns the server's OpenAPI document, which is also its health check. The document is RCPCH's: titled "RCPCH Digital Growth API", with RCPCH's production gateway as its server and the gateway's subscription key as its security scheme. RCPCH now also publishes it on GitHub Pages, so that no open gateway route has to serve it ([rcpch/digital-growth-charts-server#299](https://github.com/rcpch/digital-growth-charts-server/pull/299)). opengrowth's `/` is only the open health check, so its body differs from the pinned server's, and opengrowth's own OpenAPI document, generated from the code, is published on the documentation site (`R30`), not by the API (maintainer decision, 2026-11-24).
- CORS is configured with both a wildcard origin and credentials, which the CORS protocol does not allow together, and runs after authorisation, so preflights fail whenever a key is set. opengrowth allows any origin without credentials and answers preflights first; a deployment can restrict origins or turn CORS off (`Q8`).
- The `/growth/v1` prefix is not implemented in the server; it is applied by the gateway. Do not bake it into the Rust router.

### 7.5 Engine boundary for provenance

Every calculation result carries a record of the growth reference used and the engine that computed it. This is a clinical safety control mitigating the hazard first recorded as RCPCH dGC hazard #174, not a debugging field.

opengrowth **adopts the nested `provenance` shape** implemented on the Python `37-measurement-provenance` branch, with two required divergences: the engine reports its own identity in `calculation_engine.name` rather than impersonating the Python engine, and it adds the reference data version it used.

The engine produces provenance; every surface passes it through. A surface must **not** synthesise provenance from the route it happens to be serving, because that would make the field confirm the caller's assumption rather than test it.

Full contract, invariants, and version/commit semantics: [provenance.md](provenance.md). The remaining open element is the server-layer addition, tracked in `Q3`.

### 7.6 MCP (feature `mcp`, `R26`)

An MCP server exposing calculation, chart-coordinate, and mid-parental-height operations as tools, over stdio for local agents and streamable HTTP for hosted use. It calls the engine directly, not the HTTP server.

- Follows house-style [`mcp.md`](https://github.com/pacharanero/house-style/blob/main/mcp.md), including the conditional `rmcp` adoption gate and bounded input reading.
- Tool inputs use the same typed request structs as the CLI and server, generated from one schema source, so the three surfaces cannot drift.
- Like the CLI, tools require sex and gestation explicitly.
- Tool descriptions state the clinical-use boundary from [SAFETY.md](../SAFETY.md), and results carry full provenance so an agent can cite what it computed.
- The hosted MCP endpoint shares the server's authorisation model.

### 7.7 Documentation

User documentation lives in `docs/` and is published as a Zensical site per house-style [`docs.md`](https://github.com/pacharanero/house-style/blob/main/docs.md) (`R30`). CLI reference, OpenAPI reference, and MCP tool reference are generated from the code so they cannot drift. Clinical background on the references and methods may draw on the RCPCH documentation with attribution, subject to its licence.

### 7.8 Adding a growth reference

A published growth reference is added from its publication's own tables, by the steps below, without changing the engine's methods (section 1).

Reference tables have no standard format. Each publication tabulates its parameters at its own age spacing and in its own layout. Among the tables already in the store, WHO 2006 is daily; UK90 is weekly from 23 to 42 weeks, counted in weeks of gestation, and monthly in childhood; AAP Trisomy 21 has a monthly table to 3 years and a six-monthly one from 2 years; UK Trisomy 21 rounds its ages to two decimal places, so its steps are 29 or 33 days; and Turner is yearly. New sources arrive as PDF tables, spreadsheets, or CSV supplements, and some give a mean and SD rather than LMS parameters.

The engine assumes no spacing. A lookup finds the rows around an age by their tabulated ages, uses a row as it is when the age matches it to four decimal places, and otherwise interpolates over the actual ages: the four-point Lagrange cubic where there are two rows on each side, and linear interpolation at a table's edges or where the reference requires it (`src/table.rs`).

A reference can be added from:

- **LMS parameters** by age, used as published.
- **A mean and SD** by age, which is the LMS model with L = 1, M = the mean, and S = SD / M. The conversion is recorded as a transformation (section 3.3).

A publication using another model, such as GAMLSS with a distribution other than Box-Cox Cole and Green (which is LMS), or CLARITY's separate SDs below and above the mean (`R47`), is a larger change: that model's SDS is ported from its published description first, with its own evidence. A reference published only as centile curves would need a model fitted to it, which is a re-derivation (section 2), so it is not added unless its authors publish the parameters.

Adding one takes:

1. **A transcription script**, kept in the repository, that reads the publication's own file and writes the store's JSON layout, recording each transformation, such as months to decimal years. Where the only source is a PDF, a second person checks the extracted values against it, because a silent transcription error is the highest-consequence failure (section 7.2).
2. **Manifest entries** for its tables, with source, terms, and checksum (section 3.3). A new reference's terms are checked before it is added (`R47` adds only references that can be licensed); once added, its tables are embedded like every other.
3. **The reference's definition**: a module in `src/references/` whose `Definition` says how it selects a table for an age, where it has no data and the message it gives there, its SDS method and interpolation, its gestation-correction rule, the centile collection that describes a measurement, its chart segments and their age grids, and how the API serves it, such as its route prefix. The engine, charts, server, and command-line tool read the definition rather than branching on the reference. Besides the module, the reference needs its `growth_reference` value in `src/vocabulary.rs`, an entry in `references::ALL`, and its tables in `src/data.rs` and `src/store/` (`R46`).
4. **Evidence from the publication.** There is no upstream implementation to compare with, so Profile B does not apply. Every transcribed row must equal the source, checked by script, and calculated SDS and centiles must reproduce whatever the publication tabulates, such as centile values by age or worked examples, to its stated precision. Where its centile curves were fitted separately from the parameters it tabulates, the difference is measured and recorded. The provenance drift guard ([provenance.md](provenance.md), invariant 7, `R22`) must fail until the reference has its own provenance cases.
5. **Change control.** A new reference widens the population in [qms/intended-purpose.md](../qms/intended-purpose.md), so it needs a regulatory impact assessment first (`R44`). It is a clinical change, with a hazard log entry and clinical review, and an addition to the contract ([versioning.md](versioning.md) sections 2 and 3). The chart component, outside this repository (`R28`), has to draw it.

## 8. Conformance

Behaviour shared between the Python and Rust implementations is defined as data, not as duplicated test code.

```text
conformance/
|-- README.md      what each case set is, where it came from, who generates goldens
|-- cases/         input manifests
|-- fixtures/      inputs too large to inline
|-- independent/   vectors computed outside RCPCH code (R, WHO anthro), never regenerated by us
`-- expected/      goldens captured from the Python reference implementation
```

There are two kinds of oracle, with different authority:

- **Independent vectors** come from implementations that share no code with `rcpchgrowth`: Tim Cole's R implementation, and WHO `anthro`. They are the clinical-accuracy gate (section 5.2 point 3). They are imported as published and never regenerated by this project.
- **Python goldens** are captured from a pinned `rcpchgrowth` version, recorded in the manifest with the generator command, and never edited by hand. The Python implementation is the reference implementation for everything the independent vectors do not cover: response shape, strings, rounding, errors, and the references the R vectors do not test. A golden change without a stated rationale is a defect.

The Python goldens come at two levels. The HTTP goldens (`http-*.json`) are the Profile B gate: every request replayed, and the response compared byte for byte, mostly by digest. The engine conformance cases (`engine-*.json`) call `rcpchgrowth` directly at every boundary each reference's rules define, at the boundary and one day either side, and store every value in full, so any implementation can use them without a server: opengrowth's engine, and later its Python bindings and WebAssembly build. `conformance/README.md` describes their format.

Where an independent vector and a Python golden disagree beyond tolerance, that is a finding about the Python engine, not a licence to choose whichever Rust matches. Record it as a defect for classification under section 6.

Every suite runs in CI against the embedded tables, including UK-WHO and the HTTP replay (`Q12`, closed).

### 8.1 Existing fixtures to reuse directly

These already exist and are the backbone of the differential suite:

| Source | Cases |
|---|---:|
| `sds_age_validation_2021_refactored_2026.json` | 3,984 UK-WHO fictional children, independent R vectors from Tim Cole |
| `sds_age_validation_2021_deprecated.json` | 4,002 historical, including the 18 cases changed by the WHO migration |
| `cdc/bmi_height_weight_validation.json` | 2,730 CDC |
| `who_test_data/who_validation_data.csv` | 800 usable WHO |
| `who_test_data/who_under2_gold_192.csv` | 192 WHO gold, generated from WHO `anthro` |
| `who_chart_precalculated_centiles.py` | ~123,000 WHO chart points |

### 8.2 Coverage the existing fixtures do not provide

The Python suite has real gaps, and a port that only satisfies the existing fixtures would be under-tested. New cases are needed for:

- Turner, UK Trisomy 21, and AAP Trisomy 21 numerical output, which currently have only provenance smoke tests.
- The complete `Measurement` serialised shape, which has no golden test.
- Date and gestation boundaries: 22, 23, 25, 37, 40, 42, 44 weeks; birth, estimated delivery date, and delivery date plus 14 days; 2, 4, 5, the 1856-day WHO transition, 10, 17, 18, 19, 20 years; leap days and month ends.
- Validation limits: exactly ±8 SD and ±15 SD, which are accepted because the comparison is strict.
- Bulk cardinality at 0, 1, 2, 200, 201.
- Authorisation, CORS, 404, 405, and trailing-slash behaviour, none of which is currently tested.

### 8.3 Harness rules

Load the manifest, run the implementation, compare semantically against expected output within the declared tolerance. Do not make JSON key order part of the contract at Profile A. Name the failing case and the offending field; never assert only a pass count.

## 9. Clinical safety

This engine calculates anthropometric values used in children's clinical care, so it needs its own clinical safety process. It is not part of the RCPCH dGC platform and is not covered by that platform's MHRA registration or safety file. It is being developed as a Class I medical device with Open Health Technology CIC as manufacturer (`Q11`, [qms/](../qms/README.md)). Hazards are kept in this repository's own hazard log; hazards from the dGC log that also apply are copied in with a cross-reference.

Safety-relevant commitments:

- A root `SAFETY.md` states the clinical-use boundary, the regulatory position, and the hazard log, and grows into a full safety file set per house-style [`clinical-safety.md`](https://github.com/pacharanero/house-style/blob/main/clinical-safety.md) (`R21`).
- Reference-selection and age-correction logic is the highest-risk area. The Python repository's own analysis of the 18 fixture cases whose expected values changed during the WHO migration concluded that the largest numerical divergence between UK-WHO and WHO falls in preterm and early-infant assessment. That is an inference from a small sample rather than a systematic study, so treat it as a strong prior for where to look first, not as a bounded characterisation of the risk. Changes there require explicit clinical review, not just green tests.
- A change that alters any clinical output updates the safety documentation in the same change, or states why no update is needed.
- Reference-data changes carry provenance and validation evidence, per section 3.3.
- Tests protecting safety-relevant behaviour cite the hazard or clinical source they defend.

## 10. Acceptance gates

The first useful release is complete when:

1. Every independent R and WHO `anthro` vector passes at 1e-3 absolute SDS, the same gate the Python engine meets.
2. Profile A holds for every existing Python fixture, within the declared tolerances, with every rounding-boundary flip individually enumerated and reviewed.
3. UK-WHO preterm and early-infant cases pass, including the boundaries at 42 weeks, 2 years, and 4 years, and the 18 cases changed during the WHO migration. **This is the highest-risk gate and is not waivable.** Clinical review is required here in addition to green tests.
4. Every reference table, embedded or importable, has recorded provenance, licence, checksum, and integrity tests, and no public artefact contains a table whose terms do not permit redistribution.
5. The import, verification, status, and `DataError::Unavailable` paths are tested, including rejection of a tampered and a truncated file, so that removing an embedded table remains a working option.
6. The CLI covers the surface of the existing Python CLI.
7. The server passes an endpoint-level differential suite against the Python server for all 25 operations, including bulk partial-failure semantics.
8. Category 3 differences are documented in migration notes.
9. `NOTICE.md` and SPDX headers satisfy section 3.1.
10. `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` are green in CI, and every commit carries a DCO sign-off.

Explicitly **not** required for the first release: Profile B, the FFI binding, the MCP server, the experimental dynamic-growth functions, and bone-age or height-prediction calculations that do not currently exist in working form in Python.

The first release is still not for clinical use until `Q11` is resolved and the safety file set (`R21`) is complete.
