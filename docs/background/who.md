<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# The WHO reference

How opengrowth calculates against the WHO growth standard and growth reference, as `rcpchgrowth` 4.6.5 does, and why a WHO result can differ slightly from a UK-WHO result for the same child in the first months.

## Two datasets

- **The WHO Child Growth Standards (2006)** describe how children grow from birth to 5 years when their needs are met. opengrowth uses WHO's daily LMS tables, with a row for every day of life from birth to 1,856 days.
- **The WHO Growth Reference (2007)** runs from 5 to 19 years, with LMS values for each month: height and BMI to 19 years, and weight to 10.

UK-WHO uses the 2006 standard too, from 2 weeks to 4 years, but as the weekly and monthly tables published for the UK-WHO charts, between whose rows it interpolates cubically. The WHO reference uses the daily tables that WHO distributes with its own software, so under 5 years a measurement on any day of life matches a row exactly and needs no interpolation, and the values are the ones WHO's software uses.

## Where the datasets meet

WHO's software for the two datasets, `anthro` and `anthroplus`, disagrees slightly at 5 years. For boys' height, the 2006 standard's median at 1,826 days is 109.9593 cm, and the 2007 reference's at 60 months is 109.7265 cm, 0.23 cm less. The published tables of the 2007 reference begin at 61 months, so its 60-month row looks like an extension added for continuity rather than data.

opengrowth, like `rcpchgrowth`, uses the 2006 standard up to and including 1,856 days, about 61 months, and the 2007 reference after that. So there is no step at exactly 5 years: the change comes where the 2007 tables begin. RCPCH reported the difference to WHO as [anthro issue 64](https://github.com/WorldHealthOrganization/anthro/issues/64).

## Beyond 3 SD

WHO restricts the LMS method in the tails, for weight and BMI at every age, and for every measurement under 5 years. A measurement beyond 3 SD gets its SDS by extending linearly from 3 SD: each further SD spans the measurement distance between 2 and 3 SD, or between -2 and -3 SD below the median. WHO's [computation guide](https://cdn.who.int/media/docs/default-source/child-growth/growth-reference-5-19-years/computation.pdf) gives the rule.

The rule applies only in one direction, from a measurement to its SDS. Centile lines, calculated from an SDS to a measurement, use the plain LMS formula, because WHO's own published centile curves do. So a measurement taken from a line drawn beyond 3 SD does not calculate back to exactly that line's SDS. This asymmetry is deliberate.

## Between the table's ages

WHO's SDS lookups interpolate linearly between table ages, as WHO's software does, but the measurement at an SDS, for a centile line, interpolates cubically where it can, as for the other references. So between table ages the two directions are not exact inverses. Under 5 years every day has a row, so this matters only from 5 years, between the 2007 reference's monthly rows. It is `rcpchgrowth`'s behaviour, which opengrowth keeps ([Differences](../differences.md#rcpch-behaviour-that-opengrowth-keeps)).

## How WHO results differ from UK-WHO

Both references use the WHO 2006 standard in infancy, but a child can still get a slightly different SDS from each in the first months. The WHO reference reads WHO's daily rows without interpolating, while UK-WHO interpolates cubically between weekly and monthly rows and starts the WHO standard 2 weeks after term. When `rcpchgrowth` moved its WHO reference to the daily tables, 18 of its test cases changed by more than 0.001 SD, by up to 0.0115, all in early infancy and in babies born preterm. Prof Tim Cole compared the two and confirmed that these are expected differences between the two references, not errors.

The two references also correct ages differently: WHO follows the term rule, and UK-WHO corrects every gestation ([Ages and gestation](ages.md#which-ages-each-reference-corrects)).

## How it is checked

- `rcpchgrowth`'s tests compare its WHO SDS with those of WHO's `anthro` software for 192 cases under 2 years, the largest difference being 1.1 × 10⁻⁶, and its centile lines with WHO's published centile curves at 123,061 points, within 0.1%.
- opengrowth's WHO responses are identical, outside provenance, to the RCPCH server's for 5,791 of 5,793 captured calculation requests, the other two being [listed fixes](../differences.md#fixes-where-the-rcpch-server-fails), and for every captured chart and fictional child request. Named-format charts are compared with the server's chart files once its own engine has regenerated them, since the files it serves are out of date ([Differences](../differences.md#named-format-charts-are-calculated)).

## Where the data come from

RCPCH generated the WHO tables in `rcpchgrowth` from WHO's R packages, [anthro](https://github.com/WorldHealthOrganization/anthro) for birth to 5 years and [anthroplus](https://github.com/WorldHealthOrganization/anthroplus) for 5 to 19, using its own branches of them, [`rcpch/anthro`](https://github.com/rcpch/anthro/tree/z-to-measurement) and [`rcpch/anthroplus`](https://github.com/rcpch/anthroplus/tree/precision), which add the measurement at an SDS and control over the precision of SDS values. opengrowth embeds the tables as published in `rcpchgrowth` 4.6.5: `who-2006-infant`, birth to 2 years, `who-2006-child`, 2 to 5 years, and `who-2007-child`, 5 to 19 years, each with its checksum in `opengrowth data manifest`.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [WHO Reference Implementation](https://growth.rcpch.ac.uk/developer/who-reference-implementation/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for opengrowth, with the interpolation in each direction and opengrowth's own checks added.
