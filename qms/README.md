<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Quality manual

The quality management system (QMS) for opengrowth as software as a medical device. It is kept in this repository and run through Git and GitHub, so the record of what was designed, reviewed, tested, and released is the repository itself.

> **Status: draft, not in force.** Nothing here has been approved, audited, or used to place a device on the market. opengrowth remains **not for clinical use** until the QMS is in force, the technical documentation is complete, and a declaration of conformity has been made (`R40` to `R45`).

Structure adapted, with attribution, from the RCPCH Digital Growth Charts quality manual (`digital-growth-charts-documentation`, `docs/safety/qms.md`, CC BY-SA 4.0), which applies the same Git-based approach to the upstream device.

## Scope

- **In scope:** opengrowth releases published by the manufacturer: the `opengrowth` binary (CLI, HTTP API, MCP server), the container images, the Python bindings, the WebAssembly package, and any OHTF-operated hosted API. See [intended-purpose.md](intended-purpose.md).
- **Out of scope:** forks, modified builds, and products that embed opengrowth. Under the AGPL anyone may build and deploy their own copy; a party that does so for clinical use is the manufacturer of what it deploys, or a deployer with DCB0160 duties, and this QMS does not cover it.
- **Manufacturer:** Open Health Technology CIC. PRRC: Dr Marcus Baw. See [regulatory-strategy.md](regulatory-strategy.md).

## Regulatory basis

See [regulatory-strategy.md](regulatory-strategy.md). In brief: opengrowth is an anthropometric calculation tool, Class I, placed on the Great Britain market first. The QMS is built to ISO 13485:2016 from the start, so that an EU route or a future reclassification does not need a second system.

| Standard | Application |
| --- | --- |
| ISO 13485:2016 | This quality management system |
| ISO 14971:2019 | Risk management, through the clinical safety process in [SAFETY.md](../SAFETY.md) (`R21`) |
| IEC 62304:2006+A1:2015 | Software life cycle; software safety class B ([regulatory-strategy.md](regulatory-strategy.md)) |
| IEC 82304-1:2016 | Health software product safety and labelling |
| IEC 62366-1:2015 | Usability engineering, for the parts of the device a person reads (CLI text, chart output, error messages) |
| DCB0129, DCB0160 | NHS clinical risk management for manufacturers and deployers |

Clause numbers are referenced but no text from the standards is reproduced; they are copyright works and must be bought.

## Roles

| Role | Responsibility | Holder |
| --- | --- | --- |
| Top management | Quality policy, resources, management review | Directors of Open Health Technology CIC |
| Person responsible for regulatory compliance (PRRC) | Technical documentation, declaration of conformity, post-market surveillance and vigilance | Dr Marcus Baw |
| Clinical Safety Officer | ISO 14971 and DCB0129 risk management, hazard log, safety case | Dr Marcus Baw |
| Technical lead | IEC 62304 life cycle, release, security | Dr Marcus Baw; a second maintainer is sought |
| Independent clinical reviewer | Review of clinically significant changes and of clinical evaluation | **Vacant** ([MAINTAINERS.md](../MAINTAINERS.md)) |
| QMS administrator | Document control, audit scheduling, training records | Dr Marcus Baw |

One person currently holds almost every role. ISO 13485 requires design reviews to include someone independent of the work under review, and internal audits to be done by someone who did not do the work. **The QMS cannot come into force until at least one other competent person can review and audit** (`R44`).

## Quality policy

opengrowth will calculate children's growth parameters correctly, transparently, and reproducibly. Every calculation must be traceable to a published method, an independent test vector, and a released, signed version of the software. We develop openly, record every decision where the next reader will find it, and treat a clinical result we cannot explain as a defect.

### Quality objectives (proposed)

| Objective | Measure |
| --- | --- |
| Clinical accuracy | 100% of independent vectors within declared tolerance at every release |
| Compatibility | Enumerated, reviewed differences only from the upstream API (`KNOWN_DIFFERENCES`) |
| Safety | Every hazard has a control and evidence before release; no open high-risk hazard at release |
| Responsiveness | Safety-relevant reports acknowledged within 3 working days |
| Supply chain | Zero unreviewed advisories in `cargo deny` at release |

## How Git implements the QMS

