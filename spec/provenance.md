# Calculation provenance

Every calculation result carries a record of which growth reference produced it and which engine computed it. This is a **clinical safety control**, not a debugging convenience: it exists so a downstream consumer can detect that a measurement was calculated against a different reference from the chart it is being plotted on.

Safety driver: [dGC hazard #174](https://github.com/rcpch/digital-growth-charts-documentation/issues/174), where chart curves and a measurement result used different growth references and the discrepancy was invisible to the user. Provenance is step 1 of the four-step mitigation chain designed upstream, alongside `rcpchgrowth-python#37`, `digital-growth-charts-server#207`, and `digital-growth-charts-react-component-library#217`. opengrowth records the hazard in its own hazard log with a cross-reference; it does not rely on RCPCH's mitigation being in place.

## 1. Adopted shape

opengrowth adopts the shape the upstream server returns (engine `provenance` plus the server's `api_server` block, `digital-growth-charts-server` 5.1.0), plus an additive `reference_data` list. It is the first key of the result object, and `opengrowth serve` emits exactly this:

```json
{
  "provenance": {
    "growth_reference": "uk-who",
    "calculation_engine": {
      "name": "opengrowth",
      "version": "0.1.0",
      "commit": "unknown"
    },
    "api_server": {
      "name": "opengrowth-server",
      "version": "0.1.0",
      "commit": "unknown"
    },
    "reference_data": [
      {
        "table": "uk90-preterm",
        "file": "uk90_preterm.json",
        "sha256": "fcd126301d8fb961e3006b828688022ce950f085b0ec1d877a40c8f6df21f3ef",
        "source": "embedded"
      }
    ]
  },
  "birth_data": { "...": "..." }
}
```

Reference: `rcpchgrowth/measurement.py:202-210` on that branch.

`reference_data` is an opengrowth addition and is purely additive, so clients written against the Python shape are unaffected. It exists because tables are supplied by the user at install time (`Q1`), which makes "which data did this calculation use?" a real question rather than one answered by the engine version. It lists each table the calculation actually read, once, with the SHA-256 of the verified file; a measurement whose chronological and corrected ages fall in different UK-WHO segments lists both. The `opengrowth calculate --format json` output already follows this shape. `source` is `embedded` for a table compiled into the binary and `imported` for a verified file from the data directory, which takes precedence.

## 2. Invariants

These are normative and are carried over from the Python test suite at `rcpchgrowth/tests/test_provenance.py`.

1. **Always present.** Every successful calculation carries provenance.
2. **Present even when the numerical result is not.** A measurement that fails to produce SDS and centile values, for example because the observation date precedes the birth date, still returns a result object, and that object still carries provenance. Provenance must not sit on a code path that early-returns skip.
3. **Reports what was computed, never what was requested.** The value comes from the reference the engine actually used. A surface that derives it from its own routing has defeated the control, because the field would then confirm the caller's assumption rather than test it.
4. **`growth_reference` is exactly one of the canonical values** in [ubiquitous-language.md](ubiquitous-language.md), six today, with canonical spelling preserved. In particular `turners-syndrome`, not `turner`.
5. **`trisomy-21` and `trisomy-21-aap` remain distinguishable.** They are different references with different data, and collapsing them to a shared "Down syndrome" label would defeat the control for exactly the population most likely to be plotted on a syndrome-specific chart.
6. **Engine fields are non-empty strings.** Never null, never absent, never `""`.
7. **A drift guard test asserts the provenance test cases cover all six references.** If a seventh reference is ever added, the test fails rather than silently skipping it. This pattern is worth preserving; it is the reason the Python suite cannot quietly lose coverage.
8. **`reference_data` describes the tables actually read.** Never the tables configured, installed, or expected. If a calculation reads no table, for example because the age has no reference data, `reference_data` is an empty list rather than guessed.

## 3. Engine identity must differ from Python

**The engine reports `name: "opengrowth"`, not the Python engine's `"rcpch/rcpchgrowth-python"`** (the value `rcpchgrowth` 4.6.5 actually emits, at `measurement.py:201`).

This is a deliberate divergence from strict byte-compatibility, and it is required rather than merely permitted. The purpose of the field is to identify the engine that produced the result. While both engines are in use, during equivalence testing (`Q9`) or when an integrator moves between the RCPCH API and an opengrowth server, an engine that reports its predecessor's name makes it impossible to attribute a divergent result to either implementation. Provenance would fail at precisely the moment it is most needed.

Consumers are unaffected in the one place it matters: the chart component's mismatch guard keys on `growth_reference`, not on engine name. Nothing in the mitigation chain matches on `calculation_engine.name`.

This divergence is recorded in the migration notes as an intentional Category 3 difference.

## 4. Version and commit semantics

| Field | Python | Rust |
|---|---|---|
| `version` | `importlib.metadata.version("rcpchgrowth")` at runtime, falling back to `"unknown"` when the package is not installed, as in an editable or source checkout | `CARGO_PKG_VERSION` at compile time |
| `commit` | `"unknown"` in source; the publish workflow overwrites `rcpchgrowth/_build_info.py` with the release SHA immediately before building | Stamped at compile time from `OPENGROWTH_COMMIT`, which must be a full 40-character commit hash or the build fails, as upstream's schema refuses anything else; `"unknown"` in unstamped builds |

Two consequences worth stating explicitly:

- **Rust's `version` can never be `"unknown"`.** It is a compile-time constant. A consumer that special-cases an unknown version will simply never see that case from Rust. This is an improvement rather than a regression, but it is a behavioural difference.
- **`commit` must remain `"unknown"` in unstamped development builds.** Do not substitute a dirty working-tree hash or a build timestamp. The value's meaning is "this is a released artefact and here is exactly what it was built from"; a value that looks like a commit but identifies no reachable commit is worse than an honest `"unknown"`.

Rust's compile-time stamp is cleaner than overwriting a checked-in source file, but it must preserve the semantics above rather than merely the mechanism.

## 5. Server layer

Upstream, `digital-growth-charts-server#207` proposes extending provenance at the HTTP layer with an `api_server` sibling to `calculation_engine`, with fields not yet defined. opengrowth decides its own server-layer shape (`Q3`), and matches the upstream shape if RCPCH settles one first, so that clients handling both servers see the same structure.

Two constraints apply regardless of the final shape:

- The server adds to provenance; it never rewrites `growth_reference` or `calculation_engine`. Those belong to the engine.
- The server must declare the provenance object in its response schema. The current FastAPI server filters unknown fields through its Pydantic response model, so a field the engine emits but the schema omits is silently dropped. A Rust server must not reintroduce that failure mode: if the engine emits provenance and the schema does not declare it, that is a bug, and a conformance test should catch it.

## 6. Modelling in Rust

Provenance is a typed struct with a single serialisation point, so the shape can change in one place when `Q3` closes. Do not construct provenance JSON inline at each surface.

In opengrowth, the engine's `Provenance` (`src/provenance.rs`) travels in each result it returns, `Measurement` and `Chart`, set from the reference definition the engine used, so the server's chart-coordinates and mid-parental height responses carry provenance, which the pinned server's do not; `SdsResult` carries the same reference and table for an SDS on its own. `Record` is the one serialised form. A surface writes the engine's provenance with the file behind each table, which only the store knows, and the server adds its `api_server` identity. A `Record` can list only the tables in the engine's provenance.

Conformance tests (`tests/provenance.rs`) port the Python provenance suite case for case, adapted for the engine-name divergence in section 3, and additionally assert that provenance survives every error path the Rust engine can take, not only the observation-before-birth case the Python suite covers.
