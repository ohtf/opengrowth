<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Scripts

| Script | What it does |
| --- | --- |
| `s/test` | `cargo test` for the default build, including every conformance suite, then for the engine alone (`--no-default-features`). |
| `s/lint` | `cargo fmt --check`, and `cargo clippy -D warnings` for the default build and the engine alone, as CI runs them. |
| `s/soup` | Regenerates `qms/soup.md`, the IEC 62304 SOUP register, from `Cargo.lock`. `--check` verifies it is current, as CI does. |
| `s/deny` | `cargo deny check`: dependency licences, advisories, bans, and sources (`deny.toml`). |
| `s/conformance` | The conformance suites with their agreement reports: R vectors, Python goldens, SciPy, NumPy, CPython, and the FastAPI HTTP replays (every UK-WHO, WHO, and CDC operation, and mid-parental height). |
| `s/reference-python` | Creates the pinned Python reference environment (`rcpchgrowth==4.6.5` from PyPI) with `uv`, in `~/.cache/opengrowth/reference-python` or `$OPENGROWTH_REFERENCE_VENV`. |
| `s/goldens` | Regenerates `conformance/expected/` from the reference environments. Pass generator names to run only some. |
| `s/reference-server` | Checks out the upstream FastAPI server at a pinned commit and installs its dependencies, in `~/.cache/opengrowth/reference-server` or `$OPENGROWTH_REFERENCE_SERVER`. Used by `s/goldens http_uk_who` and `s/bench`. |
| `s/bench-round0` | Benchmark round 0: the two engines in-process. |
| `s/bench` | Benchmark rounds 1 to 4: the two HTTP servers under `oha`. Writes to `.private/bench/`. `--quick` for a smoke run. |

