<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Security

How to run an opengrowth server safely, and how opengrowth itself is built and checked. An organisation that deploys opengrowth is responsible for its own deployment's security, as for its clinical safety ([Safety](../safety.md#if-you-deploy-opengrowth)).

## Running a server

- **Require a token.** Start the server with `AUTHORIZATION_KEY` set, or `--authorization-key`. Every request except `/` must then send `Authorization: Bearer <key>`, or gets `403 Forbidden`. The key is compared in constant time, so the time a refusal takes does not reveal how much of a guessed key was right. Without a key, anyone who can reach the server can use it.
- **Keep the key secret.** Give it to the server and its clients through environment variables or a secret store, never in source code or version control, and replace it at once if it is exposed.
- **Encrypt the connection.** opengrowth serves plain HTTP. Put it behind a reverse proxy or gateway that provides TLS, and let only the proxy reach it. By default it listens on `127.0.0.1:8000`, which accepts connections from the same machine only.
- **Limit which websites can call it from a browser.** By default, pages on any website may call the API from a browser, though without credentials such as cookies. Name the sites allowed with `--cors-allow-origin`, or, behind a gateway that sets its own CORS headers, turn opengrowth's off with `--no-cors`. Browsers can call the API while a key is set.
- **Requests are bounded.** opengrowth reads a request body of at most 2 MiB, a bulk request of at most 200 observations, and a custom chart of at most 15 centiles or SDS values, so no request can make it hold an unbounded amount in memory.
- **It keeps nothing.** opengrowth stores no measurements or results, and logs no requests: it writes one line when it starts. A proxy or gateway in front of it may log requests, which hold dates of birth and measurements, so set its logging with that in mind.
- **The tables are checked.** Every reference table is checked against its recorded SHA-256 checksum before it is first used. A table that does not match is refused, and opengrowth never substitutes another; `opengrowth data import` checks imported tables in the same way. Each result names the tables it read ([Persisting results](persisting-results.md)).

## How opengrowth is built

opengrowth's code is public, at [github.com/ohtf/opengrowth](https://github.com/ohtf/opengrowth), and every change to it goes through the same checks:

- Every commit is signed off under the [Developer Certificate of Origin](https://openhealthtechnology.org/community/dco/), and CI refuses one that is not.
- CI runs on every change: formatting and lint checks, every test, including the conformance suites against the RCPCH server and `rcpchgrowth`, a check of every dependency's licence and of known security advisories against them (`cargo deny` and `cargo audit`), a security audit of the workflows themselves (`zizmor`), a check that every file states its licence (REUSE), and the documentation's own checks.
- Every GitHub Action is pinned to the full commit of the version reviewed, not to a tag that could be moved.
- Dependencies are admitted deliberately, the lockfile is committed, and every library compiled in is listed with its licence in the [SOUP register](https://github.com/ohtf/opengrowth/blob/main/qms/soup.md).

## Reporting a vulnerability

Report it privately, not in a public issue: through GitHub's private vulnerability reporting for the repository, or by email to <security@openhealthtechnology.org>. [SECURITY.md](https://github.com/ohtf/opengrowth/blob/main/SECURITY.md) says what to include. A problem that could change a calculated result is also a safety hazard: say so in the report.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Security](https://growth.rcpch.ac.uk/technical/security/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). RCPCH's page describes how RCPCH secures its own development and hosted service; this one is rewritten for a server you run yourself, and for how opengrowth is built.
