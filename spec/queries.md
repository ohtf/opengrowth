# Open queries

Decisions that need a human. Each carries a stable `Q<number>` code, states what it blocks, and records the current working assumption so work can proceed provisionally where that is safe.

Codes are never renumbered or reused. When a query is answered, record the decision and the date here, and update `standard.md` in the same change.

Each query names the item that **resolves** it and the items it **blocks**. These are always different: a query is never blocked by the item that answers it. Where the resolver is outside this repository, that is stated, because it means the decision has to be chased rather than scheduled.

---

## Q1 - May UK90-derived tables be redistributed?

**Resolved by:** UKRI's response to the pending licence application, recorded through `R2`. Not resolvable inside this repository.
**Blocks:** embedding UK90-derived tables in any public artefact (crates, release binaries, container images, Python wheels). Does **not** block implementation, because the runtime-supplied path in `standard.md` section 7.2 works under either answer.

**Background:** UK90 was produced under the Medical Research Council. Since 2018 the MRC's functions sit within UK Research and Innovation, which now owns the data. UK-WHO depends on UK90 for preterm infants and from 4 to 20 years, so without UK90 there is no UK-WHO. WHO and CDC data are open. The Trisomy 21 and Turner tables have their own terms, determined in `R2`.

A licence application has been made to UKRI. The data are more than 35 years old, and the maintainers may argue publicly that open access serves the public interest better than licensing. That argument is separate from this project's engineering, which must not depend on its outcome.

Possible outcomes:

- **Open licence or explicit redistribution permission.** Switch UK90 tables to embedded (a feature default), record the licence in table metadata, and close this query.
- **Licence permitting OHTF use but not onward redistribution.** OHTF may run a hosted server with embedded data; public binaries ship without it, and users import their own licensed copy.
- **Refusal or no answer.** Public artefacts ship without UK90 tables; users import their own licensed copy. A hosted OHTF server needs a separate legal basis before using UK90.

**Working assumption (superseded):** UK90-derived tables are runtime-supplied in every public artefact, and are not committed to this repository.

**Maintainer decision, 2026-11-18:** the UK90 tables are committed to this public repository (`data/`) and embedded in the binary by default, for pre-alpha research and development. The reasoning: the same files are already publicly distributed in the `rcpchgrowth` package and its public repository, and opengrowth is not in production use. This is onward redistribution of data whose owner, UKRI, has not yet granted a licence; the maintainer accepts that, and **no opengrowth release will be used for patient care, or declared conformant (`R43`), until UKRI grants a licence.** The files carry `LicenseRef-UK90` terms saying so, and the README says so.

Still open: UKRI's answer. If it is a refusal, or a licence that does not permit redistribution, the tables are removed from `data/` and from public artefacts, and the import path, which is tested, becomes the only route. The package is `publish = false` until this is settled.
---

## Q2 - Which Category 2 defects are corrected, and by whom?

**Resolved by:** `R4`.
**Blocks:** the affected parts of `R10` and `R12`.

The Category 2 list in `standard.md` section 6.2 contains behaviour that is probably wrong but changes clinical output, so it cannot be corrected during a port without clinical sign-off. (The `chronological_percentage_median_bmi` item turned out not to apply to the pinned 4.6.5; see `standard.md` section 6.2.)

Needed: a clinical decision per item, and an owner. If any is corrected, it is a behaviour change requiring a version bump, migration notes, and a hazard-log entry.

Decisions are made through this project's clinical safety process, with the Clinical Safety Officer as owner unless another is named.

**Working assumption:** reproduce all Category 2 behaviour exactly. Raise each as an issue against the upstream Python repository as well, as a courtesy and so the two engines can stay aligned if RCPCH makes the same correction, but do not wait on an upstream answer.

---

## Q3 - What shape does the server-layer provenance addition take?

**Resolved by:** the OHTF maintainers, during `R15`. Informed by `digital-growth-charts-server#207` upstream if RCPCH defines it first.
**Blocks:** the response shape in `R15`. Does **not** block `R12`.

**Settled:** the engine-level shape is decided. opengrowth adopts the nested `provenance` object from the Python `37-measurement-provenance` branch, containing `growth_reference` and a `calculation_engine` block of `name`, `version`, and `commit`, plus the additive `reference_data` block. See [provenance.md](provenance.md).

**Still open:** whether the server adds an `api_server` sibling identifying the deployment, and with which fields: presumably a name and version, possibly a commit and environment. Upstream `#207` proposes this but has not defined the fields.

