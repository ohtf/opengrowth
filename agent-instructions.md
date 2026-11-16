# Agent instructions

OpenGrowth is an Open Health Technology Foundation project: a Rust reimplementation of the `rcpchgrowth` growth calculation engine, driving a library, CLI, MCP server, HTTP API, and later Python bindings from one engine. It is an independent derivative of RCPCH's AGPL-licensed Python engine, not an RCPCH project. This file is the entry point for AI coding agents. Read it before changing anything.

The project is currently **specification only**. There is no engine code. If you are about to write Rust, check the [roadmap](spec/roadmap.md) first. Phase 0 does not block everything: the numerical primitives (`R5`), the workspace scaffold (`R6`), and the data crate (`R7`) can start immediately. What Phase 0 blocks is anything whose correctness is judged against the reference implementation, which is `R9` onward. Distribution of UK90-derived tables is gated on `Q1`.

## Read first

1. [README.md](README.md) - what this is and is not.
2. [spec/standard.md](spec/standard.md) - the normative design. Ownership, compatibility profiles, numerical tolerance, defect policy, architecture.
3. [spec/ubiquitous-language.md](spec/ubiquitous-language.md) - canonical terms and the name collisions that cause defects.
4. [spec/roadmap.md](spec/roadmap.md) and [spec/queries.md](spec/queries.md) - what to work on, and what is blocked.
5. [SAFETY.md](SAFETY.md) - the clinical-use boundary.
6. [pacharanero/house-style](https://github.com/pacharanero/house-style) - cross-repository engineering standards, and the [OHTF community policies](https://openhealthtechnology.org/community/) for DCO, security, and conduct. The RCPCH house style does not apply to this project.

## Core invariants

- **This is not an RCPCH project.** Do not describe it as RCPCH software, use RCPCH branding, or imply RCPCH endorsement. Naming RCPCH for compatibility ("compatible with the RCPCH Digital Growth Charts API") and attribution is correct and required.
- **Preserve upstream attribution.** Files that port logic from `rcpchgrowth` carry the RCPCH copyright line alongside the OHTF one. See `standard.md` section 3.1.
- **Published methods and the reference implementation define clinical behaviour.** Never change a number to make a test pass. If Rust disagrees with a golden, the default assumption is that Rust is wrong. Investigate before adjusting anything.
- **Never hand-edit a conformance golden.** Goldens are captured from a pinned Python version by a recorded command, or are independent vectors from the R implementation. A golden change without a stated rationale is a defect.
- **Never widen a numerical tolerance to get green.** A wide tolerance hides algorithmic divergence. Investigate the cause instead. See `standard.md` section 5.
- **`opengrowth-core` stays a leaf.** No clock, no I/O, no global state, no async runtime, no HTTP types. Timestamps and I/O belong to callers. This is what keeps the engine embeddable and deterministically testable.
- **No reference table without provenance and a licence.** A table lacking either fails the registry test. See `standard.md` section 3.3.
- **Never commit or publish UK90-derived tables while `Q1` is open.** That includes the data crate, fixtures that contain LMS values, release artefacts, and CI caches. Calculated outputs (SDS, centiles) in goldens are not tables and are fine.
- **Provenance reports what was computed, never what was requested.** A surface that derives provenance from its own routing has defeated the point. This is a safety control, not a convenience field.
- **Use Python-compatible rounding.** `f64::round()` is half-away-from-zero; Python is half-to-even. Never let `f64::round()` reach an output field.
- **Do not vendor restrictively licensed source material.** Reimplement from published method descriptions. Named exclusions are in `standard.md` section 3.4.
- **Preterm and early-infant behaviour is the highest-risk area.** Changes there need clinical review, not just passing tests.

## Workflow

Scripts arrive with `R6`. The intended surface:

- `s/test` - run the test suite
- `s/lint` - format check and clippy
- `s/conformance` - run the differential suite against captured goldens and the R vectors

## Before every commit

```sh
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
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
- committing, publishing, or caching any UK90-derived table, which is gated on `Q1`
- changing clinical behaviour, including correcting a Category 2 defect
- adding, replacing, or regenerating a reference table
- changing the API contract in a way that is not purely additive
