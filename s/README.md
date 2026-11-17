<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Scripts

| Script | What it does |
| --- | --- |
| `s/test` | `cargo test --workspace`. Conformance tests that need reference data are skipped. |
| `s/lint` | `cargo fmt --check` and `cargo clippy -D warnings`, as CI runs them. |
| `s/conformance` | The conformance suite: independent R vectors, Python goldens, and the SciPy normal grid. Needs reference data (see below). |
| `s/reference-python` | Creates the pinned Python reference environment (`rcpchgrowth==4.6.5` from PyPI) with `uv`, in `~/.cache/opengrowth/reference-python` or `$OPENGROWTH_REFERENCE_VENV`. |
| `s/goldens` | Regenerates `conformance/expected/` from the reference environment. Pass generator names to run only some. |

Reference data for `s/conformance` comes from `$OPENGROWTH_REFERENCE_DATA_DIR`, or from the reference environment's `rcpchgrowth/data_tables` directory. It is never committed to this repository (`spec/queries.md` Q1).
