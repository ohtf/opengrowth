# opengrowth specs

Design specifications for opengrowth, the Open Health Technology Foundation's Rust reimplementation of the `rcpchgrowth` growth calculation engine.

## Reading order

1. [standard.md](standard.md) - the normative specification: ownership, compatibility profiles, numerical equivalence policy, architecture, surfaces, and conformance strategy. Read this before writing code.
2. [ubiquitous-language.md](ubiquitous-language.md) - canonical terms. The Python engine, the API server, and the chart component currently use three overlapping vocabularies for the same concepts; this file picks one.
3. [provenance.md](provenance.md) - the calculation-provenance contract. Read this before touching the result object; it is a safety control with non-obvious invariants.
4. [roadmap.md](roadmap.md) - phased delivery with stable `R<number>` codes.
5. [queries.md](queries.md) - open decisions that need a human, including the UK90 licensing question and the bug-for-bug compatibility choices.
6. [benchmark.md](benchmark.md) - how the Python and Rust servers are compared for speed, load, concurrency, and cost.
7. [versioning.md](versioning.md) - what is a contract, how clinical changes are released, and how things are deprecated.

## Conventions

- Every actionable roadmap item carries a stable `R<number>` code. Never renumber, never reuse a retired code. Refer to items as `R7`, `blocks R12`, `supersedes R4`.
- Every open decision carries a stable `Q<number>` code and names what it blocks.
- The specification records design intent. User-facing documentation will live in this repository's docs site (`R30`). Upstream reference-data provenance is recorded in [growth-references](https://github.com/rcpch/growth-references).
- If the implementation intentionally diverges from this specification, update the specification in the same change or open a named `Q` item. A silent divergence is a defect.

## Status

Experimental. The first slice (UK-WHO SDS, the numerical primitives, runtime-supplied data, and the CLI) is implemented and measured; see `conformance/README.md`.
