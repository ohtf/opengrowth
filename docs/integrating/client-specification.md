<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Client specification

How a system that shows opengrowth's results should draw growth charts. The rules are the RCPCH Digital Growth Charts project board's specification, much of it inherited from the paper charts so that clinicians find the digital ones familiar. RCPCH's API licence requires them; opengrowth's licence does not, but a chart drawn any other way can mislead the clinicians who read it.

The simplest way to meet them is RCPCH's open-source [chart component](chart-component.md), which was built to this specification. The rest of this page is for those who cannot use it, or who build their own.

!!! danger "Keep the provenance with every result"

    A safe client keeps each result's complete `provenance` object with the stored result, unchanged ([Persisting results](persisting-results.md)).

## The UK-WHO data are not a lookup table

The UK-WHO reference is not a table of measurements against ages and centiles. Measurements and ages are continuous, so such a table would be vast and still lose accuracy. It is made of four datasets, from two references ([Growth charts](../background/charts.md)):

- UK90, from 23 weeks' gestation to 42 weeks: length from 25 weeks, weight, and head circumference.
- WHO 2006, from 2 weeks after term to 2 years: length, measured lying down, weight, BMI, and head circumference.
- WHO 2006, from 2 to 4 years, with height measured standing.
- UK90, from 4 to 20 years, with head circumference to 17 years in girls and 18 in boys.

Established clinical guidance sets how the four are combined. It leaves small steps in the lines at 2 and 4 years, which are intended and clinically valid.

## What this means for a chart

The datasets overlap, so each centile line must be drawn as four separate series, which break where they meet. The steps must be kept. At 2 years in particular, children change from being measured lying down to standing, and the reference has values for both at exactly 2 years, so both are plotted. A charting library given all four as one series from 0 to 20 years will join the gaps, and the steps will be lost.

The chart coordinates operation returns the lines as `centile_data`: a list with one entry for each dataset, named by its table (for UK-WHO, `uk90_preterm`, `uk_who_infant`, `uk_who_child`, and `uk90_child`). Each holds, for the sex requested, one series for each centile, with its SDS, its centile, and its points, each with `x`, the decimal age, `y`, the measurement, and `l`, the centile's label. The other sex is `null`.

