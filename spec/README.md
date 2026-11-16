# OpenGrowth specs

Design specifications for OpenGrowth, the Open Health Technology Foundation's Rust reimplementation of the `rcpchgrowth` growth calculation engine.

## Reading order

1. [standard.md](standard.md) - the normative specification: ownership, compatibility profiles, numerical equivalence policy, architecture, surfaces, and conformance strategy. Read this before writing code.
2. [ubiquitous-language.md](ubiquitous-language.md) - canonical terms. The Python engine, the API server, and the chart component currently use three overlapping vocabularies for the same concepts; this file picks one.
3. [provenance.md](provenance.md) - the calculation-provenance contract. Read this before touching the result object; it is a safety control with non-obvious invariants.
4. [roadmap.md](roadmap.md) - phased delivery with stable `R<number>` codes.
5. [queries.md](queries.md) - open decisions that need a human, including the UK90 licensing question and the bug-for-bug compatibility choices.

## Conventions

- Every actionable roadmap item carries a stable `R<number>` code. Never renumber, never reuse a retired code. Refer to items as `R7`, `blocks R12`, `supersedes R4`.
- Every open decision carries a stable `Q<number>` code and names what it blocks.
- The specification records design intent. User-facing documentation will live in this repository's docs site (`R30`). Upstream reference-data provenance is recorded in [growth-references](https://github.com/rcpch/growth-references).
- If the implementation intentionally diverges from this specification, update the specification in the same change or open a named `Q` item. A silent divergence is a defect.

## Status

Pre-implementation. No Rust code exists yet. The specification is being written first because the hardest problems in this project are contractual (what "identical" means) and legal (what data may be redistributed), not mechanical.
