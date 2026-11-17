<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Technical documentation

The medical device file for opengrowth (ISO 13485 clause 4.2.3), for a Class I declaration of conformity in Great Britain. Each entry points to where the content lives; most of it is already in the repository as part of ordinary development.

> **Status: skeleton.** Entries marked "not started" must exist before the declaration of conformity (`R42`).

| Section | Content | Where | Status |
| --- | --- | --- | --- |
| Device description | What opengrowth is, its surfaces, versions, and variants | [README.md](../README.md), [spec/standard.md](../spec/standard.md) section 7 | Partial |
| Intended purpose | The canonical statement | [intended-purpose.md](intended-purpose.md) | Draft |
| Classification | Class I rationale for each market | [regulatory-strategy.md](regulatory-strategy.md) | Draft |
| Manufacturer | Open Health Technology CIC; PRRC | [regulatory-strategy.md](regulatory-strategy.md) | Decided |
| Labelling and instructions for use | Product identification, version, intended purpose, warnings, and how to use each surface; shown in `--help`, the API description, and the documentation site | `R30` | Not started |
| Unique device identifier | UK UDI assignment when required | | Not started |
| Design and manufacturing | Architecture, build, and release process | [spec/standard.md](../spec/standard.md), [procedures.md](procedures.md#release) | Partial |
| Essential requirements checklist | UK MDR 2002 Annex I, each requirement with the evidence that meets it | | Not started |
| Standards applied | | [regulatory-strategy.md](regulatory-strategy.md#standards) | Draft |
| Risk management file | Plan, hazard log, safety case, benefit-risk | [SAFETY.md](../SAFETY.md), `R21`, `R27` | Partial |
| Software life cycle | IEC 62304 class B documentation | [procedures.md](procedures.md#software-life-cycle) | Draft |
| SOUP | Third-party software in the release | [soup.md](soup.md) | Generated |
| Verification | Conformance results against independent vectors and the upstream device | [conformance/README.md](../conformance/README.md) | Partial (UK-WHO) |
| Clinical evaluation | Equivalence to the RCPCH Digital Growth Charts API (Class I), with the byte-identical conformance evidence and the independent R vectors; literature on the growth references and the LMS method | | Not started |
| Usability | IEC 62366-1 summative evaluation of what a person reads: CLI text output, error messages, chart output | | Not started |
| Cybersecurity | Threat model, secure development, vulnerability handling | [SECURITY.md](../SECURITY.md), `R37` | Partial |
| Data protection | Privacy statement and data flows | `R39` | Not started |
| Post-market surveillance plan | | [procedures.md](procedures.md#post-market-surveillance-and-vigilance) | Draft |
| Declaration of conformity | Signed by the manufacturer for a named version | | Not started |
| MHRA registration | Device registration, GMDN code (65712, as for the upstream device, to confirm) | | Not started |
