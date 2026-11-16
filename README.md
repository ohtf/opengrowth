# OpenGrowth

An open-source growth calculation engine for children: SDS, centiles, centile bands, and chart coordinates against the UK-WHO, WHO, CDC, Trisomy 21, and Turner syndrome growth references. One Rust binary provides the library, CLI, HTTP API, and MCP server.

> **Status: Experimental, specification only.** No engine code exists yet. The design is being written first, in [spec/](spec/), because the hardest problems are contractual and legal rather than mechanical. Do not use this for anything clinical.

OpenGrowth is an [Open Health Technology Foundation](https://openhealthtechnology.org) project, hosted at [github.com/ohtf/opengrowth](https://github.com/ohtf/opengrowth). Its OHTF lifecycle status is **Experimental**.

## What this is

One engine driving several surfaces from a single implementation:

- a **library** for embedding in Rust programs
- a **command-line tool**, so people and AI agents can run growth calculations locally with no runtime to install
- an **MCP server**, local and remote, built on the same engine
- an **HTTP API** compatible with the [RCPCH Digital Growth Charts API](https://growth.rcpch.ac.uk), so existing integrators can switch without changing their clients
- later, **Python bindings**

Today the equivalent functionality is spread across a Python package, an API server, a CLI wrapper, a React component, a demo client, and a documentation site in separate repositories. The aim is two repositories: this one (engine, CLI, MCP, API, Python bindings, docs) and one React repository (chart component and demo client).

## Relationship to RCPCH

OpenGrowth is an independent derivative of [`rcpchgrowth`](https://github.com/rcpch/rcpchgrowth-python), the Python growth engine published by the Royal College of Paediatrics and Child Health under AGPL-3.0-or-later. It is **not an RCPCH project** and is not affiliated with or endorsed by RCPCH. References to RCPCH software describe compatibility and attribution only.

- **Clinical behaviour is defined by the published methods and the Python reference implementation.** This is a reimplementation, not a new clinical method. See [spec/standard.md](spec/standard.md).
- **Attribution.** The calculation logic is ported from `rcpchgrowth`, copyright Royal College of Paediatrics and Child Health. That copyright is preserved in this repository's notices and in the SPDX headers of ported files.

## What this is not

- **Not a new clinical method.** No new references and no reinterpretation of existing ones.
- **Not a substitute for clinical judgement.** See [SAFETY.md](SAFETY.md).
- **Not the home of reference data provenance.** The upstream record of source datasets is [growth-references](https://github.com/rcpch/growth-references).
- **Not usable yet.**

## Reference data

The reference datasets are not covered by this repository's code licence and carry their own terms. WHO and CDC data are open. The UK90 reference, which UK-WHO depends on, was produced under the Medical Research Council and is now owned by UK Research and Innovation (UKRI). A licence application to UKRI is pending.

OpenGrowth is designed to work under either outcome:

- if redistribution is permitted, UK90 tables are embedded in the binary like the others
- if it is not, the binary ships without them, and users who hold their own licence supply the data at install time through a verified import command, much as SNOMED CT releases are loaded

Every table records its own provenance and terms, enforced by a test. See [spec/standard.md](spec/standard.md) section 3 and `Q1` in [spec/queries.md](spec/queries.md).

## Development

Nothing to run yet. Tooling arrives with `R6` in the [roadmap](spec/roadmap.md).

## Contributing

Read [agent-instructions.md](agent-instructions.md) and [SAFETY.md](SAFETY.md) first. Do not change clinical behaviour, reference data, or the API contract without the evidence those documents require.

Sign off every commit under the [Developer Certificate of Origin](https://openhealthtechnology.org/community/dco/) (`git commit -s`). Report security issues as described in [SECURITY.md](SECURITY.md).

## Licence

Code is licensed under [AGPL-3.0-or-later](LICENSE), inherited from the Python engine this project ports. Written content is CC-BY-SA-4.0.

Copyright Open Health Technology Foundation contributors. Portions derived from `rcpchgrowth` are copyright Royal College of Paediatrics and Child Health.

Reference datasets carry their own separate terms, which take precedence.
