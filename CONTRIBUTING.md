# Contributing

Thank you for helping. opengrowth calculates children's growth measurements against reference populations for use in clinical care, and is being developed as a Class I medical device ([qms/](qms/README.md)), so some changes need more evidence than passing tests. This page says which, and what to bring.

## The basics

- Sign off every commit under the [Developer Certificate of Origin](https://openhealthtechnology.org/community/dco/): `git commit -s`. CI rejects commits without a `Signed-off-by` line.
- Follow the [OHTF code of conduct](https://openhealthtechnology.org/community/code-of-conduct/).
- Report security vulnerabilities privately, as described in [SECURITY.md](SECURITY.md), not in issues.
- Report anything that could affect a clinical result as an issue labelled `hazard` ([SAFETY.md](SAFETY.md)).
- Read [agent-instructions.md](agent-instructions.md) first. It is written for AI agents but is the shortest accurate summary of the project's rules for anyone.

## Before opening a pull request

```sh
s/lint          # rustfmt and clippy, as CI runs them
s/test          # unit tests and the SciPy and CPython comparisons
s/deny          # dependency licences, advisories, and sources
s/conformance   # if you touched calculation, dates, validation, or the API (needs reference tables)
```

`s/conformance` needs the reference tables, which are not in the repository; see [s/README.md](s/README.md). CI cannot run the UK-WHO conformance tests yet (`spec/queries.md` Q12), so a maintainer runs them before merging anything that could affect them.

## Changes that need more than green tests

| Change | What to bring |
| --- | --- |
| Any change to a calculated number, a clinical string, or which reference is used | Evidence against the independent vectors or the published method, the clinical safety review in [SAFETY.md](SAFETY.md), and an entry in the changelog's clinical changes section. Never change a number to make a test pass. |
| Preterm or early-infant behaviour | All of the above, plus review by a clinician. |
| Reference tables | Source, licence, transformation, and validation evidence, in the same change. Never commit a UK90-derived table (`spec/queries.md` Q1). |
| A conformance golden | Regenerate it with `s/goldens`; never edit it by hand. Say why it changed in the commit. |
| The HTTP API, CLI JSON, or MCP tool schemas | Follow [spec/versioning.md](spec/versioning.md). A new difference from the upstream API goes in `KNOWN_DIFFERENCES` with a reason. |
| Anything that adds interpretation, alerts, recommendations, or predictions, or changes what opengrowth says it is for | A regulatory impact assessment against [qms/intended-purpose.md](qms/intended-purpose.md) before the work starts. Such features could change the device class. |
| A new dependency | Check it is maintained and its licence is in `deny.toml`. Explain why it is needed. |

## Style

- Conventional commits: `feat(core):`, `fix(server):`, `docs:`, `chore(deps):`, `ci:`.
- Code follows the surrounding code; comments explain why, not what.
- Documentation: British English, plain and concrete, hyphens rather than em dashes, and Markdown paragraphs on one line rather than hard-wrapped.
- New files carry SPDX headers; `reuse lint` must pass.

## Licence of contributions

Code is contributed under [AGPL-3.0-or-later](LICENSE) and documentation under CC-BY-SA-4.0, the licences of this repository. Copyright stays with you, recorded as "Open Health Technology Foundation contributors".
