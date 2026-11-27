<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Porting the Digital Growth Charts documentation

The documentation site (`docs/`) adapts the RCPCH Digital Growth Charts documentation, which Dr Marcus Baw wrote with few exceptions (`R36`, split into `R50` to `R52`). This file records where it came from, the rules for adapting a page, and the decision for every upstream page.

## Source and terms

- **Repository:** [rcpch/digital-growth-charts-documentation](https://github.com/rcpch/digital-growth-charts-documentation), at commit `f00799aadf2a4eca39299d294ca0908609a13d22` (24 November 2026), published at [growth.rcpch.ac.uk](https://growth.rcpch.ac.uk).
- **Written content:** CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health, by the contributors its history records. Derived pages are CC BY-SA 4.0 too.
- **Site code:** MIT. None of it is copied.
- **History:** not imported. Upstream's commits carry no DCO sign-off, which CI requires of every commit here, so content is copied, with this commit recorded, a source note on every derived page, and every contributor named in `docs/about/acknowledgements.md`.

## Adapting a page

- **Attribution.** A derived page ends with a note naming the upstream page, linked, its licence and copyright, a link to the acknowledgements, and what changed. Its SPDX header credits RCPCH and the upstream contributors as well as OHTF contributors.
- **No RCPCH branding.** No logo, colour, theme, or image that carries RCPCH branding. RCPCH is named only where the text is about RCPCH's own software or service, or to credit it.
- **A self-hosted server.** Instructions for RCPCH's hosted service (subscription keys, the API gateway, prices, support routes) become instructions for a deployment's own opengrowth server, or are left out.
- **The intended purpose.** `s/check-wording` applies. Guidance on acting on a measurement, which would read as opengrowth interpreting it, is left out, and families are not addressed, as they are not intended users ([qms/intended-purpose.md](../qms/intended-purpose.md)).
- **Accuracy.** Statements about behaviour are checked against opengrowth, and where opengrowth differs from the upstream server the page says so.

## Inventory

32 pages are adapted, 2 replaced by generated pages, and 57 dropped, of 91.

| Upstream page | Decision | Destination | Reason |
| --- | --- | --- | --- |
| `_assets/_snippets/api-baseurl.md` | adapt | `integrating/making-calls.md` (R52) | The base address becomes the deployment's own server. |
| `_assets/_snippets/centile-terminology.md` | adapt | `background/how-it-works.md` (R51) | Defines centile terms the clinical pages use. |
| `_assets/_snippets/dgc-platform-comprises.md` | drop |  | Describes RCPCH's hosted platform. |
| `_assets/_snippets/docs-contributions.md` | adapt | `about/contributing.md` (R50) | How to contribute to the documentation. |
| `_assets/_snippets/htn-award.md` | drop |  | An award to RCPCH's project. |
| `_assets/_snippets/self-host-warning.md` | adapt | `safety.md` (R50) | RCPCH's case for its hosted service is dropped; what a deploying organisation is responsible for is kept. |
| `_utilities/page-template.md` | drop |  | An authoring template for that site. |
| `about/about.md` | adapt | `about/acknowledgements.md` (R50) | Only the project's history; the rest describes RCPCH's service. |
| `about/acknowledgements.md` | adapt | `about/acknowledgements.md` (R50) | Credits for the upstream project, with the documentation's contributors added. |
| `about/awards-press-blogs.md` | drop |  | News about RCPCH's project. |
| `about/team.md` | drop |  | RCPCH's team; the acknowledgements link to it. |
| `about/videos.md` | drop |  | Videos of RCPCH's service. |
| `about/whos-using-dgc.md` | drop |  | Users of RCPCH's service, not of opengrowth. |
| `clinician/chart-information-health-staff.md` | adapt | `background/charts.md` (R51) | How the charts are constructed and plotted; guidance on acting on a measurement is left out (intended purpose). |
| `clinician/date-age-calculations.md` | adapt | `background/ages.md` (R51) | Decimal, chronological, and corrected ages: the same calculations. |
| `clinician/faqs-for-clinicians.md` | adapt | `background/questions.md` (R51) | Questions about the calculations; those about RCPCH's service are left out. |
| `clinician/growth-chart-papers.md` | adapt | `background/papers.md` (R51) | The published methods and references. |
| `clinician/growth-references.md` | adapt | `background/growth-references.md` (R51) | The references opengrowth implements. |
| `clinician/how-the-api-works.md` | adapt | `background/how-it-works.md` (R51) | How a measurement becomes an SDS and centile: the same method. |
| `contact/contact.md` | drop |  | RCPCH's contacts; support is on GitHub (integrating/support.md). |
| `contact/downloads.md` | drop |  | RCPCH's downloadable files. |
| `developer/api-docker.md` | drop |  | Running the Python server. |
| `developer/api-python.md` | drop |  | Developing the Python server. |
| `developer/api-testing.md` | drop |  | Testing the Python server; opengrowth's evidence is its conformance suites. |
| `developer/contributing.md` | adapt | `about/contributing.md` (R50) | Contributing, for opengrowth's process: DCO, safety evidence. |
| `developer/faqs-for-developers.md` | adapt | `integrating/questions.md` (R52) | Where the answers apply to opengrowth. |
| `developer/five-repository-upgrade-runbook.md` | drop |  | RCPCH's release process across its repositories. |
| `developer/rcpchgrowth-cli.md` | replace | `cli/` (R56) | opengrowth's own command-line tool, generated from its help. |
| `developer/rcpchgrowth.md` | adapt | `background/how-it-works.md` (R51) | The method parts: reference selection, interpolation, and the calculations. |
| `developer/react-client.md` | drop |  | Developing RCPCH's demo client. |
| `developer/react-component.md` | drop |  | Developing RCPCH's chart component. |
| `developer/start-here.md` | drop |  | Orientation in RCPCH's repositories; about/contributing.md replaces it. |
| `developer/versioning.md` | drop |  | RCPCH's versioning; opengrowth's is spec/versioning.md. |
| `developer/who-reference-implementation.md` | adapt | `background/who.md` (R51) | How the WHO reference is implemented: the same method. |
| `developer/writing-documentation.md` | adapt | `about/contributing.md` (R50) | Writing this site's pages. |
| `index.md` | drop |  | Replaced by opengrowth's home page. |
| `integrator/api-reference.md` | replace | `api/reference.md` (R49) | opengrowth's generated OpenAPI document, in Swagger UI. |
| `integrator/chart-component-storybook.md` | drop |  | RCPCH's component gallery; the chart component page links to it. |
| `integrator/client-specification.md` | adapt | `integrating/client-specification.md` (R52) | What a client showing the results must do. |
| `integrator/faqs-for-integrators.md` | adapt | `integrating/questions.md` (R52) | Where the answers apply to a self-hosted server. |
| `integrator/getting-started.md` | adapt | `integrating/index.md` (R52) | Rewritten for a self-hosted server. |
| `integrator/making-api-calls.md` | adapt | `integrating/making-calls.md` (R52) | The same requests, to the deployment's own server with a bearer token. |
| `integrator/persisting-api-results.md` | adapt | `integrating/persisting-results.md` (R52) | Storing results and their provenance. |
| `integrator/snomed-codes.md` | adapt | `integrating/snomed-codes.md` (R52) | SNOMED CT codes for the measurements and results. |
| `integrator/support.md` | adapt | `integrating/support.md` (R52) | opengrowth's support routes: GitHub issues, and hazards. |
| `integrator/turner-down-syndrome.md` | adapt | `background/syndrome-references.md` (R51) | The syndrome-specific references; choosing one stays with the clinician. |
| `integrator/using-the-chart-component.md` | adapt | `integrating/chart-component.md` (R52) | RCPCH's chart component works with opengrowth unchanged. |
| `legal/accessibility-statement.md` | drop |  | Covers RCPCH's site. |
| `legal/data-protection.md` | drop |  | RCPCH as controller or processor does not transfer; R39 writes opengrowth's own. |
| `legal/disclaimer.md` | adapt | `about/licences.md` (R50) | A disclaimer for opengrowth, without RCPCH's warranty terms. |
| `legal/licensing-copyright.md` | adapt | `about/licences.md` (R50) | opengrowth's licences and the terms of its data. |
| `legal/privacy-notice.md` | drop |  | RCPCH's privacy notice; R39 writes opengrowth's own. |
| `parents/chart-information-families.md` | drop |  | Families are not intended users (qms/intended-purpose.md). |
| `parents/faqs-parents.md` | drop |  | Families are not intended users (qms/intended-purpose.md). |
| `products/api-server.md` | adapt | `integrating/index.md` (R52) | What the API does, for a self-hosted server. |
| `products/command-line-client(deprecated).md` | drop |  | Deprecated upstream. |
| `products/flask(deprecated).md` | drop |  | Deprecated upstream. |
| `products/flutter-app.md` | drop |  | An RCPCH product. |
| `products/get-digital-growth-charts.md` | drop |  | Subscribing to RCPCH's service. |
| `products/google-sheets-plugin.md` | drop |  | An RCPCH product. |
| `products/pricing.md` | drop |  | RCPCH's prices. |
| `products/products-overview.md` | drop |  | RCPCH's products. |
| `products/python-library.md` | drop |  | rcpchgrowth; opengrowth's Python bindings are R18. |
| `products/react-client.md` | drop |  | An RCPCH product. |
| `products/react-component.md` | adapt | `integrating/chart-component.md` (R52) | Only what an integrator needs to use the component with opengrowth. |
| `researchers/introduction.md` | adapt | `integrating/research.md` (R52) | Calculating in bulk for research, with opengrowth's tools. |
| `safety/csmf/clinical-evaluation.md` | drop |  | RCPCH's clinical safety file covers RCPCH's device, not opengrowth (SAFETY.md). |
| `safety/csmf/clinical-risk-mgmt-plan.md` | drop |  | As above. |
| `safety/csmf/clinical-risk-mgmt-system.md` | drop |  | As above. |
| `safety/csmf/clinical-safety-case-report.md` | drop |  | As above. |
| `safety/csmf/hazard-log.md` | drop |  | As above; opengrowth's hazard log is R27. |
| `safety/csmf/intended-purpose.md` | drop |  | opengrowth has its own, adapted with attribution (qms/intended-purpose.md). |
| `safety/csmf/license.md` | drop |  | RCPCH's licence terms for its service. |
| `safety/csmf/pms-plan.md` | drop |  | As for the clinical safety file. |
| `safety/csmf/software-lifecycle.md` | drop |  | As for the clinical safety file. |
| `safety/csmf/third-party-tools-safety-assmt.md` | drop |  | As for the clinical safety file. |
| `safety/download.md` | drop |  | Downloads of RCPCH's safety file. |
| `safety/dtac.md` | drop |  | RCPCH's DTAC submission; R39 covers what a deployer needs. |
| `safety/investigations/index.md` | drop |  | RCPCH's investigations; the differences page links to them where they affect behaviour (R53). |
| `safety/investigations/pooled-term-data-discrepancy-report.md` | drop |  | As above. |
| `safety/investigations/trisomy-21-reference-mismatch-report.md` | drop |  | As above. |
| `safety/medical-device-reg/doc-api.md` | drop |  | RCPCH's device registration. |
| `safety/medical-device-reg/essential-req.md` | drop |  | RCPCH's device registration. |
| `safety/medical-device-reg/mdr-technical-docs.md` | drop |  | RCPCH's device registration. |
| `safety/medical-device-reg/mhra.md` | drop |  | RCPCH's device registration. |
| `safety/overview.md` | drop |  | RCPCH's safety overview; opengrowth's is safety.md. |
| `safety/qms.md` | drop |  | RCPCH's QMS; opengrowth's is qms/. |
| `tags.md` | drop |  | A tag index; this site does not use tags. |
| `technical/data-security-protection-toolkit.md` | drop |  | RCPCH's DSPT submission. |
| `technical/security.md` | adapt | `integrating/security.md` (R52) | Securing a self-hosted server: the token, CORS, and request bodies. |
| `technical/status.md` | drop |  | RCPCH's service status. |
