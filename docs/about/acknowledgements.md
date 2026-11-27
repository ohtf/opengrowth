<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Acknowledgements

opengrowth builds on the Digital Growth Charts project of the Royal College of Paediatrics and Child Health (RCPCH). RCPCH publishes `rcpchgrowth`, the Python growth engine that opengrowth reimplements; the Digital Growth Charts API server, whose behaviour opengrowth reproduces; and the documentation that much of this site adapts. opengrowth is not an RCPCH project, and RCPCH does not endorse it.

## The Digital Growth Charts project

The NHS commissioned the project in early 2020 to calculate growth results reliably for children aged one year and under. Work began in May 2020, and the scope soon grew to children of all ages, then to mid-parental height, bone age, the Turner syndrome and Down syndrome references, and later the CDC and WHO references.

The project's own acknowledgements thank:

- Prof Tim Cole, Prof Charlotte Wright, and the project board, for specialist advice and guidance in turning paper growth charts into digital ones.
- Dr Dan Leach, for advice on medical device registration.
- Dr Charles van Lennep, for contributions to the React code and the React Native client.
- Dr Anchit Chandran and 'William' Nai-Wei Wang, for proofreading, corrections, and contributions to the documentation.
- Dr Akanimoh Osutuk, for input validation in the Python library, which opengrowth reproduces.

The project's [team page](https://growth.rcpch.ac.uk/about/team/) lists everyone who worked on it.

## The documentation

Pages adapted from the [Digital Growth Charts documentation](https://growth.rcpch.ac.uk) say so at their foot. That documentation is licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), copyright 2020-2026 the Royal College of Paediatrics and Child Health, and Dr Marcus Baw wrote most of it. Its history records these contributors, by name or handle, in order of the number of their changes: Marcus Baw, Anchit Chandran, Michael Barton, eatyourpeas, 'William' Nai-Wei Wang, Simon Chapman, Danny Cowen, Magda Umerska, Tim Cole, mbarton, dc2007git, Charles van Lennep, Sean Cusack, and adelercpch.

This site adapts it as it was at commit [f00799a](https://github.com/rcpch/digital-growth-charts-documentation/commit/f00799aadf2a4eca39299d294ca0908609a13d22), on 24 November 2026.

## The growth references

Each reference is the work of its authors, and its tables keep their owners' terms ([Licences](licences.md)):

- **UK90:** UK Research and Innovation, formerly the Medical Research Council.
- **WHO Child Growth Standards (2006) and WHO Growth Reference (2007):** the World Health Organization.
- **CDC 2000 growth charts:** the US Centers for Disease Control and Prevention, National Center for Health Statistics.
- **Turner syndrome, UK:** Lyon, Preece, and Grant, 1985.
- **Trisomy 21, UK:** Styles, Cole, Dennis, and Preece, 2002, with BMI from 2013.
- **Trisomy 21, American Academy of Pediatrics:** Zemel and others, 2015.

## Methods and software

- The LMS method is Cole and Green's (*Statistics in Medicine*, 1992).
- Prof Tim Cole generated the independent test vectors that opengrowth's calculations are checked against.
- opengrowth's normal distribution functions are a translation of SciPy's, which come from Stephen L. Moshier's Cephes Math Library.
- This site is built with [Zensical](https://zensical.org/).

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Acknowledgements](https://growth.rcpch.ac.uk/about/acknowledgements/) and the history in [The RCPCH Growth Charts Project](https://growth.rcpch.ac.uk/about/about/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's contributors. It is rewritten for opengrowth, with the documentation's contributors and the references' authors added.
