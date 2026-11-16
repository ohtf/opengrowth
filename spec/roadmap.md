# Roadmap

Legend: `[x]` done, `[~]` in progress, `[ ]` not started

Every actionable item has a stable `R<number>` code. Codes are never renumbered and never reused. Standing constraints are plain bullets and carry no checkbox.

## Standing constraints

- The independent R and WHO `anthro` vectors are the clinical-accuracy gate. The Python engine is the reference implementation for everything they do not cover. Goldens are captured from it, never hand-edited.
- OpenGrowth is an OHTF project, not an RCPCH project. Upstream attribution is preserved; RCPCH branding is not used.
- No UK90-derived table is committed or published while `Q1` is open.
- No reference table ships without recorded provenance, licence, and integrity tests.
- No phase is complete while its differential tests are red.
- Preterm and early-infant behaviour is the highest-risk area throughout and gets clinical review, not just passing tests.

## Phase 0 - contract, licensing, and evidence

Nothing here writes engine code. The purpose is to make the target unambiguous before anyone starts matching numbers.

- [~] **R1 - Write the specification, roadmap, terminology, and open queries.** This document set.
- [~] **R2 - Determine redistribution terms per table.** For UK90, UK-WHO, WHO, CDC, Trisomy 21 UK, Trisomy 21 AAP, Turner, and the correlation matrices: record source, owner, licence, and whether the table may be embedded in public artefacts or must be runtime-supplied. UK90 licence application submitted to UKRI; awaiting response. Records the outcome of `Q1`. Does not block any implementation phase.
- [ ] **R3 - Capture the differential conformance corpus.** Import the independent vectors (Tim Cole's ~4,000 R-generated fictional children, the WHO `anthro` gold set) into `conformance/independent/` with their provenance. Pin a Python version (`Q10`), run it over the existing fixtures plus **all** the coverage gaps in `standard.md` section 8.2, and freeze the outputs as goldens with a recorded generator command. That includes the currently untested Turner and Trisomy 21 numerical output, the full `Measurement` serialised shape, the date and gestation boundary matrix, the validation limits at exactly ±8 and ±15 SD, and bulk cardinality. Server-level behaviour without a Python test today - authorisation, CORS, 404, 405, trailing slash - is captured here as observed behaviour and consumed by `R15`. Also captures current behaviour for the Category 2 and Category 3 defects so divergence is measured rather than assumed. Blocks `R9` onward.
- [ ] **R4 - Classify every known Python defect.** Assign each to Category 1, 2, or 3 per `standard.md` section 6, with a named owner for each Category 2 clinical decision. Resolves `Q2`, `Q6`, `Q7`.

- [ ] **R24 - Set up the OHTF repository.** Create `ohtf/opengrowth` on GitHub with private vulnerability reporting, the `hazard` issue label and template, DCO checks, and branch protection appropriate to the contributor count. List the project as Experimental on the OHTF projects page via a PR to `ohtf/openhealthtechnology.org-website`.
- [ ] **R25 - Write `NOTICE.md`.** State that OpenGrowth is a modified, Rust reimplementation of `rcpchgrowth`, name the upstream version and commit ported from and the date, preserve RCPCH's copyright, and list third-party data and its terms. Satisfies AGPL section 5 and `standard.md` section 3.1.
- [ ] **R27 - Open the hazard log.** Create the initial hazard entries from `SAFETY.md`, and copy in the upstream dGC hazards that apply to the engine (starting with #174) with cross-references.

## Phase 1 - foundations

- [ ] **R5 - Establish numerical primitives.** LMS forward and inverse, normal CDF and inverse CDF, linear and four-point Lagrange cubic interpolation, and Python-compatible round-half-to-even. Includes the measured comparison of candidate normal-distribution implementations against a SciPy grid, and confirmation of the tolerance values. Resolves `Q4` and `Q5`. Good first feasibility slice: pair it with term-infant UK-WHO SDS against the R vectors to show early whether the 1e-3 gate is met.
- [ ] **R6 - Scaffold the workspace.** Cargo workspace with the `opengrowth-*` crate skeletons and a single `opengrowth` binary, `s/` scripts, CI enforcing format, clippy, tests, and DCO sign-off, actions pinned to commit SHAs, REUSE-compliant SPDX headers, Dependabot, and the licence and safety files. Includes the CI data-access approach from `Q12`.
- [ ] **R7 - Build the data crate.** `LmsPoint` model, the provider interface covering embedded and runtime-supplied tables, embedded open tables with provenance, licence, and checksum metadata, per-table feature flags choosing embedded or runtime-supplied, the registry test asserting every table has provenance, a licence, and a checksum, integrity tests for point counts, ordering, and endpoints, and the reference-data version scheme used in provenance.

## Phase 2 - clinical calculation

- [ ] **R8 - Implement dates and gestation.** Decimal age, estimated delivery date, calendar-age phrasing, corrected gestational age, and the reference-specific correction rules including the CDC and WHO term reset and the two-year correction cutoff. Independently safety-critical; tested against the full boundary matrix from `R3`.
- [ ] **R9 - Implement reference selection.** In ascending order of complexity: Turner, UK Trisomy 21, AAP Trisomy 21, WHO, CDC, UK-WHO. UK-WHO last because its preterm and early-infant transitions carry the highest documented risk.
- [ ] **R10 - Implement SDS and measurement conversion.** `sds_for_measurement` and `measurement_from_sds`, including the CDC extended-BMI sigma tail and the WHO extreme-tail linear extrapolation, with its documented asymmetry between the two directions. Gated on the UK-WHO, WHO, CDC, and gold fixtures.
- [ ] **R11 - Implement centile bands and clinical comment strings.** Exact string reproduction, including the "on or near" band width and the measurement-method renaming.
- [ ] **R22 - Implement calculation provenance.** Typed struct with a single serialisation point, emitting `growth_reference` and the `calculation_engine` block, with the Rust engine reporting its own name rather than impersonating the Python engine. Port the Python provenance conformance suite case for case, including the drift guard asserting all six references are covered and the case proving provenance survives a failed numerical calculation. Includes the additive `reference_data` block. Mitigates the hazard first recorded as RCPCH dGC #174; see [provenance.md](provenance.md). Ships with `R12`.
- [ ] **R12 - Implement the `Measurement` result object.** Complete nested shape, provenance, error capture semantics where a validation failure is recorded but calculation still proceeds, and both chronological and corrected plottable branches. Full golden serialisation test. Carries the provenance object from `R22` as its first key.

- [ ] **R23 - Implement runtime-supplied reference data.** `opengrowth data import`, `opengrowth data status`, the checksum manifest, verification and rejection of tampered or truncated input, the platform data directory with flag and environment overrides, and the typed `ReferenceDataUnavailable` error that never falls back to another reference. Required for UK-WHO in public artefacts while `Q1` is open. See `standard.md` section 7.2.

## Phase 3 - surfaces

- [ ] **R13 - Implement chart-coordinate generation.** Centile collections, age grids, component ordering, and the four-decimal rounding. Validated against the ~123,000 WHO chart points.
- [ ] **R14 - Build the CLI.** Covers the surface of the existing Python CLI, follows the house Rust CLI conventions, requires sex and gestation explicitly, and includes shell completions and a machine-readable `--schema` for agents.
- [ ] **R15 - Build the HTTP server to Profile A.** Exposed as `opengrowth serve`. All 25 operations, bulk partial-failure semantics, explicit-null serialisation, the four rounded centile fields, bearer-token authorisation, and a generated OpenAPI document diffed in CI. Chart assets become build-time artefacts rather than startup side effects.
- [ ] **R16 - Implement mid-parental height and secondary calculations.** Including the existing double-rounding of the centile and the parental ±8 SD rejection.

- [ ] **R26 - Build the MCP server.** Exposed as `opengrowth mcp`, over stdio and streamable HTTP, sharing request types with the CLI and server, per house-style `mcp.md`. Tool descriptions carry the clinical-use boundary; results carry full provenance.
- [ ] **R30 - Build the documentation site.** Zensical site in `docs/` with generated CLI, OpenAPI, and MCP references, a data-import guide for UK90, migration notes from the RCPCH API, and clinical background with attribution.

## Phase 4 - beyond parity

- [ ] **R17 - Decide the fate of the experimental dynamic-growth functions.** Velocity, acceleration, thrive lines, and conditional weight gain are labelled experimental and not for clinical use, and several are broken. Either port them behind an explicit experimental flag with the defects fixed, or drop them and say so. Not required for the first release.
- [ ] **R18 - Add PyO3 bindings.** Published as `opengrowth` on PyPI, built from this repository with maturin, so Python users, notebooks, and research workflows get the same engine. Offers an import surface close to `rcpchgrowth` to ease migration, without using that name. Requires `R12` to be stable.
- [ ] **R19 - Set up distribution.** Release automation via the house-style cargo-dist and auto-tag cascade, versioning, crates.io, a Homebrew tap, container images, and PyPI wheels. Every artefact is checked to contain no table whose terms forbid redistribution (`Q1`).
- [ ] **R20 - Reach Profile B for the replay corpus.** Optional. Only for endpoints where byte-level replay comparison is actually planned.
- [ ] **R21 - Complete the clinical safety file set.** Safety case, hazard log, and evidence per house-style `clinical-safety.md`, the equivalence evidence from `Q9`, the regulatory position from `Q11`, and guidance for deployers on their DCB0160 duties and on detecting divergence in production.
- [ ] **R28 - Consolidate the React side into one repository.** Plan, with whoever maintains the forks, how the chart component library and demo client combine into one React repository that targets the OpenGrowth API, completing the two-repository target. Lives outside this repository; tracked here because it depends on `R15`.
- [ ] **R29 - Benchmark against the Python server.** Measure throughput, latency, memory, and cost per million calculations for single and bulk requests on equal hardware, and publish the method and results. Evidence for the performance claim in `standard.md` section 1.
