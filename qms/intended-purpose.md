<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Intended purpose

> **Status: draft for approval by the PRRC.** Once approved, this is the canonical statement. Where any other document, label, help text, or API description summarises opengrowth's purpose, this one takes precedence, and the others must not claim more.

Structure adapted, with attribution, from the RCPCH Digital Growth Charts intended purpose (`docs/safety/csmf/intended-purpose.md`, CC BY-SA 4.0).

## Statement

opengrowth is an **anthropometric calculation tool**. It calculates the standard deviation score (SDS) and centile of a child's height or length, weight, head circumference, or body mass index against a published growth reference, chosen by the user, and describes the measurement's position relative to that reference's centile lines. It also calculates decimal and corrected ages, and the coordinates of the reference's centile lines for drawing a growth chart.

It is for use by suitably qualified healthcare professionals, and by the clinical software they use, in place of reading values from a printed growth chart.

opengrowth **does not diagnose**, and produces no diagnosis, clinical interpretation, recommendation, referral advice, alert, or risk score. Clinical interpretation of a measurement, and every decision about a child's care, rests entirely with the healthcare professional, who uses the calculated values alongside the history, examination, and other information.

## Wording rules

To keep every surface consistent with this statement:

- **Use:** calculate, calculation, express, SDS, centile, position relative to the reference, growth reference, chart coordinates.
- **Do not use** to describe what opengrowth does: diagnose, detect, identify, screen, monitor, assess, interpret, decide, predict, alert, flag, recommend, refer, risk. They may appear only to say what opengrowth does not do.
- Help text, API descriptions, the README, release notes, and the documentation site follow these rules. A change that adds a capability outside this statement needs a regulatory impact assessment first (`R44`).

## Users

| User | Role |
| --- | --- |
| Healthcare professionals | The intended user of the calculated values. |
| Clinical software (EPRs, EHRs, personal health records) | Calls the API, the Python bindings, or the WebAssembly package and displays the values to a healthcare professional. The deploying organisation is responsible for its own system under DCB0160. |
| Software engineers | Integrate, test, and administer opengrowth through the CLI and API. Not users of the values for care. |

Children and families may be shown the values or a chart by a healthcare professional, but they are not the intended user.

## Surfaces

| Surface | Within the intended purpose |
| --- | --- |
| HTTP API (`opengrowth serve`) | Yes, called by clinical software. |
| Python bindings, WebAssembly package | Yes, embedded in clinical software. |
| CLI | For integration, testing, and administration. Clinical software may call it. |
| MCP server | **Excluded until assessed.** An AI agent that relays values to a person is outside this statement. Tool descriptions say so, and the hazard log covers it as foreseeable misuse. |

## Population and references

Children and young people within each reference's published range: for UK-WHO, from 23 weeks' gestation to 20 years. Syndrome-specific references (Turner syndrome, Trisomy 21) are for children with that diagnosis, made by a clinician and chosen by the user; opengrowth does not choose a reference.

## Limits of the calculation

- Measurements more than 8 SD from the reference median (15 SD for BMI) are refused as probable measurement or data-entry errors. This is a plausibility check on the input, not a clinical judgement.
- Where a reference has no data for a measurement at an age, for example BMI below two weeks, opengrowth says so and calculates nothing.
- Corrected age is calculated from the gestation entered; opengrowth cannot check that the gestation is right.

## Not intended for

- Adults.
- Use by children, families, or the public without a healthcare professional.
- Choosing a growth reference, or diagnosing the condition a syndrome-specific reference applies to.
- Prescribing, dosing, or any therapeutic calculation.
