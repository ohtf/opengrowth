# Versioning, compatibility, and deprecation

What integrators can rely on, and how it changes. This is normative: a change that breaks it is a defect, or needs an explicit exception recorded here.

## 1. Version numbers

One version for the whole workspace, single-sourced in `Cargo.toml` and released with `s/version++` (house-style `distribution.md`). It follows [Semantic Versioning](https://semver.org).

The version is reported in every result's `provenance.calculation_engine.version` and `provenance.api_server.version`, so a result can always be traced to the release that produced it.

Before 1.0.0, a minor release may break the Rust library API, with migration notes. The HTTP API contract in section 2 holds from the first release regardless, because it is inherited from an API already in clinical use.

## 2. What is a contract

| Surface | Contract | Not a contract |
| --- | --- | --- |
| HTTP API | Paths, methods, request fields and their validation, response fields, their names, types, nullability, order, and meaning, status codes, and error shapes. The target is byte identity with the upstream RCPCH API outside `provenance` (`standard.md` section 4.2). | Transport headers other than `content-type`, `allow`, and `location`; timing. |
| CLI | Command names, flags, exit codes, and `--format json` output. | Text output, which is for people and may change in any release. |
| MCP | Tool names and input and output schemas. | Tool descriptions, which may be reworded. |
| Rust library | The public API of `opengrowth-core`, from 1.0.0. | Everything in the CLI, server, data, and conformance crates. |
| Python bindings | The public API, from their first stable release (`R18`). | |

Additive changes (a new optional request field, a new response field, a new endpoint, command, or tool) are allowed in a minor release. For the HTTP API, a new response field is still a difference from upstream, so it goes in the migration notes; the only one so far is `provenance.reference_data`.

## 3. Clinical changes

A change to any calculated value, clinical string, centile band, reference selection, or age-correction rule is a **clinical change**, even if it fixes a bug and even if no field changes shape.

A clinical change:

- goes through the clinical safety process in [SAFETY.md](../SAFETY.md), with a hazard log entry and the evidence for it;
- is released in at least a minor version (a major version after 1.0.0 if integrators would need to act);
- is listed in its own **Clinical changes** section at the top of the changelog entry, with before and after values for a worked example;
- is announced to known integrators before release where it could change a value they show clinicians.

A change that keeps every output identical, such as a performance improvement proven against the conformance suite, is not a clinical change.

## 4. Deprecation

Nothing in a contract is removed without notice:

- **Announce** the deprecation in the changelog and the documentation, naming the replacement and the removal date.
- **Signal it in responses.** Deprecated HTTP endpoints send `Deprecation` ([RFC 9745](https://www.rfc-editor.org/rfc/rfc9745)) and `Sunset` ([RFC 8594](https://www.rfc-editor.org/rfc/rfc8594)) headers and a `Link` to the migration notes. Deprecated CLI flags print a warning on stderr and keep working.
- **Wait.** At least 6 months for a field or flag, and at least 12 months for an endpoint, command, or tool, before removal in a major release.
- **Never shorten the wait** for a surface known to be in clinical use, except to remove something unsafe, which follows the clinical safety process instead.

## 5. Reference data versions

Reference tables are identified by checksum, not by name. A corrected or updated table is a new manifest entry with its own checksum; the old entry stays loadable for at least one minor release with a warning, so a deployment can upgrade the binary and the data separately. Every result lists the tables it read in `provenance.reference_data`, so a change of data is always visible in the output.

## 6. Differences from the upstream API

Every intentional difference from the RCPCH API is enumerated in three places, kept in step: `standard.md` section 6.3, the `KNOWN_DIFFERENCES` list in `crates/opengrowth-conformance/tests/http_uk_who.rs`, and the migration notes in the documentation (`R30`). A difference that appears in only one of them is a defect.
