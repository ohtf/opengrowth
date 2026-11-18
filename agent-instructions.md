# Agent instructions

opengrowth is an Open Health Technology Foundation project: a Rust reimplementation of the `rcpchgrowth` growth calculation engine, driving a library, CLI, MCP server, HTTP API, and later Python bindings from one engine. It is an independent derivative of RCPCH's AGPL-licensed Python engine, not an RCPCH project. This file is the entry point for AI coding agents. Read it before changing anything.

The project is **experimental**. The engine computes UK-WHO SDS and centiles, bit-identical to the Python engine on the conformance corpus; the other references, the full result object, the HTTP server, and the MCP server do not exist yet. Check the [roadmap](spec/roadmap.md) before starting work. The UK90 tables are in the repository for research only, pending a UKRI licence (`Q1`).

One crate, `opengrowth`, with features: the engine (every module outside `store`, `server`, `cli`), `store` (tables: embedded and imported), `server` (the HTTP API, `opengrowth serve`), `cli` (the `opengrowth` and `og` binaries), and `embedded-data` (the tables in `data/`). Conformance suites are in `tests/`. Do not split out a crate unless something else needs to reuse it on its own (`spec/standard.md` section 7).

## Read first

1. [README.md](README.md) - what this is and is not.
2. [spec/standard.md](spec/standard.md) - the normative design. Ownership, compatibility profiles, numerical tolerance, defect policy, architecture.
3. [spec/ubiquitous-language.md](spec/ubiquitous-language.md) - canonical terms and the name collisions that cause defects.
4. [spec/roadmap.md](spec/roadmap.md) and [spec/queries.md](spec/queries.md) - what to work on, and what is blocked.
5. [SAFETY.md](SAFETY.md) - the clinical-use boundary.
6. [qms/README.md](qms/README.md) and [qms/intended-purpose.md](qms/intended-purpose.md) - the quality management system and what opengrowth is for.
7. [CONTRIBUTING.md](CONTRIBUTING.md) and [spec/versioning.md](spec/versioning.md) - what needs extra evidence, what is a contract, and how clinical changes are released.
8. [pacharanero/house-style](https://github.com/pacharanero/house-style) - cross-repository engineering standards, and the [OHTF community policies](https://openhealthtechnology.org/community/) for DCO, security, and conduct. The RCPCH house style does not apply to this project.

## Core invariants

- **The name is written in lowercase: opengrowth**, including at the start of a sentence. Never "OpenGrowth".
- **opengrowth is an anthropometric calculation tool, and a Class I medical device in development.** It never diagnoses, interprets, alerts, recommends, or predicts. In user-facing text, follow the wording rules in [qms/intended-purpose.md](qms/intended-purpose.md). Do not build a feature listed under "Features that would change the class" in [qms/regulatory-strategy.md](qms/regulatory-strategy.md) without a regulatory impact assessment approved by the PRRC.
- **The QMS is in the repository.** `qms/` documents, `.github/ISSUE_TEMPLATE/`, `.github/labels.yml`, and `.github/CODEOWNERS` are controlled documents. After any dependency change run `s/soup` and commit `qms/soup.md`; CI checks it.
- **This is not an RCPCH project.** Do not describe it as RCPCH software, use RCPCH branding, or imply RCPCH endorsement. Naming RCPCH for compatibility ("compatible with the RCPCH Digital Growth Charts API") and attribution is correct and required.
- **Preserve upstream attribution.** Files that port logic from `rcpchgrowth` carry the RCPCH copyright line alongside the OHTF one. See `standard.md` section 3.1.
- **Published methods and the reference implementation define clinical behaviour.** Never change a number to make a test pass. If Rust disagrees with a golden, the default assumption is that Rust is wrong. Investigate before adjusting anything.
- **Never hand-edit a conformance golden.** Goldens are captured from a pinned Python version by a recorded command, or are independent vectors from the R implementation. A golden change without a stated rationale is a defect.
- **Never widen a numerical tolerance to get green.** A wide tolerance hides algorithmic divergence. Investigate the cause instead. See `standard.md` section 5.
- **The engine stays a leaf**, and builds with `--no-default-features`. No clock, no I/O, no global state, no async runtime, no HTTP types. Timestamps and I/O belong to callers. This is what keeps the engine embeddable and deterministically testable.
- **No reference table without provenance and a licence.** A table lacking either fails the registry test. See `standard.md` section 3.3.
- **The UK90 tables are for research only until UKRI grants a licence (`Q1`).** Never describe opengrowth as usable for patient care, and never publish it to a package registry, while `Q1` is open. Do not change files in `data/` except by copying a new verified source with a manifest update.
- **Provenance reports what was computed, never what was requested.** A surface that derives provenance from its own routing has defeated the point. This is a safety control, not a convenience field.
- **Use Python-compatible rounding.** `f64::round()` is half-away-from-zero; Python is half-to-even. Use `opengrowth::round::python_round`. Never let `f64::round()` reach an output field.
- **Parse floats exactly.** `serde_json` must keep its `float_roundtrip` feature; without it, parsed tables and goldens can be 1 ulp off and bit-identity with Python is lost. Don't add a JSON parser that lacks an exact mode.
- **The HTTP API is byte-identical to upstream outside `provenance`.** Response field order follows the upstream Pydantic models, floats go through `pyjson`, and request validation follows Pydantic's order and messages. Any new difference goes into `KNOWN_DIFFERENCES` in `tests/http_uk_who.rs` and `spec/standard.md` section 6.3, with a reason.
- **Keep the platform `pow`, `ln`, and `exp`.** They make SDS bit-identical to CPython on Linux; the portable `libm` versions do not (`conformance/README.md`).
- **Do not vendor restrictively licensed source material.** Reimplement from published method descriptions. Named exclusions are in `standard.md` section 3.4.
- **Preterm and early-infant behaviour is the highest-risk area.** Changes there need clinical review, not just passing tests.

## Workflow

See [s/README.md](s/README.md). The main ones:

- `s/test` - run the test suite (skips UK-WHO conformance, which needs reference tables)
- `s/lint` - format check and clippy
- `s/deny` - dependency policy (licences, advisories, sources)
- `s/conformance` - the differential suite against the R vectors, Python goldens, and SciPy grid
- `s/reference-python` and `s/goldens` - the pinned Python reference environment, and regenerating goldens from it

## Before every commit

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny --locked check
```

CI enforces all three. Do not commit red.

## Git workflow

- Sign off every commit under the DCO: `git commit -s`.
- Commit each validated coherent parcel on `main` while the project has one or two contributors; move to branches and PRs once others are contributing regularly.
- Do not push unless the user explicitly asks.
- PR descriptions open with a two-or-three-line summary and fold the rest into `<details><summary>`.

## Assurance

Treat agent output as a capable junior's work: review the diff and run the validation, and do not accept a green suite as evidence that the behaviour is right.

For anything touching clinical calculation, reference selection, age correction, or reference data, validate independently against the R vectors, the Python implementation, or the published source. Agent-generated tests are not sufficient evidence on their own for clinical behaviour; they encode the author's assumptions, which is exactly what is in question.

Record safety-relevant changes and their evidence in the same change as the implementation.

## Publicity

- Owner: Open Health Technology Foundation.
- Channels: [openhealthtechnology.org](https://openhealthtechnology.org), via a PR against `ohtf/openhealthtechnology.org-website`, for major announcements only.

Suggest a post at milestones per [house-style publicity](https://github.com/pacharanero/house-style/blob/main/publicity.md). Never post without approval.

## Approval required

Ask before:

- creating a GitHub remote, pushing, or publishing anything
- publishing the crate or any artefact to a registry, which is gated on `Q1`
- changing clinical behaviour, including correcting a Category 2 defect
- adding, replacing, or regenerating a reference table
- changing the API contract in a way that is not purely additive
