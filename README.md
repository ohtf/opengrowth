# opengrowth

An open-source growth calculation engine for children: SDS, centiles, centile bands, and chart coordinates against the UK-WHO, WHO, CDC, Trisomy 21, and Turner syndrome growth references. One Rust binary provides the library, CLI, HTTP API, and MCP server.

> **Status: Experimental.** All six references work in the CLI and the HTTP API. All 25 API operations return the same bytes as the RCPCH FastAPI server, outside provenance, for every captured request apart from the differences listed in [spec/standard.md](spec/standard.md) section 6.3, most of them requests where upstream fails with a 500. The MCP server is not built yet. Do not use this for anything clinical.

opengrowth is an [Open Health Technology Foundation](https://openhealthtechnology.org) project, hosted at [github.com/ohtf/opengrowth](https://github.com/ohtf/opengrowth). Its OHTF lifecycle status is **Experimental**.

## What this is

One engine driving several surfaces from a single implementation:

- a **library** for embedding in Rust programs
- a **command-line tool**, so people and AI agents can run growth calculations locally with no runtime to install
- an **MCP server**, local and remote, built on the same engine
- an **HTTP API** compatible with the [RCPCH Digital Growth Charts API](https://growth.rcpch.ac.uk), so existing integrators can switch without changing their clients
- later, **Python bindings**

Today the equivalent functionality is spread across a Python package, an API server, a CLI wrapper, a React component, a demo client, and a documentation site in separate repositories. The aim is two repositories: this one (engine, CLI, MCP, API, Python bindings, docs) and one React repository (chart component and demo client).

## Relationship to RCPCH

opengrowth is an independent derivative of [`rcpchgrowth`](https://github.com/rcpch/rcpchgrowth-python), the Python growth engine published by the Royal College of Paediatrics and Child Health under AGPL-3.0-or-later. It is **not an RCPCH project** and is not affiliated with or endorsed by RCPCH. References to RCPCH software describe compatibility and attribution only.

- **Clinical behaviour is defined by the published methods and the Python reference implementation.** This is a reimplementation, not a new clinical method. See [spec/standard.md](spec/standard.md).
- **Attribution.** The calculation logic is ported from `rcpchgrowth`, copyright Royal College of Paediatrics and Child Health. That copyright is preserved in this repository's notices and in the SPDX headers of ported files.

## What this is not

- **Not a new clinical method.** opengrowth devises no references and reinterprets none. References others publish can be added ([spec/standard.md](spec/standard.md) section 7.8).
- **Not a substitute for clinical judgement.** See [SAFETY.md](SAFETY.md).
- **Not the home of reference data provenance.** The upstream record of source datasets is [growth-references](https://github.com/rcpch/growth-references).
- **Not usable yet.**

## Reference data

The reference tables are in [`data/`](data/) and compiled into the binary, copied unmodified from the `rcpchgrowth` 4.6.5 package, where they are already publicly distributed. They are not covered by this repository's code licence and carry their own terms ([`LICENSES/`](LICENSES/)). Every table is embedded unless its owner asks for its removal, including those whose terms are not yet recorded (`R2` in [spec/roadmap.md](spec/roadmap.md)).

The UK90 reference, which UK-WHO depends on, was produced under the Medical Research Council and is now owned by UK Research and Innovation (UKRI). **It is included for pre-alpha research and development only. A licence application to UKRI is pending, and no opengrowth release will be used for patient care until it is granted.** If UKRI refuses, the tables come out of the binary and users who hold their own licence import them with `opengrowth data import`, a path that already works and is tested.

Every table records its own provenance and terms, enforced by a test. See [spec/standard.md](spec/standard.md) section 3 and `Q1` in [spec/queries.md](spec/queries.md).

## Quick start

```sh
cargo install --locked --path .   # installs `opengrowth` and its short alias `og`

opengrowth data status            # the reference tables are built in, and verified

opengrowth calculate --sex female --birth-date 2026-10-06 --observation-date 2026-11-17 \
  --gestation 27+3 --measurement weight --value 1.42
```

```text
UK-WHO: weight 1.42 kg, female, born 2026-10-06 at 27+3 weeks, measured 2026-11-17

                 decimal age        SDS   centile  table
chronological         0.1150    -8.0665      0.00  who-2006-infant
corrected            -0.1259    -1.6745      4.70  uk90-preterm
```

Add `--format json` for full-precision output with provenance: the engine version and the checksum of every table read.

## Playtesting

A hands-on tour of what works today, for maintainers and reviewers. Allow about 20 minutes. Everything here runs locally and touches nothing outside `~/.local/share/opengrowth` and `~/.cache/opengrowth`.

### 1. Install

You need [`rustup`](https://rustup.rs/), which installs the Rust version pinned in `rust-toolchain.toml` the first time you build, [`uv`](https://docs.astral.sh/uv/) (for the Python reference engine and its tables), and optionally `jq`.

```sh
git clone git@github.com:ohtf/opengrowth.git && cd opengrowth
cargo install --locked --path .
opengrowth            # bare invocation lists the commands
og                    # the same program, shorter to type
og completions install   # tab completion for both names; detects your shell, or pass --shell
```

### 2. Look at the reference tables

The tables are built in (see [Reference data](#reference-data)), but you can also import your own copy, which then takes precedence:

```sh
opengrowth data status      # every table "embedded" and in use
opengrowth data manifest    # where each table comes from, and its terms
opengrowth data import data # import the repository's own copy
opengrowth data status      # now "imported", verified against the same checksums
```

Run the import a second time: every table should say "already present". For step 5 you also need the Python engine: `s/reference-python` installs `rcpchgrowth` 4.6.5 into its own environment under `~/.cache/opengrowth`.

### 3. Calculations to try

Each of these exercises a different part of UK-WHO. Look at the `table` column as well as the numbers.

| Case | Command | What to look for |
| --- | --- | --- |
| Term 7-year-old | `opengrowth calculate --sex male --birth-date 2019-03-04 --observation-date 2026-11-17 --gestation 40+0 --measurement height --value 121.5` | UK90 child table; chronological and corrected ages agree |
| Extremely preterm, 6 weeks old | `opengrowth calculate --sex female --birth-date 2026-10-06 --observation-date 2026-11-17 --gestation 27+3 --measurement weight --value 1.42` | Corrected age is negative and uses UK90 preterm; chronological age uses WHO and gives a startling SDS, which is why correction matters |
| Second birthday | `opengrowth calculate --sex male --birth-date 2024-11-17 --observation-date 2026-11-17 --gestation 40+0 --measurement height --value 87.1` | 730 days is 1.9986 years, so still the WHO infant (lying) table. Try a day later |
| Fourth birthday | `opengrowth calculate --sex female --birth-date 2022-11-17 --observation-date 2026-11-17 --gestation 40+0 --measurement weight --value 16.1` | Exactly 4.0 switches from WHO to UK90 |
| Born post-term at 42+3 | `opengrowth calculate --sex female --birth-date 2025-06-01 --observation-date 2026-06-01 --gestation 42+3 --measurement ofc --value 45.2` | Corrected age is *older* than chronological |
| BMI at 5 days | `opengrowth calculate --sex female --birth-date 2026-11-12 --observation-date 2026-11-17 --gestation 39+1 --measurement bmi --value 12.9` | No result, with the upstream message: UK-WHO has no BMI below 2 weeks. Exit code 1 |
| Head circumference at 17.5 years, girl | `opengrowth calculate --sex female --birth-date 2009-05-01 --observation-date 2026-11-17 --gestation 40+0 --measurement ofc --value 55` | No result: girls' OFC data stops at 17 |

Add `--format json` to any of them for full precision and provenance, for example:

```sh
opengrowth --format json calculate --sex male --birth-date 2019-03-04 --observation-date 2026-11-17 \
  --gestation 40+0 --measurement height --value 121.5 | jq '{sds: .corrected.sds, tables: .provenance.reference_data}'
```

### 4. Safety behaviours to try

- **Required inputs.** Leave out `--gestation` or `--sex`: the CLI refuses rather than assuming term or a sex.
- **Implausible gestation.** `--gestation 21+0` or `45+0` is rejected; the accepted range is 22+0 to 44+6.
- **Tampered table.** Copy the tables, change one digit, and point opengrowth at the copy (GNU `sed`; on macOS write `sed -i ''`):

  ```sh
  W=$(mktemp -d) && cp -r data/. "$W/"
  sed -i '0,/"M": 102.49/s//"M": 102.50/' "$W/uk90_child.json"
  opengrowth --data-dir "$W" data status    # uk90-child: checksum mismatch
  opengrowth --data-dir "$W" calculate --sex male --birth-date 2019-03-04 \
    --observation-date 2026-11-17 --gestation 40+0 --measurement height --value 121.5
  ```

  The calculation refuses to run at all. That includes calculations that would not have needed the tampered table, which is deliberately conservative. Tell us whether you think it is right.
- **Missing tables.** Build without the tables (`cargo run --no-default-features --features cli -- calculate ...`): the calculation reports which table is missing and how to import it. It never substitutes another reference.

### 5. Check it against the Python engine yourself

Any case from step 3 can be checked against `rcpchgrowth` directly:

```sh
~/.cache/opengrowth/reference-python/bin/python -c "
from datetime import date
from rcpchgrowth import Measurement
m = Measurement(sex='female', birth_date=date(2026, 10, 6), observation_date=date(2026, 11, 17),
                gestation_weeks=27, gestation_days=3, measurement_method='weight',
                observation_value=1.42, reference='uk-who').measurement['measurement_calculated_values']
print(repr(m['chronological_sds']), repr(m['corrected_sds']))"
```

The SDS should match opengrowth's `--format json` output to every digit. Centiles agree to about 15 significant figures but not always to the last digit; see [conformance/README.md](conformance/README.md).

### 6. The HTTP API

```sh
opengrowth serve                       # http://127.0.0.1:8000; --bind to change, AUTHORIZATION_KEY to require a token
curl -s -X POST http://127.0.0.1:8000/uk-who/calculation -H 'content-type: application/json' -d '{
  "birth_date": "2020-04-12", "observation_date": "2028-06-12", "observation_value": 115,
  "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height"}' | jq .
```

Things to try:

- Point an existing Digital Growth Charts client or the React chart component at it. It should not notice the difference, apart from `provenance`, which names opengrowth and lists the checksum of each table it read.
- Send something wrong: a missing field, `"sex": "Male"`, `"gestation_weeks": 50`, a measurement of 300 cm, malformed JSON, a body labelled `text/plain`. The 422 bodies should be exactly the upstream ones.
- `POST /uk-who/bulk-calculation` with up to 200 observations; a bad observation becomes an inline error and the rest still calculate.
- `GET /uk-who/calculation` (405) and `POST /uk-who/calculation/` (307), answered as upstream answers them.
- Run a build without tables (`cargo run --no-default-features --features cli -- serve`): calculations return 503 naming the missing table, never an answer from another reference.

### 7. Run the evidence

```sh
s/conformance        # R vectors, Python goldens, SciPy, CPython, and 5,286 FastAPI responses replayed byte for byte
s/bench-round0       # in-process speed race: Rust engine against Python engine
s/reference-server   # once: the pinned FastAPI server, for the next command
s/bench --quick      # HTTP race against the real FastAPI server; report in .private/bench/
```

### What we would like to hear

- Any calculation where opengrowth and `rcpchgrowth` disagree. That would be a defect in one of them, so include the exact command.
- Anything confusing in the output, the error messages, or the `data` commands.
- Whether the text output is what you would want to see at a terminal, and whether the JSON is what an agent or script needs.
- Any HTTP response that differs from the RCPCH API other than in `provenance`. Include the request body.

Open an issue. For anything that could affect a clinical result, label it `hazard`, as described in [SAFETY.md](SAFETY.md).

### Clean up

```sh
cargo uninstall opengrowth
rm -r ~/.local/share/opengrowth ~/.cache/opengrowth
```

## Development

```sh
s/test          # unit and SciPy/CPython conformance tests
s/lint          # rustfmt and clippy, as CI runs them
s/conformance   # the conformance suites with their reports (cargo test also runs them)
```

See [s/README.md](s/README.md) and [conformance/README.md](conformance/README.md), which records the current agreement figures.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md), [agent-instructions.md](agent-instructions.md), and [SAFETY.md](SAFETY.md) first. Maintainers are listed in [MAINTAINERS.md](MAINTAINERS.md); we are looking for a second maintainer and an independent clinical reviewer. Do not change clinical behaviour, reference data, or the API contract without the evidence those documents require.

Sign off every commit under the [Developer Certificate of Origin](https://openhealthtechnology.org/community/dco/) (`git commit -s`). Report security issues as described in [SECURITY.md](SECURITY.md).

## Licence

Code is licensed under [AGPL-3.0-or-later](LICENSE), inherited from the Python engine this project ports. Written content is CC-BY-SA-4.0.

Copyright Open Health Technology Foundation contributors. Portions derived from `rcpchgrowth` are copyright Royal College of Paediatrics and Child Health.

Reference datasets carry their own separate terms, which take precedence.
