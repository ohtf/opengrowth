<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Procedures

> **Status: draft, not in force.** Each procedure says what triggers it, who does it, and what record proves it happened. Adapted, with attribution, from the RCPCH Digital Growth Charts QMS (CC BY-SA 4.0). Where a regulatory deadline is quoted, confirm it against the current legislation before the QMS comes into force.

Issue templates are in `.github/ISSUE_TEMPLATE/`; labels are in `.github/labels.yml` and are created by the `create-qms-labels` workflow.

## Document control

- Controlled documents: everything under `qms/`, `SAFETY.md`, `spec/`, `NOTICE.md`, and the source and conformance evidence of the released crates.
- A change is a pull request. Regulated paths need approval from a code owner (`.github/CODEOWNERS`). Commit messages give the reason for the change.
- The approved version of a document is its content at a signed release tag. History is the Git log; documents carry no separate revision table.
- History on `main` is never rewritten after merge. Force-pushes and branch deletion are blocked by branch protection.
- External documents (standards, regulations, guidance, papers) are referenced by title, version, and URL or DOI, never copied in.

## Software life cycle

IEC 62304, software safety class B.

| Activity | How |
| --- | --- |
| Planning | [spec/roadmap.md](../spec/roadmap.md) items with stable codes |
| Requirements (design inputs) | `spec/`, the intended purpose, and the upstream API contract; new significant work starts with a `design-input` issue |
| Architecture | `spec/standard.md` section 7 |
| Detailed design and implementation | Source, reviewed in pull requests |
| Unit and integration verification | `cargo test` in CI; `s/conformance` against independent vectors and upstream goldens; the HTTP replay |
| System verification | The conformance suite run on the release build |
| Release | [Release](#release) |
| Problem resolution | Issues, with `hazard`, `capa`, or `non-conformity` labels as appropriate |
| SOUP | [Suppliers and SOUP](#suppliers-and-soup) |

Design reviews happen before work on a `design-input` starts and before each release, recorded as review comments on the issue or pull request. A design review needs at least one competent reviewer other than the author.

Changes are classified as clinical or non-clinical under [spec/versioning.md](../spec/versioning.md). A clinical change also needs a regulatory impact assessment against [intended-purpose.md](intended-purpose.md) and [regulatory-strategy.md](regulatory-strategy.md): does it stay within the intended purpose, and does it add any feature listed under "Features that would change the class"?

## Risk management

ISO 14971 through DCB0129, as set out in [SAFETY.md](../SAFETY.md): hazards are issues from the `hazard` template, scored with the severity, likelihood, and risk-level labels. The hazard log is reviewed at every release. No release proceeds with a risk at level 3 or above without documented PRRC justification in the safety case.

## Suppliers and SOUP

- **SOUP** (software of unknown provenance, IEC 62304): every third-party crate compiled into a released artefact. [soup.md](soup.md) lists them, generated from `Cargo.lock` by `s/soup`. CI fails if it is out of date. `cargo deny` checks licences, advisories, and sources on every push.
- Each release pull request reviews the SOUP diff. A new SOUP item needs a reason and a check that it is maintained (house-style `dependencies.md`).
- **Suppliers:** GitHub (repository, CI, releases, issues: the QMS records), crates.io, the reference-data owners (UKRI for UK90, WHO), and any hosting provider for an OHTF-operated API. Each is listed with its criticality and reviewed annually at management review.

## Tool validation

ISO 13485 clause 4.1.6 requires software used in the QMS and in production to be validated for its intended use, proportionately to risk.

| Tool | Use | Validation |
| --- | --- | --- |
| Rust toolchain | Builds the device | Stable channel pinned in `rust-toolchain.toml`; the conformance suite run on the built artefact is the validation |
| GitHub Actions | Verification and release | Workflows are controlled documents, actions pinned to commit SHAs, checked by `zizmor` |
| `cargo deny`, `cargo audit`, `reuse` | Supplier and licence control | Versions pinned in CI |
| Golden generators (`conformance/generators/`) | Produce verification evidence | Run against pinned upstream versions recorded in each golden |
| **AI coding agents** | Write code, tests, and documents | See below |

**AI coding agents.** Much of this repository is written with AI agents. They are treated as a development tool whose output is a draft by an unqualified author: nothing an agent writes is trusted until a competent person has reviewed it, and agent-written tests are not evidence for clinical behaviour on their own (`agent-instructions.md`, Assurance). The controls are the same as for any author: pull-request review, CI, and the conformance suite against independent vectors. The pull request records that an agent was used. Agent configuration and instructions (`agent-instructions.md`) are controlled documents.

## Release

1. Release pull request from `s/version++`: changelog with a clinical changes section, SOUP diff, hazard log review, and safety case update.
2. Verification: CI green; `s/conformance` run on the release build with every independent vector and golden passing, the log attached to the pull request.
3. PRRC approval of the release pull request.
4. Tag, build, and publish (`R19`), with SBOM, build attestations, and checksums (`R37`).
5. Release record: the tag, the release page, and the attached evidence.

## Complaints

Any expression of dissatisfaction, or report of a possible safety or quality problem, from any source. Opened as a `complaint` issue within 2 working days of receipt, assessed for reportability as a serious incident, investigated, and closed with the outcome. Reports containing personal or patient data are handled privately (`SECURITY.md`) and summarised in the issue without the data.

## Post-market surveillance and vigilance

- **Surveillance:** complaints, issues, security reports, upstream `rcpchgrowth` and server changes, published literature on the growth references, and divergence reports from integrators. Reviewed continuously, and appraised at each management review.
- **Report:** a post-market surveillance report for Class I in Great Britain, at the interval the GB post-market surveillance requirements set (confirm against SI 2024/1368), and at least at each management review.
- **Vigilance:** a serious incident is opened as a `serious-incident` issue, and the PRRC reports it to the MHRA within the GB deadlines: 2 days for a serious public health threat, 10 days for death or unanticipated serious deterioration in health, and 15 days for any other serious incident (confirm against SI 2024/1368). Field safety corrective actions and notices follow the same process.

## CAPA and nonconformity

A `non-conformity` issue records a failure to meet a requirement of the QMS, a standard, or the product specification. A `capa` issue is raised for a systemic cause: a confirmed complaint, an audit finding, a serious incident, or an adverse trend. It records the root cause, the corrective and preventive actions, an owner and due date, and how effectiveness will be verified. CAPAs are reviewed at management review.

## Internal audit

At least annually, using the `internal-audit` template, by an auditor independent of the area audited. Each finding is an `audit-finding` issue; nonconformities lead to CAPAs. The audit report is committed to `qms/records/audits/`.

## Management review

At least annually, and after any significant quality event, by the directors of Open Health Technology CIC with the PRRC, using the `management-review` template. Inputs: quality policy and objectives, audit results, CAPA status, complaints, post-market surveillance, regulatory changes, resources, and actions from the previous review. Minutes go to `qms/records/management-review/`.

## Competence and training

Each person with a regulated role has a record in `qms/records/training/`: role, qualifications, relevant experience, and training completed with dates. Nobody acts alone on a regulated task until their record shows they are competent for it.
