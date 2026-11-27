<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Growth charts

A growth chart plots a child's measurements against age, over the centile lines of a reference. opengrowth calculates both parts, the lines and the point for each measurement, but does not draw charts: the system showing them does, for example with RCPCH's open-source chart component.

## How the UK-WHO charts are built

The UK-WHO charts join the World Health Organization's growth standard to the UK 1990 reference:

| Age | Data |
| --- | --- |
| Preterm and at birth, 23 to 42 weeks' gestation | UK90: British children measured around 1990 |
| 2 weeks to 4 years | The WHO 2006 growth standard, from healthy breastfed children |
| 4 to 20 years | UK90 |

UK90 includes both breastfed and bottle-fed children. The WHO standard comes from children who were breastfed and grew up in good conditions, so it shows how children grow when their needs are met, whatever their feeding or ethnicity, and it describes infancy and early childhood better. The aim of the charts is to show healthy growth for all children.

The lines step where one source meets the next. The steps are deliberate: smoothing them away would mean inventing data that do not exist. There is another step at 2 years, where children are measured standing rather than lying down. Standing compresses the spine a little, so a child's height is slightly less than their length, and the lines shift down to allow for it.

## What opengrowth gives a chart

- **The lines.** The chart coordinates operation returns the measurement at each centile line's SDS, across the reference's ages, for a named set of centiles or for centiles or SDS values the client chooses ([How the calculations work](how-it-works.md#chart-coordinates)).
- **The points.** Every calculation result includes `plottable_data`: for a centile chart, the age and the measurement, and for an SDS chart, the age and the SDS, each at chronological and at corrected age, with the calendar age and the comments on correction for a tooltip.
- **The reference.** Both name the reference they come from, in `provenance.growth_reference`, so a client can confirm that a chart and the results on it come from the same reference.

An SDS chart plots SDS against age, and so can show every kind of measurement on the same axes. Plot its points at corrected age.

## BMI and percentage median BMI

Body mass index is weight in kilograms divided by the square of height in metres. opengrowth takes BMI as a measurement, in kg/m²; it does not calculate it from a weight and a height.

For a BMI measurement, opengrowth also calculates the percentage median BMI: the child's BMI as a percentage of the reference's median BMI for their age and sex, at chronological and at corrected age.

## Mid-parental height

Mid-parental height lets a clinician compare a child's height with what their parents' heights suggest. A common hand calculation adds 13 cm to the mother's height for a boy, or takes 13 cm from the father's height for a girl, and averages the result with the other parent's height. opengrowth, like the UK-WHO charts, averages the parents' SDS instead, with an allowance for regression to the mean ([How the calculations work](how-it-works.md#mid-parental-height)), which is more accurate, particularly when the parents' heights are far apart. A chart can show the mid-parental range beside the child's latest measurement, rather than at the top of the chart, as paper charts do.

## Measuring accurately

A calculation is only as good as its measurement. The UK-WHO chart instructions give this guidance:

- **Weight:** use only Class III clinical electronic scales, set to metric.
- **Length, before 2 years:** use a length board or mat. The person measuring should be trained to do it.
- **Height, from 2 years:** use a correctly installed stadiometer, or an approved portable rigid rule with a T-piece, with the child standing as straight as possible, and record the height to the last whole millimetre.
- **Head circumference:** use a narrow plastic or paper tape, where the circumference is greatest.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Chart information for Health staff](https://growth.rcpch.ac.uk/clinician/chart-information-health-staff/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It keeps how the charts are built and plotted, and what opengrowth provides for them, and leaves out the guidance on when to measure and what a measurement means, which is outside what opengrowth is for.