| QMS need | Mechanism |
| --- | --- |
| Document control (4.2.4) | Controlled documents are Markdown in this repository. A document's approved version is the version on `main` at a signed release tag; its history is `git log`. Approval is a reviewed, merged pull request. |
| Record control (4.2.5) | Records are GitHub issues (hazards, CAPA, complaints, serious incidents, design inputs, management reviews, audits) using the templates in `.github/ISSUE_TEMPLATE/` and the labels in `.github/labels.yml`, Git history, CI run logs, and release artefacts with attestations. Retention: the life of the device plus the period the regulations require, by keeping the repository and mirroring it. |
| Design inputs and outputs (7.3.3, 7.3.4) | Inputs: `spec/`, the intended purpose, the upstream API contract, and published methods. Outputs: source, `conformance/`, documentation. |
| Design verification (7.3.6) | CI, `s/conformance` against independent vectors and upstream goldens, and the HTTP replay. |
| Design validation (7.3.7) | Clinical evaluation by equivalence to the upstream device plus the conformance evidence (`R42`). |
| Design changes (7.3.9) | Pull requests with review, classified as clinical or non-clinical per [spec/versioning.md](../spec/versioning.md). |
| Traceability | Requirement codes (`R12`), hazard issue numbers, and test names cross-referenced in commits and pull requests. |

## ISO 13485:2016 clause map

| Clause | Where it is met | Status |
| --- | --- | --- |
| 4.1 General requirements, outsourced processes | This manual; [suppliers](procedures.md#suppliers-and-soup) | Draft |
| 4.1.6 Validation of QMS software | [Tool validation](procedures.md#tool-validation), including CI and AI coding agents | Not started |
| 4.2.2 Quality manual | This page | Draft |
| 4.2.3 Medical device file | [technical-file.md](technical-file.md) | Skeleton |
| 4.2.4 Control of documents | [How Git implements the QMS](#how-git-implements-the-qms); `.github/CODEOWNERS` | Draft; branch protection not yet enforcing |
| 4.2.5 Control of records | [How Git implements the QMS](#how-git-implements-the-qms); `.github/ISSUE_TEMPLATE/`; `.github/labels.yml` | Templates in place |
| 5 Management responsibility | [Roles](#roles), [Quality policy](#quality-policy), [management review](procedures.md#management-review) | Draft; top management not confirmed |
| 6.2 Human resources | [Competence and training](procedures.md#competence-and-training) | Not started |
| 6.3, 6.4 Infrastructure, work environment | GitHub, CI runners, signing; [suppliers](procedures.md#suppliers-and-soup) | Not started |
| 7.1 Planning of product realisation | [spec/roadmap.md](../spec/roadmap.md), [software life cycle](procedures.md#software-life-cycle) | Partial |
| 7.2 Customer-related processes | [intended-purpose.md](intended-purpose.md), API contract, [spec/versioning.md](../spec/versioning.md) | Draft |
| 7.3 Design and development | [Software life cycle](procedures.md#software-life-cycle) | Practised, not written as a procedure |
| 7.4 Purchasing | [Suppliers and SOUP](procedures.md#suppliers-and-soup), [soup.md](soup.md), `deny.toml` | SOUP register generated |
| 7.5 Production and service provision | [Release](procedures.md#release) (`R19`, `R37`) | Not started |
| 7.6 Monitoring and measuring equipment | Not applicable to software except as tool validation | |
| 8.2.1 Feedback | [Post-market surveillance](procedures.md#post-market-surveillance-and-vigilance) | Not started |
| 8.2.2 Complaint handling | [Complaints](procedures.md#complaints) | Not started |
| 8.2.3 Reporting to regulatory authorities | [Vigilance](procedures.md#post-market-surveillance-and-vigilance) | Not started |
| 8.2.4 Internal audit | [Internal audit](procedures.md#internal-audit) | Not started |
| 8.2.6 Monitoring of product | CI gates, `s/conformance` | Practised |
| 8.3 Nonconforming product | [CAPA and nonconformity](procedures.md#capa-and-nonconformity) | Not started |
| 8.4 Analysis of data | [Management review](procedures.md#management-review) | Not started |
| 8.5 Improvement, CAPA | [CAPA and nonconformity](procedures.md#capa-and-nonconformity) | Not started |

## Scheduled activities

| Activity | Frequency | Record |
| --- | --- | --- |
| Management review | At least annually, and after a significant quality event | Management review issue |
| Internal audit | At least annually, by someone independent of the area audited | Audit issue and finding issues |
| SOUP and supplier review | Every release, and at least annually | `soup.md` diff and review in the release pull request |
| Hazard log review | Every release, and on any new or changed hazard | Hazard issues; safety case |
| Post-market surveillance review | Continuous; appraised at each management review | PMS report |
| CAPA review | Continuous; status at each management review | CAPA issues |
