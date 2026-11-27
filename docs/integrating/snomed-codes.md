<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# SNOMED CT codes

When you store opengrowth's results in a patient record, it is good practice to code the stored values with [SNOMED CT](https://www.snomed.org/) concepts. Coded centiles and SDS are easier to exchange between systems, to search, and to keep consistent.

Store the result's complete [provenance](persisting-results.md) with it too, whatever your terminology model. They do different jobs: SNOMED CT says what the value means, and provenance says which code and data produced it.

!!! note "Suggestions, not requirements"

    The concepts below are suggestions to start from, not a required value set. Your organisation's terminology team may already have conventions: if so, follow them. The aim is to point to active SNOMED CT observable entities that match what opengrowth returns.

## What opengrowth returns

For each measurement sent, `height`, `weight`, `bmi`, or `ofc`, opengrowth returns:

- a **centile**, the measurement's percentile position for the child's age and sex, and
- an **SDS**, or z-score.

The concepts below are SNOMED CT observable entities, the question being answered. Store the number opengrowth calculated as the value against the matching concept: for example, *Body height for age percentile* = `0.4`.

## Concepts for centiles

SNOMED CT models a centile as a "for age percentile" observable entity:

| `measurement_method` | Result | Suggested concept | SCTID |
| --- | --- | --- | --- |
| `height` | Height centile | Body height for age percentile (observable entity) | `1153605006` |
| `height`, under 2 years | Length centile | Length for age percentile (observable entity) | `1153591001` |
| `weight` | Weight centile | Weight for age percentile (observable entity) | `1153592008` |
| `bmi` | BMI centile | Body mass index for age percentile (observable entity) | `1153602009` |
| `ofc` | Head circumference centile | Child head circumference for age percentile (observable entity) | `1153595005` |

## Concepts for SDS

In growth charting an SDS is a z-score, which SNOMED CT models as a "for age z-score" observable entity:

| `measurement_method` | Result | Suggested concept | SCTID |
| --- | --- | --- | --- |
| `height` | Height SDS | Body height for age z-score (observable entity) | `1153604005` |
| `height`, under 2 years | Length SDS | Length for age z score (observable entity) | `1153590000` |
| `weight` | Weight SDS | Weight for age z-score (observable entity) | `1153593003` |
| `bmi` | BMI SDS | Body mass index for age z-score (observable entity) | `1153596006` |
| `ofc` | Head circumference SDS | Head circumference for age z-score (observable entity) | `1153594009` |

!!! tip "Height or length under 2 years"

    Before 2 years, children are usually measured lying down, for length, rather than standing, for height. If your system records which, use the *Length for age* concepts for measurements taken lying down, and the *Body height for age* concepts for those taken standing. If it does not, the *Body height for age* concepts are a reasonable default.

## The measurement itself

SNOMED CT also has observable entities for the measurements themselves, separate from those for the centile and SDS: for example *Body height measure* (`50373000`), *Body weight* (`27113001`), *Body mass index* (`60621009`), and *Head circumference* (`363812007`).

## The SNOMED CT release

The RCPCH Digital Growth Charts documentation checked these concepts against the UK Edition of SNOMED CT, release 42.1.0 of 6 May 2026, when all were active. SNOMED CT changes with each release, and a concept can be made inactive and replaced, so check them against the release your own terminology server holds before you go live.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [SNOMED CT codes for storing growth results](https://growth.rcpch.ac.uk/integrator/snomed-codes/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is adapted for opengrowth; the concepts are as RCPCH checked them, and have not been checked again for this site.