- `centile_format` names the lines: `cole-nine-centiles`, the default and the UK standard, two-thirds of an SD apart; `three-percent-centiles`; `five-percent-centiles`; or `eighty-five-percent-centiles`. It can instead be a list of up to 15 centiles, or of SDS values when `is_sds` is `true`. WHO and CDC charts use their own collections. `extended-who-centiles` is accepted, as by the RCPCH server, but returns an error, as it does there.
- Only the measurement requested is returned, so a full set of charts takes one call for each measurement.
- The child's own points come from each calculation result's `plottable_data` ([Growth charts](../background/charts.md#what-opengrowth-gives-a-chart)).

## Drawing the chart

- Let the user switch between the height and weight charts, or see both, and offer BMI and head circumference charts to health staff.
- Let the chart zoom while keeping its axes visible, or offer a choice of age ranges.
- Offer SDS charts to health staff.
- Do not emphasise the 50th centile: it must be no darker or wider than the other lines, as that could suggest to families that being on it is desirable or normal. The 0.4th, 9th, 50th, 91st, and 99.6th centiles are dashed, and the 2nd, 25th, 75th, and 98th are solid.
- Choose colours that meet accessibility guidelines and are clearly visible, avoid combinations that make the chart harder to use, and avoid the dated pink and pale blue for girls and boys.
- Do not crowd the chart, and show information only where it applies: information about puberty in girls belongs only on girls' charts, for example.
- Point out that weights in the first weeks after birth are hard to compare with the lines: the paper charts draw no lines from birth to 2 weeks.
- Show why the lines break: where one dataset meets another, and where children change from lying to standing.
- Label the axes with suitable intervals. Below 42 weeks' gestation, the age axis shows gestation; from 42 weeks to 2 years, weeks and months; above 2 years, months and years; above 4 years, years and half-years. The measurement axis shows units, and spans the measurements rather than the whole chart.
- Offer help with reading the chart, drawn from the UK-WHO charts' instructions.

### Centile charts

- Use the Cole nine centiles.
- Scale each part of the chart to suit its age range.
- When the user points at a measurement, show its exact age and centile band, or show them in a table.
- Do not join the points with lines.
- Plot a measurement at chronological age as a round dot, and at corrected age as a cross. When both are shown, join them with a line, often with an arrow back to the dot.
- A bone age, measured from an X-ray of the left hand, is plotted as a cross at the bone age, with the height measured the same day, joined by a dotted or dashed line to the measurement's own points.
- An event, such as the start of a treatment, is a vertical arrow above or below its measurement, outside the centile lines.
- Offer a switch between chronological age, corrected age, and both.
- Leave out grid lines, which help only for plotting by hand, and keep the measurement axis unobtrusive.

### SDS charts

- Put age along the bottom and SDS up the side, so the centile lines become horizontal.
- Plot every available measurement, weight, height, head circumference, and BMI, on the same chart, each in its own consistent colour: weight always red and height always blue, for example.
- Points may be joined by fine lines.
- The SDS axis crosses the age axis at 0, with horizontal lines every 0.67 SD from -2.67 to 2.67.
- Plot the SDS calculated at corrected age against chronological age, labelled with how many weeks early the child was born.

### Gestation at birth

On a centile chart, make clear that gestation at birth has been allowed for, by offering the measurement at chronological age, as a circle, and at corrected age, as a cross, joined by a line when both are shown, with a switch between them.

UK-WHO corrects for gestation at every age, and for babies born at term too ([Ages and gestation](../background/ages.md)). Paper charts corrected by hand, and so only until 1 or 2 years, depending on how early the baby was born:

| Gestation at birth | Paper charts | UK-WHO in opengrowth |
| --- | --- | --- |
| 39+4 | Plotted on the pooled term reference | Plotted on the UK90 preterm data until 42 weeks' corrected gestation, and corrected at every age |
| 35+6 | Corrected until 1 year of corrected age | Corrected at every age |
| 26+0 | Corrected until 2 years of corrected age | Corrected at every age |

WHO and CDC follow the term rule instead ([Ages and gestation](../background/ages.md#which-ages-each-reference-corrects)).

## Rendering standards

The whole of childhood need not be shown at once, but the lines and points must meet these standards:

- Label the centile lines.
- Keep the overlaps between datasets visible, and never join them by interpolation.
- Draw the 0.4th, 9th, 50th, 91st, and 99.6th centiles dashed: not dotted, and not continuous.
- Draw the 2nd, 25th, 75th, and 98th centiles as thin continuous lines.
- Label the axes: height or length in cm, weight in kg, body mass index in kg/m², head circumference in cm, and age in years.
- Mark the age axis monthly under 2 years, and every 3 months over 2.
- Show measurements to one decimal place.
- Show centiles as whole numbers, except above 99 or below 1. Beyond the outermost lines, show them as > 99.6 or < 0.4.

## The nine-centile format

The nine lines of the UK90 and UK-WHO charts are labelled with rounded centiles, but defined by exact SDS:

| Centile | SDS | Line |
| --- | ---: | --- |
| 0.4th | -2.67 | Dashed |
| 2nd | -2.00 | Continuous |
| 9th | -1.33 | Dashed |
| 25th | -0.67 | Continuous |
| 50th | 0 | Dashed |
| 75th | 0.67 | Continuous |
| 91st | 1.33 | Dashed |
| 98th | 2.00 | Continuous |
| 99.6th | 2.67 | Dashed |

## Centile bands

A centile space is the distance between two lines. A measurement is "on" a line within a quarter of a centile space of it, and otherwise "between" two lines. Each result's centile band sentence says which, for example "This height measurement is on or near the 2nd centile." Upstream and opengrowth both place each band 0.1665 SD either side of the line's rounded centile, which gives these limits:

| The measurement is | SDS from | SDS to |
| --- | ---: | ---: |
| well outside the normal range. Please check its accuracy. | | -4 |
| below the normal range | -4 | -2.8186 |
| on or near the 0.4th centile | -2.8186 | -2.4856 |
| between the 0.4th and 2nd centiles | -2.4856 | -2.2202 |
| on or near the 2nd centile | -2.2202 | -1.8872 |
| between the 2nd and 9th centiles | -1.8872 | -1.5073 |
| on or near the 9th centile | -1.5073 | -1.1743 |
| between the 9th and 25th centiles | -1.1743 | -0.8410 |
| on or near the 25th centile | -0.8410 | -0.5080 |
| between the 25th and 50th centiles | -0.5080 | -0.1665 |
| on or near the 50th centile | -0.1665 | 0.1665 |
| between the 50th and 75th centiles | 0.1665 | 0.5080 |
| on or near the 75th centile | 0.5080 | 0.8410 |
| between the 75th and 91st centiles | 0.8410 | 1.1743 |
| on or near the 91st centile | 1.1743 | 1.5073 |
| between the 91st and 98th centiles | 1.5073 | 1.8872 |
| on or near the 98th centile | 1.8872 | 2.2202 |
| between the 98th and 99.6th centiles | 2.2202 | 2.4856 |
| on or near the 99.6th centile | 2.4856 | 2.8186 |
| above the normal range | 2.8186 | 4 |
| well outside the normal range. Please check its accuracy. | 4 | |

The project board's specification puts each band around the line's exact SDS instead, so its limits differ from these by up to 0.054 SD, at the 2nd and 98th centiles: for example, its band for the 2nd centile runs from -2.17 to -1.83. `rcpchgrowth` chose the rounded centiles so that one rule serves every collection of lines, and opengrowth reproduces it exactly until clinicians have reviewed the difference ([standard](https://github.com/ohtf/opengrowth/blob/main/spec/standard.md), section 6.2). The specification also gives BMI bands their own descriptions; neither `rcpchgrowth` nor opengrowth returns those.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Client Specification](https://growth.rcpch.ac.uk/integrator/client-specification/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It describes the chart coordinates as opengrowth returns them, gives the centile bands opengrowth calculates, with their difference from the specification, and leaves out RCPCH's licence terms and services and the specification's BMI descriptions.
