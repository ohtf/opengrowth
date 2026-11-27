<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# How the calculations work

opengrowth turns a measurement into an SDS and a centile by the LMS method, from the tables of the reference the user chooses. Its command-line tool and its API are two ways in to the same engine. The method is the one `rcpchgrowth` uses, reproduced exactly: for the same inputs, opengrowth's results are identical to the last bit to `rcpchgrowth` 4.6.5's, apart from the [differences](../differences.md) it lists.

## SDS and centiles

A **standard deviation score** (SDS), or z-score, says how many standard deviations a measurement is from the reference's median, for a child of that age and sex. An SDS of 0 is the median; -2 is two standard deviations below it.

A **centile** is the percentage of the reference population that measured less. An SDS of 0 is the 50th centile, and -2 is the 2.3rd. opengrowth converts between the two with the standard normal distribution, using a translation of SciPy's functions for it, which gives results identical to SciPy's.

Each **centile line** on a chart is the measurement that a given share of the reference population fell below: 50% below the 50th centile line, 91% below the 91st. The nine lines of the UK-WHO charts, from the 0.4th to the 99.6th, are two-thirds of a standard deviation apart. 99.2% of the reference population fall between the outer two lines, and half between the 25th and 75th.

## The LMS method

The LMS method (Cole and Green, 1992) summarises a reference as a table of three values at a series of ages, for each sex and measurement:

- **M**, the median;
- **S**, the coefficient of variation, a measure of the spread;
- **L**, a power that removes the skew from the distribution.

The SDS of a measurement `x` is

```text
z = ((x / M)^L - 1) / (L × S)        or, when L is 0,        z = ln(x / M) / S
```

and the measurement at an SDS, from which centile lines are drawn, is

```text
x = M × (1 + L × S × z)^(1 / L)      or, when L is 0,        x = M × e^(S × z)
```

## Between the table's ages

Most children's ages fall between two rows of a table, which are days, weeks, months, or years apart, depending on the reference and the age. opengrowth then interpolates L, M, and S for the exact age:

- An age that rounds to a row's age, at four decimal places, uses that row.
- Where there are two rows on each side of the age, opengrowth fits a cubic through the four: the four-point Lagrange interpolation of Tim Cole's LMSgrowth.
- Otherwise, as at the ends of a table and at a join between two tables, it interpolates linearly between the rows either side.
- The WHO reference always interpolates linearly, and so do the CDC tables for BMI.

## Limits

- A measurement more than 8 SD from the median, or 15 SD for BMI, is refused as a probable error of measurement or entry. The limits were set from a review of datasets and the literature, recorded in [rcpchgrowth-python issue 32](https://github.com/rcpch/rcpchgrowth-python/issues/32).
- Where a reference has no data for a measurement at an age, opengrowth says so and calculates nothing ([Growth references](growth-references.md)).

## WHO and CDC

Two references calculate differently in part of their range:

- **WHO, beyond 3 SD.** WHO's restricted LMS method applies to weight and BMI at every age, and to every measurement under 5 years: beyond 3 SD, each further SD spans the same measurement distance as the step from 2 to 3 SD, rather than following the LMS curve. [The WHO reference](who.md) explains why.
- **CDC BMI, above the 95th centile.** The CDC's extended method replaces the LMS centile with `90 + 10 × Φ((BMI - P95) / σ)`, where `P95` is the 95th centile, `σ` comes from the CDC table, and `Φ` is the normal distribution, and the SDS is calculated from that centile ([CDC extended BMI data](https://www.cdc.gov/growthcharts/extended-bmi-data-files.htm)).

## Describing a measurement

Each result includes a sentence that places the measurement among the reference's centile lines, for example "This height measurement is between the 0.4th and 2nd centiles." The Digital Growth Charts project board wanted users to report these, rather than exact centiles. The sentence says:

- **on or near** a line, where the SDS is within a quarter of two-thirds of an SD of it, 0.1665;
- **between** two lines, otherwise;
- **below** or **above the normal range**, beyond the outermost lines and their quarter-spacing bands;
- **well outside the normal range**, asking for the measurement to be checked, beyond 4 SD.

The lines are the Cole nine centiles for UK-WHO, WHO, and the syndrome-specific references. CDC uses its own: 3rd, 5th, 10th, 25th, 50th, 75th, 90th, 95th, and 97th, and for BMI a set running from the 5th to the 99.99th.

In API responses, centiles are rounded to four decimal places, and SDS values are not rounded.

## Chart coordinates

A centile line is the measurement at a fixed SDS, calculated at a fixed series of ages, the same that `rcpchgrowth` uses. For UK-WHO the series is weekly before term, days to weeks apart up to 2 years, and monthly from 2 years. Ages where the reference has no data are left out of the line. For UK-WHO's Cole nine centiles, each line is drawn at its SDS rounded to the nearest two-thirds; every other line is drawn at its centile's exact SDS.

## Mid-parental height

Mid-parental height uses UK-WHO, WHO, or CDC. Each parent's height is converted to an SDS at adult age, 20 years for UK-WHO and CDC and 19 for WHO. The mid-parental SDS is half the mean of the two, `(mother's SDS + father's SDS) / 4`, which allows for regression to the mean: very tall or very short parents tend to have children less extreme than themselves. opengrowth returns it as a height, an SDS, and a centile, with lines at 1.4 SD either side, which in the study the method comes from contained 90% of children (Wright and Cheetham, 1999; see [Growth chart papers](papers.md)).

## How opengrowth is checked

- Against Tim Cole's independent test vectors, the R vectors: all 7,721 UK-WHO SDS agree within 0.001, and every value the vectors leave out, opengrowth leaves out too.
- Against `rcpchgrowth` 4.6.5: the same 7,721 SDS and centiles, and thousands of other cases, are identical to the last bit.
- Against the RCPCH server: every captured request, more than 34,000, gets the same response outside provenance, apart from the listed differences.

The [conformance suite](https://github.com/ohtf/opengrowth/blob/main/conformance/README.md) has the details, and CI runs it on every change.

## Further reading

- [What are centiles?](https://www.openhealthhub.org/t/centile-part-1-what-are-centiles/463), on Open Health Hub.
- [The development of growth references and growth charts](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3920659/), by T J Cole.
- The CDC's [percentile data files with LMS values](https://www.cdc.gov/growthcharts/percentile_data_files.htm), and the [construction of the LMS parameters](https://www.cdc.gov/nchs/data/nhsr/nhsr063.pdf) for its 2000 charts.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [How the API Works](https://growth.rcpch.ac.uk/clinician/how-the-api-works/), the method sections of [RCPCHGrowth](https://growth.rcpch.ac.uk/developer/rcpchgrowth/), and the description of centile lines, CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for opengrowth and checked against its code, which corrects two details: CDC interpolates cubically except for BMI, and the mid-parental range is 1.4 SD either side.
