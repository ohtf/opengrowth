<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Growth references

A growth reference describes how a defined population of children grew: for each sex and measurement, at each age, the spread of the measurements, summarised as three numbers, L, M, and S ([How the calculations work](how-it-works.md)). opengrowth compares a child's measurement with the reference the user chooses, and calculates where it falls: its SDS and centile. It never chooses a reference itself.

## The references

Each reference has its own operations in the API, under its own path, and its own name for the command-line tool's `--reference`. Every reference offers a calculation, a bulk calculation, chart coordinates, and fictional child data; the [API reference](../api/reference.md) gives their requests and responses.

| Reference | API path | `--reference` | Describes | Measurements | Ages with data |
| --- | --- | --- | --- | --- | --- |
| UK-WHO | `/uk-who` | `uk-who`, the default | Children in the UK | Height, weight, BMI, head circumference | 23 weeks' gestation to 20 years |
| WHO | `/who` | `who` | The WHO growth standards and reference | Height, weight, BMI, head circumference | Term to 19 years; weight to 10 years, head circumference to 5 years |
| CDC | `/cdc` | `cdc` | Children in the US, as the CDC growth charts describe them | Height, weight, BMI, head circumference | Term to 20 years; BMI from 2 years, head circumference to 3 years |
| Trisomy 21 (UK) | `/trisomy-21` | `trisomy-21` | Children with Down syndrome in the UK and Ireland | Height, weight, BMI, head circumference | Term to 20 years; BMI to 18.82 years, head circumference to 18 years |
| Trisomy 21 (AAP) | `/trisomy-21-aap` | `trisomy-21-aap` | Children with Down syndrome in the US | Height, weight, BMI, head circumference | Term to 20 years; height and head circumference from 1 month, BMI from 2 years |
| Turner syndrome | `/turner` | `turners-syndrome` | Girls with Turner syndrome in the UK | Height only | 1 to 20 years |

In requests, `height` is height or length, however it was measured, and head circumference is `ofc`, for occipitofrontal circumference. Where a reference has no data for a measurement at an age, opengrowth says so, in the same words as the RCPCH server, and calculates nothing.

!!! warning "The syndrome-specific references"

    The Trisomy 21 and Turner syndrome references describe children with those conditions, and are for a child known to have one. opengrowth does not diagnose either condition: choosing the reference stays with the clinician. [Syndrome-specific references](syndrome-references.md) says how a system can offer them safely.

## How UK-WHO is put together

UK-WHO joins three sources. opengrowth chooses the table from the child's age, corrected for gestation, so a client asks for UK-WHO and never for one of its tables:

| Age | Source |
| --- | --- |
| 23 weeks' gestation to 42 weeks (2 weeks after term) | UK90, preterm and at birth |
| 42 weeks to 2 years | WHO 2006 growth standard, length measured lying down |
| 2 to 4 years | WHO 2006 growth standard, height measured standing |
| 4 to 20 years | UK90 |

Length data begin at 25 weeks' gestation, BMI 2 weeks after term, and head circumference ends at 18 years in boys and 17 in girls.

The UK-WHO paper charts compare every baby born at term, from 37 to 42 weeks, with one pooled term population at birth, and draw no centile lines from birth to 2 weeks. opengrowth, like the RCPCH server, compares each baby with babies born at the same gestation instead, so its centiles in the first weeks can differ from those read off a paper chart ([Ages and gestation](ages.md)). RCPCH's [investigation of the difference](https://growth.rcpch.ac.uk/safety/investigations/pooled-term-data-discrepancy-report/) works through examples.

## WHO and CDC

The WHO reference joins the WHO 2006 growth standard, daily from birth to 1,856 days, just over 5 years, and the WHO 2007 growth reference, monthly from there to 19 years. The CDC reference uses the earlier, pre-2025 version of the WHO 2006 table for height and weight to 2 years, the CDC infant table for head circumference to 3 years, and the CDC 2000 growth charts from 2 to 20 years, with the CDC's extended method for BMI above the 95th centile. Neither has data before term, so a baby who is still before term by corrected age gets no corrected SDS against them.

[The WHO reference](who.md) explains how opengrowth calculates WHO results beyond 3 SD and between the table's ages.

## Where the data come from

Every table opengrowth uses is compiled into it, as published in `rcpchgrowth` 4.6.5, and checked against its recorded SHA-256 checksum before it is first used. `opengrowth data manifest` lists each table with its source, file, checksum, and terms, and every result's provenance names the tables it read.

Each table keeps its owner's terms, which [Licences](../about/licences.md) sets out. The UK90 tables, which UK-WHO uses before 42 weeks and after 4 years, are included for research and development only, while a licence application to UK Research and Innovation is pending.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Growth Chart References](https://growth.rcpch.ac.uk/clinician/growth-references/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for opengrowth: the paths, names, and age limits are opengrowth's, checked against its code, and the RCPCH service's details are left out.