**Working assumption:** the engine-level struct is stable and `R12` proceeds on it. The server-layer addition is modelled as an optional field behind the same single serialisation point, so adding it later touches one place. If RCPCH defines `api_server` first, match it, so clients see one structure from both servers.

**Update, 2026-11-17:** upstream has now defined it. `digital-growth-charts-server` 5.1.0 returns `api_server` with `name`, `version`, and `commit`, after `calculation_engine`. `opengrowth serve` matches that shape, reporting `opengrowth-server`, and puts the additive `reference_data` list last. Proposed for closure.
---

## Q4 - What are the numerical tolerances?

**Resolved by:** `R5`, which measures actual agreement before the values are fixed.
**Blocks:** the pass criteria for `R10` and acceptance gate 1.

`standard.md` section 5.2 proposes SDS within `1e-9` absolute, centiles exact after four-decimal rounding, and chart coordinates exact. These are opening positions, not measurements.

Needed: confirmation after `R5` measures actual agreement across the fixture corpus, including the tails beyond ±8 SD where normal-distribution differences are largest.

**Working assumption:** the proposed values. If measurement shows they cannot be met, the correct response is to investigate the cause rather than widen the tolerance, because a wide tolerance would mask genuine algorithmic divergence.

**Measured, 2026-11-17** (UK-WHO only; `conformance/README.md`):

- **SDS: exceeded.** All 7,721 UK-WHO SDS on the R-vector corpus are bit-identical to Python on Linux, so the 1e-9 tolerance holds with zero margin used. Keep 1e-9 as the gate so that a platform with a different `pow` still passes, and report the bit-exact rate alongside it.
- **Ages: bit-identical** (7,968 of 7,968).
- **Centiles: bit-identical** (2026-11-18), unrounded, for 7,721 of 7,721, since the normal CDF became SciPy's own (`Q5`, `R32`).
- **Chart coordinates: byte-identical** through the API for UK-WHO, 60 of 60 captured requests (2026-11-18). This needed NumPy's rounding wherever upstream rounds a NumPy float (`round::PyFloat`, `standard.md` section 5.1).

Still open for the other references.

---

## Q5 - Which normal distribution implementation?

**Resolved by:** `R5`, via the experiment described below.
**Blocks:** `R10` and every numerical acceptance gate.

The choice determines how closely centiles and SDS can track SciPy, particularly in the tails. This is an accuracy decision, not a convenience decision, and it should be settled by measurement rather than by reputation or download count.

Needed: an experiment evaluating candidate Rust implementations against a SciPy-generated grid spanning the ordinary range and the extreme tails, reporting maximum absolute and relative deviation, plus a maintenance and licence review of each candidate.

**Working assumption:** none. Do not pick a crate before the experiment.

**Experiment, 2026-11-17** (`tests/normal_vs_scipy.rs`, SciPy 1.18.1):

| Candidate | Max relative difference from SciPy | Bit-exact |
| --- | --- | --- |
| CDF A: Cephes `ndtr` structure on `libm` `erf`/`erfc` | 5.7e-14 for \|z\| < 37; 2.5e-15 within ±8 SD | 54% (59% within ±8 SD) |
| CDF B: `0.5 * erfc(-z / sqrt 2)` on `libm` | 5.7e-14 | 54% |
| PPF: AS241 `PPND16`, implemented in the engine | 8.4e-16 | 31% |

`libm` is the rust-lang-maintained port of musl's maths library (MIT/Apache-2.0); AS241 is a published algorithm with no licence encumbrance.

**Working decision:** CDF A and AS241, now in the engine, with the measured bounds asserted by the test. This is not yet closed, because byte-identical API output (Profile B) would favour bit-exact agreement. The next step is a candidate porting SciPy's own `ndtr` and `ndtri` (Cephes, BSD-3-Clause via SciPy's `xsf`), measured the same way. Adopt it only if it is bit-exact or close to it, and only after the licence review.

**Decision, 2026-11-18 (`R32`):** SciPy's own `ndtr` and `ndtri`, translated line for line from `xsf` at the commit SciPy 1.18.1 builds (`src/normal/cephes.rs`), replace CDF A and AS241.

| Candidate | Max relative difference from SciPy | Bit-exact |
| --- | --- | --- |
| CDF: SciPy `ndtr`, translated | 0 | 100%: 11,969 of 11,969, \|z\| up to 40 |
| PPF: SciPy `ndtri`, translated | 0 | 100%: 4,330 of 4,330 |

