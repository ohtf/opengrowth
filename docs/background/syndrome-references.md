<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Syndrome-specific references

Children with Down syndrome or Turner syndrome grow differently from other children, so their measurements are usually compared with a reference that describes children with the same condition. opengrowth has three:

- **Trisomy 21 (UK):** Styles, Cole, Dennis, and Preece, 2002, with BMI from 2013. Children with Down syndrome in the UK and Ireland, from term to 20 years.
- **Trisomy 21 (AAP):** Zemel and others, 2015. Children with Down syndrome in the US, from term to 20 years.
- **Turner syndrome:** Lyon, Preece, and Grant, 1985. The height of girls with Turner syndrome in the UK, from 1 to 20 years.

[Growth references](growth-references.md) gives each one's measurements and ages, and [Growth chart papers](papers.md) the publications.

!!! warning "Only for a child known to have the condition"

    These references describe children with the condition, and are not for other children. opengrowth does not diagnose either condition and does not choose the reference: the clinician does.

## Offering them in a system

This section is for the people who build systems that show growth charts.

### Offer them only where the condition is recorded

- **Do:** offer a syndrome-specific reference for a child whose record shows the condition, where the system can tell.
- **Don't:** offer the syndrome-specific references alongside UK-WHO as a routine choice for every child.

### Keep the results from before

A child's measurements have often been plotted on UK-WHO for some time before the condition is known. A system should:

- **Do:** keep the UK-WHO results from before then.
- **Do:** use the syndrome-specific reference from then on, or from an earlier date the clinician chooses.
- **Do:** ideally, show both, so that the change of reference is plain.
- **Don't:** delete or hide the earlier UK-WHO results unless the clinician asks.

### Growth hormone in Turner syndrome

A girl with Turner syndrome who is treated with growth hormone may need comparing with girls without the condition, as well as with the Turner reference, because the clinical question changes: how her growth compares with her peers' as a result of treatment.

- **Do:** allow the Turner and UK-WHO references to be viewed together, when the clinician wants both.
- **Do:** label each, so there is no doubt which series is which.
- **Don't:** remove the UK-WHO reference once growth hormone starts.

### Side by side

Viewing UK-WHO and a syndrome-specific chart side by side can help, particularly where the reference changes: when the condition becomes known, or when growth hormone starts.

- **Do:** label each chart or series with its reference, for example "UK-WHO" and "Turner syndrome".
- **Do:** design the view with the clinical staff who will use it.
- **Don't:** use any design that could make one reference look like the other.

## What opengrowth provides

Every result, and every set of chart coordinates, names the reference it was calculated against, in `provenance.growth_reference`, so a system can confirm that a chart and the results plotted on it share a reference, and label each with it.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Turner Syndrome and Down Syndrome: Implementation Guidance](https://growth.rcpch.ac.uk/integrator/turner-down-syndrome/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for opengrowth, with the references' sources and what opengrowth provides added.
