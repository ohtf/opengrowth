<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Integrating opengrowth

opengrowth's HTTP API takes the same requests as the RCPCH Digital Growth Charts API and returns the same responses, apart from the [listed differences](../differences.md). It is a REST API over JSON, kept deliberately simple: one request calculates one measurement, or up to 200 for one child. A system already built for the RCPCH API can call an opengrowth server instead by changing its address and its token ([Moving from the RCPCH API](migrating.md)).

!!! warning "Not for clinical use yet"

    opengrowth is pre-alpha software. Build and test integrations with it, but do not use it for patient care until it can be used clinically ([Safety](../safety.md)).

!!! danger "Plan to keep the provenance"

    Before building an integration, make sure your data model can store the complete `provenance` object with every calculated result. It names the reference, the engine and server versions and the commits they were built from, and every table read, and it is how affected results would be found if a calculation were ever found to be wrong. Read [Persisting results](persisting-results.md) before you design how to store responses.

## Run a server

`opengrowth serve` runs the API. [Getting started](../getting-started.md) shows how to install opengrowth.

```sh
export AUTHORIZATION_KEY="..."    # from your secret store
opengrowth serve --bind 127.0.0.1:8000
```

| Option | What it does |
| --- | --- |
| `--bind` | The address and port to listen on: `127.0.0.1:8000` unless set, or `OPENGROWTH_BIND`. |
| `--authorization-key` | Requires `Authorization: Bearer <key>` on every request except `/`. Also read from `AUTHORIZATION_KEY`, the RCPCH server's variable. Without a key, anyone who can reach the server can call it. |
| `--workers` | The number of worker threads: one per CPU unless set. |
| `--cors-allow-origin` | The websites whose pages may call the API from a browser: `*`, the default, for any, or exact origins separated by commas. Also read from `OPENGROWTH_CORS_ALLOW_ORIGINS`. |
| `--no-cors` | Sends no CORS headers, for a gateway that adds its own. |
| `--data-dir` | A directory of reference tables to use instead of those imported with `opengrowth data import`. Also read from `OPENGROWTH_REFERENCE_DATA_DIR`. |

The tables are compiled into the program, and each is checked against its recorded checksum when it is first read. `opengrowth data status` shows which are in use.

`GET /` is a health check that never needs the token. It answers with the server's name and version, and the commit it was built from in the `x-git-revision` header.

!!! danger "The key is a secret"

    Anyone who has the key can call the server as if they were you. Keep it out of source code and version control: give it to the server and its clients through environment variables or a secret store. If a key is exposed, replace it at once, on the server and in every client.

## Build it so that results name their code

Each result's provenance includes the commit the engine and server were built from. A build records it only when `OPENGROWTH_COMMIT` holds the full 40-character commit hash at compile time; otherwise it reports `unknown`, and stored results could not be traced to their exact code. Build what you deploy from a checkout:

```sh
OPENGROWTH_COMMIT="$(git rev-parse HEAD)" cargo build --release --locked
```

## In this section

- [Making calls](making-calls.md): requests, the token, and the optional parameters.
- [Persisting results](persisting-results.md): storing results with their provenance.
- [Client specification](client-specification.md): how a system must draw growth charts.
- [Chart component](chart-component.md): RCPCH's open-source chart component.
- [SNOMED CT codes](snomed-codes.md): coding stored results.
- [Security](security.md): running a server safely.
- [Research](research.md): calculating many measurements.
- [Moving from the RCPCH API](migrating.md): what changes.
- [Questions from integrators](questions.md) and [support](support.md).

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Getting Started with Digital Growth Charts](https://growth.rcpch.ac.uk/integrator/getting-started/) and [dGC API Server](https://growth.rcpch.ac.uk/products/api-server/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for a server you run yourself, in place of RCPCH's subscription service.