Licence review: the code is the Cephes Math Library by Stephen L. Moshier, which `xsf` distributes under BSD-3-Clause with the author's permission (`xsf` `LICENSES_bundled.txt`), in SciPy's 2024 C++ translation, also BSD-3-Clause. BSD-3-Clause is compatible with AGPL-3.0-or-later. The file keeps its licence, and binary distributions must reproduce the notice (`NOTICE.md`).

With this, unrounded centiles are bit-identical to Python (`Q4`). Bit-identity depends on the platform `exp` and `log` matching SciPy's, as SDS already depends on `pow`: true on Linux with glibc, unmeasured elsewhere (`R34`). The `libm` crate is no longer an engine dependency. **Closed.**

---

## Q6 - How should chart null coordinates be handled?

**Resolved by:** `R4`, as a Category 3 defect decision.
**Blocks:** `R13`, and the chart response schema in `R15`.

The current server declares chart `y` as a required non-nullable float, but 68 stored chart points contain `null`, concentrated in the eighty-five-percent-centile Trisomy 21 male BMI and CDC female weight files. Requests traversing those points fail response validation with a `500`.

Options: permit `null` in the schema and emit those points, which changes the response shape for consumers; or omit the points, which changes array lengths and could alter a rendered curve; or investigate why they are null and fix the underlying data.

Needed: a decision, ideally informed by why those points are null in the first place, since that may be a data-generation defect rather than a schema defect.

**Working assumption:** investigate the cause first. A data fix is preferable to either schema change because it removes the problem instead of encoding it.

**Overtaken, 2026-11-18:** at the pinned server (29bb4e9) upstream has already made `y` nullable (`Optional[float] = None`), so there is no `500` left to fix: null points serialise as `null`, and opengrowth emits them identically, as Profile B requires. The cause is known and is not a data defect: `measurement_for_z` returns `None` where `1 + L S z` is negative, so the LMS transform has no real answer at that extreme SDS. Upstream's schema comment says the same, and renderers leave a gap. Nothing remains to decide unless a consumer cannot handle `null`.

---

## Q7 - Should `extended-who-centiles` work?

**Resolved by:** `R4`, as a Category 3 defect decision.
**Blocks:** `R13`.

The chart request schema accepts `extended-who-centiles`, but no such asset is generated and the constant is not handled by the centile-band logic, so a request produces a `500`. Either the format should be implemented, or it should be removed from the accepted values and rejected with a `422`.

**Working assumption:** implement it, since it is already advertised in the public schema and some integrator may have written it into a client. Confirm whether any client actually uses it before deciding.

**Measured, 2026-11-18:** at the pinned server the request is a 422, not a 500: the server reads a stored file that was never generated, `Item not found: chart-data/extended-who-centiles-uk-who-{sex}-{method}.json`. opengrowth returns the same 422, byte for byte, until this is decided.

---

## Q8 - What is the intended CORS policy?

**Resolved by:** the OHTF maintainers, before `R15`.
**Blocks:** `R15`.

The current server sets a wildcard origin together with `allow_credentials=True`, which is contradictory, and additionally lists `http://localhost:8000`, which is redundant under the wildcard. Because the authorisation middleware sits outside CORS, preflight requests to non-root paths can be rejected with `403` before CORS processing.

Needed: a statement of the intended policy. Reproducing a contradictory configuration in a new implementation is not obviously right, but changing it could break a browser client.

**Working assumption:** reproduce the current observable behaviour where a browser client could depend on it, resolve the contradiction rather than copy it (check what the current server actually sends; Starlette may echo the request origin when credentials are allowed, which is more permissive than intended), make the policy configurable per deployment, and record the difference in the migration notes.

