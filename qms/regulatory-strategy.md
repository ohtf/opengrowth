<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Regulatory strategy

> **Status: draft.** Written by the project. The classification rationale should be checked by a regulatory affairs adviser before the declaration of conformity (`R41`). Where this cites draft legislation, recheck it against the text as made.

## Manufacturer

| Role | Holder |
| --- | --- |
| Legal manufacturer | **Open Health Technology CIC** (trading as the Open Health Technology Foundation), a UK company |
| Person responsible for regulatory compliance (PRRC) | **Dr Marcus Baw** |
| Clinical Safety Officer | Dr Marcus Baw |

Decided 2026-11-17 (`Q11`). Being UK-based, the manufacturer needs no UK Responsible Person for Great Britain, but would need an EU Authorised Representative to place opengrowth on the EU market.

## Is it a medical device?

Yes, once released for clinical use: calculating a child's SDS and centile against a growth reference for healthcare professionals has a medical purpose, and is more than storing or communicating data. The upstream RCPCH Digital Growth Charts API, which performs the same calculations, is registered with the MHRA as Class I (GMDN 65712, "Paediatric growth calculation API software").

Releases before the declaration of conformity are research and development software, labelled not for clinical use.

## Position: Class I

opengrowth is an anthropometric calculation tool ([intended-purpose.md](intended-purpose.md)). It expresses a measurement relative to a reference population, as a printed growth chart does. It does not diagnose, does not produce any interpretation, recommendation, or alert, and leaves all clinical interpretation to the healthcare professional. The position is Class I in every market, and the intended purpose and every surface are worded to keep it so.

### Great Britain

- **Now (UK MDR 2002, as amended):** Class I. Standalone software is an active device. Growth calculation falls under none of the higher active-device rules, so the residual rule applies. This is the MHRA software flowchart result recorded for the upstream device in 2021.
- **Draft Medical Devices (Amendment) Regulations 2026** (notified to the WTO on 8 May 2026 as G/TBT/N/GBR/120; MHRA call for evidence from 11 May 2026): new Schedule 9 classification rules. Software is an active device. Rule 10 makes an active device for diagnosis and monitoring Class IIa only if it supplies energy, images radiopharmaceuticals, *allows direct diagnosis*, or *allows monitoring of vital physiological processes*. opengrowth does neither: it calculates a position on a growth reference, and growth is not a vital physiological process. Rule 12: active devices outside Rules 9 to 11 are **Class I**. Part 10 comes into force 18 months after the regulations are made.
- **Conformity route:** self-declaration of conformity, technical documentation, MHRA registration, UKCA marking. ISO 13485 certification is not legally required for Class I, but this QMS is built to it.
- **Equivalence:** the draft regulations allow a software device to show equivalence to another device where the intended purpose is the same and the data inputs and outputs are similar, using clinical data relating to a specific version of that device. opengrowth's conformance evidence is exactly this: byte-identical responses to `digital-growth-charts-server` 29bb4e9 (`rcpchgrowth` 4.6.5) across the captured corpus ([conformance/README.md](../conformance/README.md)). The equivalence claim needs:
  - the intended purposes to match. opengrowth's statement is narrower in wording than the upstream device's, which mentions supporting the detection of growth disorders. The argument is that the function, inputs, and outputs are identical, and that opengrowth's statement describes the same device more precisely. Record this argument in the clinical evaluation (`R42`);
  - the upstream version named, with its clinical evidence available to cite or independently reproduced. The independent R vectors, which do not depend on RCPCH's evidence, are the fallback.

### European Union and Northern Ireland

EU MDR 2017/745 applies in the EU and, under the Windsor Framework, in Northern Ireland.

- **Current Rule 11:** software providing information used to take decisions with diagnostic or therapeutic purposes is at least Class IIa. The case for Class I rests on the intended purpose. opengrowth provides no information *for diagnosis*; it calculates an anthropometric quantity, as a ruler or printed chart provides one. The case against is that centiles are used in decisions about children's care. MDCG 2019-11 is the guidance to argue from. The Class I case is defensible but not certain under the current rule.
- **Proposed amendment:** the Commission's proposal of 16 December 2025, COM(2025) 1023, would make software Class I by default, classified higher only for use in "serious" or "critical" situations. Those terms are not yet defined, and the proposal is not law; adoption is not expected before late 2026 at the earliest.
- **Route if pursued as Class I:** EU Authorised Representative, technical documentation to MDR Annexes II and III, EU declaration of conformity, CE marking, and EUDAMED registration (actor, device, and UDI). PRRC under Article 15.
- **If Class IIa were required:** a notified body would assess the QMS and the technical documentation. That cost and time is the reason not to pursue the EU until either the Class I case is confirmed or the amendment is in force.

**Decision:** Great Britain first. Revisit the EU and Northern Ireland when COM(2025) 1023 is adopted, or sooner if an EU deployer needs it.

### Features that would change the class

These would move opengrowth towards Class IIa or above in any market, and need a regulatory impact assessment before being built:

- text that interprets a result clinically, such as "faltering growth", "obese", "refer", or "abnormal";
- alerts, flags, or thresholds that prompt action;
- predictions, such as predicted adult height presented for care decisions, or risk scores;
- automatic choice of a growth reference from a diagnosis;
- anything aimed at patients or families interpreting their own results.

**Inherited wording to review.** The upstream centile-band sentences say a measurement is "below the normal range", "above the normal range", or "well outside the normal range. Please check its accuracy." They describe position relative to the reference's centile lines, and the last is an input plausibility prompt. They are kept for byte identity with the upstream device. Review whether "normal range" could be read as interpretation (`Q14`).

## Software safety class (IEC 62304)

**Class B**, as for the upstream device: a wrong value could contribute to a hazardous situation resulting in non-serious injury, such as a delayed referral, with clinical examination and serial measurement acting as external risk controls. Confirm in the risk file (`R21`).

## Standards

| Standard | Use |
| --- | --- |
| ISO 13485:2016 | Quality management system |
| ISO 14971:2019 | Risk management |
| IEC 62304:2006+A1:2015 | Software life cycle, class B |
| IEC 62366-1:2015 | Usability, for text and charts a person reads |
| IEC 82304-1:2016 | Health software product requirements and labelling |
| DCB0129 | NHS clinical risk management (manufacturer) |
| MDCG 2019-11 | Software qualification and classification guidance (EU) |
