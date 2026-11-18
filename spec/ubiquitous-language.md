# Ubiquitous language

The Python engine, the HTTP API, and the React chart component currently use overlapping and sometimes contradictory names for the same concepts. Several of the collisions below are latent defect sources; one of them, the split between a chart's reference and a measurement's reference, is the subject of an open dGC hazard.

This file picks one vocabulary. Use these terms in Rust identifiers, CLI commands, JSON fields, tests, documentation, and commit messages.

---

## Growth reference

**Canonical:** *growth reference* - the published statistical reference against which a measurement is assessed, identified by one of these values:

```text
uk-who
who
cdc
trisomy-21
trisomy-21-aap
turners-syndrome
```

These strings are the public contract. They appear in API requests, in calculation provenance, and in the chart component. There are six today; a reference added under `standard.md` section 7.8 adds a value, and existing values do not change.

**Rust type:** `GrowthReference`.

**Do not use:** bare `Reference` as a Rust type name. In the chart component, `Reference` already means a wrapper around curve data, and reusing it would collide at exactly the point where the mismatch hazard lives.

### Beware: `turner` versus `turners-syndrome`

The HTTP route is `/turner` and the chart component prop value is `turner`, but the calculation engine value is `turners-syndrome`. Both spellings are load-bearing in different places, and code comparing one against the other without normalising will produce a false mismatch.

Rust holds `turners-syndrome` as the canonical value and normalises at each boundary that uses the other spelling. Every such normalisation is explicit and tested.

---

## Reference component

**Canonical:** *reference component* - one of the underlying datasets a growth reference is assembled from, selected by age. Examples: `uk90_preterm`, `uk_who_infant`, `uk_who_child`, `uk90_child`, `who_2006_infant`, `who_2006_child`, `who_2007_child`, `cdc_infant`, `cdc_child`, `fenton`, `trisomy_21_aap_infant`, `trisomy_21_aap_child`.

These use snake_case, unlike the hyphenated growth-reference values, and they are the keys in chart-coordinate responses.

**Do not use:** "reference" alone for a component. A growth reference is what a caller selects; a reference component is what the engine resolves it to at a given age. Provenance reports the growth reference, not the component, because the two can differ within a single result when chronological and corrected ages resolve to different components.

---

## LMS point

**Canonical:** *LMS point* - a single row of a reference table: an age together with its Lambda, Mu, and Sigma parameters, and optionally a `sigma` value used for the CDC extended-BMI tail.

**Rust type:** `LmsPoint { age, l, m, s, sigma }`.

**Beware:** the `sigma` field used by CDC extended BMI is **not** the `S` of LMS. They are different quantities that happen to share a name. Keep `s` and `sigma` distinct and never conflate them.

---

## SDS

**Canonical:** *SDS*, standard deviation score - how many standard deviations a measurement lies from the reference median, after LMS transformation.

**Aliases in the wild:** *z-score*, *z*. The Python engine uses `z_score` internally and `sds` in output. Rust uses `sds` throughout the public surface. `z` is acceptable only as a local variable inside the LMS transformation itself.

---

## Centile

**Canonical:** *centile* - the percentage of the reference population falling below the measurement, in the range 0 to 100. Derived from SDS through the normal cumulative distribution function.

**Do not use:** *percentile*. The clinical convention in this domain, and in all existing RCPCH surfaces, is *centile*.

**Rounding:** the API rounds centiles to four decimal places at exactly four fields. SDS is not rounded. Do not round in the engine.

---

## Centile band

**Canonical:** *centile band* - the human-readable description of where a measurement sits, such as "on or near the 50th centile" or "between the 25th and 50th centiles".

These strings are part of the API contract and must be reproduced exactly, including their existing quirks.

---

## Chart coordinate `l`

**Beware:** in chart-coordinate responses, each data point has fields `l`, `x`, and `y`. Here **`l` is the centile or SDS label of the line the point belongs to, not the Lambda of LMS.**

This is the single most dangerous name collision in the codebase: `l` means Lambda in a reference table and a line label in chart output. Rust names these distinctly in its own types and only emits `l` at the JSON serialisation boundary, with a comment stating which meaning applies.

---

## Chronological age and corrected age

**Canonical:**

- *chronological age* - decimal age from date of birth to observation date.
- *corrected age* - age adjusted for gestation at birth, measured from the estimated delivery date.

Both are computed and returned for every measurement, and both appear as separate plottable branches. Correction applies to post-term as well as preterm births, despite what the Python docstring says, and the rules for when correction stops differ between references.

**Do not use:** *adjusted age*, *gestationally corrected age*, or *post-menstrual age* in code or output. They are used loosely in clinical conversation but are not the API's terms.

---

## Estimated delivery date

**Canonical:** *estimated delivery date* - birth date plus the difference between term and gestation at birth, where term is fixed at 280 days.

**Abbreviation:** `edd` is acceptable in field names, matching the existing `estimated_date_delivery` field. Note that the existing field name has the words in an unusual order; preserve it in JSON for compatibility while using the natural order in Rust identifiers.

---

## Measurement

**Canonical:** *measurement* - one observation of one measurement method on one child at one date, together with everything the engine derives from it.

**Measurement method:** exactly one of `height`, `weight`, `ofc`, `bmi`. Note that `ofc` is rendered as "head circumference" in user-facing strings and as "Head circumference" at the start of an error message. Preserve both.

**Do not use:** *measurement type*, *parameter*, or *modality*.

---

## Provenance

**Canonical:** *provenance* - the record, carried in the calculation result, of which growth reference and which engine produced it. A nested object containing `growth_reference` and a `calculation_engine` block of `name`, `version`, and `commit`.

Provenance is produced by the engine and passed through by every surface. A surface that synthesises provenance from its own routing has defeated the purpose, because the value would then report what the caller asked for rather than what was computed.

**Do not use:** *reference* alone for this field. The engine-level key is `growth_reference`, and the earlier flat `reference` proposal is superseded.

Full contract in [provenance.md](provenance.md).

---

## Compatibility profile

**Canonical:** *Profile A* / *semantic*, *Profile B* / *wire-body*, *Profile C* / *wire-http*, as defined in `standard.md` section 4.

Always name the profile when claiming compatibility. "Byte-identical" without a profile is ambiguous and has already caused confusion between "the JSON parses to the same thing" and "the bytes are the same".

---

## Conformance case

**Canonical:** *conformance case* - one input plus its expected output, captured from the Python reference implementation and stored as data under `conformance/`.

**Do not use:** *test vector* for these. Reserve *literature vector* for cases traceable to a published source, such as the WHO gold fixture generated from WHO `anthro`, since those carry stronger evidential weight than a captured golden.
