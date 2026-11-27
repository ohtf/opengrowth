<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Ages and gestation

opengrowth calculates every age from three inputs: the date of birth, the date of the measurement, and the gestation at birth, in weeks and days. From them it works out two ages, chronological and corrected, and calculates an SDS and centile at each.

## Decimal age

A decimal age is an age in years, as a decimal fraction: the number of days divided by 365.25, the average length of a year once leap years are counted. A child measured 1,000 days after birth is 2.738 years old.

## Gestation and the due date

A pregnancy is counted from the first day of the mother's last menstrual period, and lasts 40 weeks, or 280 days, to the due date, the estimated date of delivery (EDD). Ovulation comes about two weeks after that first day, so the time from conception is about 266 days. A baby born from 37 to 42 weeks, 3 weeks before to 2 weeks after the due date, is born at term.

Gestation at birth is written in completed weeks and extra days: 27+3 is 27 weeks and 3 days. opengrowth accepts gestations from 22+0 to 44+6. The command-line tool needs the gestation; the API, like the RCPCH server, takes a request without one as 40+0, which is right only for a baby born on their due date.

## Chronological age

The time since birth, whatever the gestation. A baby born at 24 weeks is 16 weeks old on their due date: 16 × 7 / 365.25 = 0.307 years.

## Corrected age

The time since the due date. A baby born early is younger by corrected age than by chronological age, by the time they were born early, and until their due date their corrected age is negative. Correcting compares a baby with others of the same maturity, rather than with babies born at term, who have had longer to grow.

A baby born at 27+3 and measured 6 weeks later is 0.1150 years old by chronological age, and -0.1259 by corrected age: their due date is still 46 days away. A baby born after 40 weeks is corrected the other way, and is older by corrected age: one born at 41+5 stays 12 days older by corrected age than by chronological age.

## Which ages each reference corrects

| References | Correction |
| --- | --- |
| UK-WHO, Trisomy 21 (UK and AAP), Turner syndrome | Every gestation other than exactly 40+0, at every age. A baby born at 37+2 stays 19 days younger by corrected age throughout childhood, though the difference matters less as they grow. |
| WHO, CDC | The term rule. A baby born at 37 to 42 weeks, whatever the days, is not corrected. One born before 37 weeks is corrected until their corrected age reaches 2 years, and not after. One born after 42+6 is corrected at every age. |

Where no correction applies, a result's `corrected_decimal_age` is the same as its `chronological_decimal_age`. Each result also carries sentences saying whether an age was corrected and why, one written for clinicians and one in plain language, in `measurement_dates.comments`.

## Why UK-WHO corrects every gestation

Paper charts corrected only babies born before 37 weeks, and only until 1 year of corrected age for those born at 32 to 36 weeks, or 2 years for those born earlier. Correcting by hand was laborious, and on a paper chart the years crowd together as children grow, so the correction mattered less.

With the calculation automated, the RCPCH Digital Growth Charts project board chose to correct throughout childhood, and to correct babies born at term as well: each baby is compared with babies born at the same gestation, rather than with all term babies pooled together. A baby born after 40 weeks is corrected backwards, with no upper limit. `rcpchgrowth` calculates UK-WHO this way, and so does opengrowth.

Near birth this can give a different centile from a paper chart, which compares every term baby with one pooled term population at birth, and draws no centile lines from birth to 2 weeks. RCPCH's [investigation of the difference](https://growth.rcpch.ac.uk/safety/investigations/pooled-term-data-discrepancy-report/) works through examples.

## Other ages in a result

- **Calendar ages**, chronological and corrected, in words: for example "8 years and 2 months".
- **Corrected gestational age**, in weeks and days, while the baby is younger than 42 weeks by corrected age. This is sometimes called post-menstrual age.
- **The estimated date of delivery**, as a date and in words: for example "Sun 12 April, 2020".

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Date and Age Calculations](https://growth.rcpch.ac.uk/clinician/date-age-calculations/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for opengrowth, with each reference's correction rule, checked against opengrowth's code, and worked examples added.