**Measured, 2026-11-18** (`conformance/expected/http-cors.json`, from the pinned server through Starlette's TestClient):

- Without a key, preflights succeed, echo the caller's origin, and send `access-control-allow-credentials: true`. The CORS protocol does not allow a credentialed response to say `*`, so Starlette echoes whatever origin asks, which in effect allows credentialed requests from every website. Other responses say `access-control-allow-origin: *`, together with `allow-credentials: true`, which browsers ignore unless the request carried credentials; a request with a cookie gets its origin echoed.
- With a key, the authorisation middleware runs first and refuses every preflight to a path other than `/` with a 403 and no CORS headers. Browsers never send credentials on a preflight, so no browser can call a key-protected server from another origin.
- Requests without an `Origin` header, which is every request from a non-browser client, get no CORS headers in either case.
- The demo client (`rcpch/digital-growth-charts-react-client` at `c0756f2`) calls `fetch` without the `credentials` option, so it sends no cookies cross-origin. It sends `Content-Type: application/json` and, if configured, `Subscription-Key`, so every call is preflighted. It needs the preflight to succeed and allow those headers, and `access-control-allow-origin` on the response; it does not need credentials.
- opengrowth sent no CORS headers and answered preflights with 405, so the demo client could not call it from another origin at all.

**Decision, 2026-11-18:** the demo client and every API client must keep working (maintainer). opengrowth sends what the pinned server's own Starlette middleware sends when configured with `allow_credentials=False` and placed in front of authorisation: any origin, without credentials, with preflights answered before authorisation. `tests/http_cors.rs` checks this byte for byte against that configuration, and checks that the pinned server differs from it only in three ways: the credentials header; preflights that echo the origin; and, with a key, the 403 on preflights. A deployment can restrict origins (`--cors-allow-origin`, `$OPENGROWTH_CORS_ALLOW_ORIGINS`) or turn CORS off for a gateway that adds its own (`--no-cors`). The redundant `http://localhost:8000` entry has no equivalent. The only client this could stop is a browser application that sends cookies cross-origin; the API does not use cookies, and no such client is known.

---

## Q9 - How is equivalence with the Python server demonstrated before anyone relies on opengrowth?

**Resolved by:** the OHTF maintainers and the Clinical Safety Officer, as part of `R21`.
**Blocks:** `R20`, `R21`, and the ambition level for Profile B.

opengrowth is not replacing RCPCH's server, so there is no cutover of RCPCH traffic to plan. The question is what evidence is enough before an integrator points production traffic at an opengrowth server instead of the RCPCH API.

Options:

- **Self-hosted differential replay.** Run the pinned Python server locally from its AGPL source and replay the conformance corpus plus large volumes of generated inputs against both. Needs no access to RCPCH's live service.
- **Shadow comparison against the live RCPCH API.** More realistic, but sends traffic to a service OHTF does not control, which needs RCPCH's permission and must use no real patient data.
- **Integrator shadow mode.** An early-adopter integrator calls both and logs disagreements. Strongest evidence, but needs a partner and careful data handling.

**Working assumption:** self-hosted differential replay as the required gate, because it needs nobody's permission. Integrator shadow mode is welcome as extra evidence. Profile B is pursued only if replay comparison needs byte-level diffs.
---

## Q10 - How does opengrowth track the upstream Python engine?

**Resolved by:** the OHTF maintainers. RCPCH's own plans for `rcpchgrowth` are outside this project's control.
**Blocks:** `R3` (which version to pin), `R18`.

RCPCH may continue to change `rcpchgrowth`, freeze it, or deprecate it. opengrowth needs a policy that works in all three cases, because the Python engine is the reference implementation for everything the independent R vectors do not cover.

Needed: which upstream version the goldens are pinned to; how upstream changes are reviewed (adopt, decline, or record as divergence); and what becomes the reference if upstream stops being maintained.

**Working assumption:** pin goldens to the latest released `rcpchgrowth` version at the time of `R3`. Review each later upstream release for clinical changes and adopt them through the normal clinical safety process, recording any declined change as an intentional divergence. If upstream stops, the pinned version plus the independent vectors remain the reference, and opengrowth becomes the maintained implementation. The Python bindings (`R18`) are published as `opengrowth` on PyPI, not under the `rcpchgrowth` name.

---

## Q11 - What is opengrowth's regulatory position?

**Resolved by:** the Clinical Safety Officer and OHTF, with regulatory advice where needed. Not resolvable by engineering.
**Blocks:** any clinical use, any hosted OHTF service used for patient care, and `R21`.

The engine calculates anthropometric values used in clinical care, so a product built on it for patient care is likely to be software as a medical device under UK MDR. opengrowth is not covered by the RCPCH dGC registration.

Needed: whether OHTF places anything on the market (for example a hosted API), and if so as manufacturer of what; or whether OHTF publishes the engine as open-source software only, with each deploying organisation taking the manufacturer role for its own product.

**Working assumption:** OHTF publishes the engine as open-source software that is not for clinical use, and does not operate a clinical service. Deployers carry the manufacturer and DCB0160 responsibilities for their own products. The safety evidence in this repository is written to make their job easy.

**Decided, 2026-11-17:** opengrowth will be a Class I medical device in Great Britain, with **Open Health Technology CIC** as legal manufacturer and **Dr Marcus Baw** as PRRC. It is an anthropometric calculation tool that does not diagnose. The QMS, intended purpose, and classification rationale are in `qms/`. The working assumption above is superseded. The EU and Northern Ireland are deferred until the EU MDR amendment COM(2025) 1023 is adopted ([qms/regulatory-strategy.md](../qms/regulatory-strategy.md)). Closed.

---

## Q12 - How does CI run UK90-dependent tests without publishing the data?

**Resolved by:** the OHTF maintainers, during `R6`.
**Blocks:** UK-WHO conformance in CI (`R9` onward for UK-WHO). Local development is unaffected.

The repository is public, so UK90 tables cannot be committed, cached in public CI artefacts, or baked into images while `Q1` is open. CI still needs them to run the UK-WHO suite, which is the highest-risk area.

Options: fetch the tables at job start from private storage using a repository secret, never caching or uploading them; run UK-WHO conformance only on a self-hosted runner that holds a licensed copy; or skip UK-WHO conformance on forks and external PRs and require it on `main`.

**Working assumption:** fetch from private storage with a secret, verified against the checksum manifest, on `main` and maintainer PRs only. External PRs run everything except UK-WHO conformance, and a maintainer runs it before merging. Revisit if `Q1` resolves in favour of open data.

**Closed, 2026-11-18:** with the tables in the repository (`Q1` decision), CI runs every suite, including UK-WHO conformance and the HTTP replay, on every push.

---

## Q13 - Should upstream's leaked Python error messages be kept?

**Resolved by:** the OHTF maintainers and the Clinical Safety Officer.
**Blocks:** nothing; the current behaviour is safe. Affects `R20` only as an enumerated difference if changed.

When `gestation_weeks` or `gestation_days` is sent as `null`, upstream returns a 422 whose message is a raw Python `TypeError`, such as "unsupported operand type(s) for *: 'NoneType' and 'int'". opengrowth reproduces it byte for byte. It is a correct refusal with a useless message.

Options: keep it for byte identity; or replace it with a clear message ("gestation_weeks must be a number from 22 to 44, not null") and list it as a known difference. Changing the message is a contract change, though no client can sensibly depend on this text.

**Working assumption:** keep it until decided, because the goal is byte identity and the behaviour is already a safe refusal.

---

## Q14 - Does the inherited centile-band wording read as interpretation?

**Resolved by:** the PRRC, with regulatory advice (`R41`).
**Blocks:** nothing now; affects the Class I argument and, if changed, `R20`.

The upstream centile-band sentences say a measurement is "on or near the 50th centile" or "between the 2nd and 9th centiles", which describe position, but also "below the normal range", "above the normal range", and "well outside the normal range. Please check its accuracy." The first two describe position relative to the outermost centile lines; the last is an input plausibility prompt. "Normal range" could be read as a clinical interpretation, which the intended purpose disclaims.

Options: keep them, byte-identical to the upstream Class I device, with the argument recorded; or replace "normal range" with positional wording, such as "below the 0.4th centile line", as an enumerated difference from upstream and a clinical change.

**Working assumption:** keep them, since they are the wording of a registered Class I device, and record the argument in the classification rationale.

---

## Q15 - How should the CLARITY achondroplasia tables be used?

**Resolved by:** the authors of Hoover-Fong JE et al. 2021 (CLARITY), asked by the maintainer, then the PRRC.
**Blocks:** the CLARITY reference in `R47`.

CLARITY's Table S1 (`data/sources/clarity-2021/`) gives, for each sex and month of age, a mean, an SD, and separate SDs below and above the mean. The paper does not say enough to calculate a Z-score at every age the way its authors intend:

1. **Which SD?** The methods say weight Z-scores use the SD on the measurement's side of the mean. Do height and head circumference Z-scores also use the lower and upper SDs, or the single SD?
2. **Which age is each row?** The values were estimated at each month of age, but the file labels each row as the interval `[k, k+1)`. Is row k the value at exactly k months, or for the whole month? For boys, row 0 gives a mean length of 52.8 cm and weight of 4.7 kg, above the WHO medians for average-stature boys at birth (49.9 cm, 3.3 kg). Is row 0 the value at birth?
3. **Between months?** For an age between two rows, should the values be interpolated, and how, or should the row for the completed month be used?
4. **Beyond 18 years?** The paper covers birth to 18 years, but the file runs to 239 months, with few or no measurements after about 15 years. Are the rows after 216 months meant for use?

**Decided, 2026-11-19 (maintainer):** the reference is `achondroplasia-clarity`, as `trisomy-21-aap` names its source; ages are chronological, with no correction for gestation, since the cohort was born at about term; and `R46` comes first, so that CLARITY is the first reference added as a self-contained definition.

**Working assumption** until the authors reply: none. The CLARITY reference is not built until these are answered.
