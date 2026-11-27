<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Scripts

| Script | What it does |
| --- | --- |
| `s/test` | `cargo test` for the default build, including every conformance suite, then for the engine alone (`--no-default-features`). |
| `s/lint` | `cargo fmt --check`, and `cargo clippy -D warnings` for the default build and the engine alone, as CI runs them. |
| `s/soup` | Regenerates `qms/soup.md`, the IEC 62304 SOUP register, from `Cargo.lock`. `--check` verifies it is current, as CI does. |
| `s/deny` | `cargo deny check`: dependency licences, advisories, bans, and sources (`deny.toml`). |
| `s/conformance` | The conformance suites with their agreement reports: R vectors, Python goldens, SciPy, NumPy, CPython, the engine conformance cases, and the FastAPI HTTP replays (every operation of every reference). |
| `s/reference-python` | Creates the pinned Python reference environment (`rcpchgrowth==4.6.5` from PyPI) with `uv`, in `~/.cache/opengrowth/reference-python` or `$OPENGROWTH_REFERENCE_VENV`. |
| `s/reference-tables` | Prints a directory of `rcpchgrowth` 4.6.5's own reference tables, downloading the wheel from PyPI on first use. CI checks every file in `data/` against them. |
| `s/goldens` | Regenerates `conformance/expected/` from the reference environments. Pass generator names to run only some. |
| `s/reference-server` | Checks out the upstream FastAPI server at a pinned commit and installs its dependencies, in `~/.cache/opengrowth/reference-server` or `$OPENGROWTH_REFERENCE_SERVER`. Used by `s/goldens http_uk_who` and `s/bench`. |
| `s/bench-round0` | Benchmark round 0: the two engines in-process, pinned to the same CPU on the fastest core. `--json` for JSON. |
| `s/bench` | Benchmark rounds 0 to 4: the two engines, then the two HTTP servers under `oha`, each server on the same whole cores. About 15 minutes; `--quick` for a smoke run. Writes to `.private/bench/`. |
| `s/bench-page` | Renders `docs/benchmarks.md` from `docs/benchmarks.json`. Given a `results.json` from `s/bench`, publishes that run first; `--check` fails if the page is out of date, as CI does. |
| `s/openapi` | Regenerates `docs/api/openapi.json`, the OpenAPI document the documentation publishes, from `opengrowth openapi`. `tests/openapi.rs` fails while it is out of date. |
| `s/cli-docs` | Regenerates `docs/cli/`, the command-line reference, from the program's own help. `tests/cli_docs.rs` fails while it is out of date. |
| `s/docs` | Serves the documentation site locally with hot reload, on the first free port from 8000. Needs `.venv` set up from `requirements.txt` once. |
| `s/build-docs` | Builds the documentation site into `site/`. `--strict` fails on any warning, as CI does. |
| `s/check-wording` | Fails on a word `qms/intended-purpose.md` reserves, in the documentation site or the README, unless it says what opengrowth does not do. |
| `s/check-differences` | Fails if `docs/differences.md` does not name every difference from the upstream server that a conformance test allows. `--list` prints them. |
| `s/check-docs-links` | Fails if the built site links to a page, file, or anchor of its own that does not exist. Run `s/build-docs` first. |
| `s/check-docs-nav` | Fails if a page in `docs/` is missing from the nav in `mkdocs.yml`, or the nav names a page that does not exist. |

