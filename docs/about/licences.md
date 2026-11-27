<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Licences

## opengrowth's code

opengrowth's code is licensed under the [GNU Affero General Public License, version 3 or later](https://github.com/ohtf/opengrowth/blob/main/LICENSE), inherited from `rcpchgrowth`, the engine it reimplements. Anyone may use, change, and share it. Anyone who runs a changed version as a service that people use over a network must offer those people its source code, so improvements come back to everyone.

The code is copyright the Open Health Technology Foundation contributors. Portions derived from `rcpchgrowth` are copyright the Royal College of Paediatrics and Child Health.

## This documentation

This site is licensed [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/), copyright the Open Health Technology Foundation contributors. Pages adapted from the Digital Growth Charts documentation are also copyright 2020-2026 the Royal College of Paediatrics and Child Health and that documentation's contributors, as each page says ([Acknowledgements](acknowledgements.md)).

## The growth reference tables

The tables are not covered by the code licence. Each keeps its owner's terms, which take precedence:

| Tables | Owner | Terms |
| --- | --- | --- |
| UK90 preterm and child | UK Research and Innovation, formerly the Medical Research Council | Included for research and development only, while a licence application is pending. No release will be used for patient care until it is granted. |
| WHO 2006 growth standard, and the WHO 2007 growth reference | World Health Organization | Described upstream as open data; not yet recorded in full. |
| CDC 2000 growth charts | US Centers for Disease Control and Prevention, National Center for Health Statistics | A US federal government work; not yet recorded in full. |
| Turner syndrome, UK; Trisomy 21, UK and AAP | Their authors | Not yet recorded. |

`opengrowth data manifest` lists every table with its source, checksum, and terms. Every table is embedded in the program unless its owner asks for its removal; a user who holds a licence can import their own copy with `opengrowth data import`.

## Methods

The LMS method, which opengrowth uses to turn a measurement into an SDS and back, is a published statistical method (Cole and Green, 1992). opengrowth's implementation of it is covered by the code licence.

## Other software

- opengrowth's normal distribution functions are translated from SciPy, under the BSD 3-clause licence; [NOTICE.md](https://github.com/ohtf/opengrowth/blob/main/NOTICE.md) gives the notice.
- Every Rust library compiled into opengrowth is listed, with its licence, in the [SOUP register](https://github.com/ohtf/opengrowth/blob/main/qms/soup.md). CI checks their licences are compatible with the AGPL.
- [Zensical](https://zensical.org/), under the MIT licence, builds this site.

## No warranty

opengrowth is provided without warranty of any kind, as sections 15 and 16 of the AGPL set out. It is pre-alpha software and is not for clinical use ([Safety](../safety.md)).

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Licensing and Copyright](https://growth.rcpch.ac.uk/legal/licensing-copyright/) and [Disclaimer](https://growth.rcpch.ac.uk/legal/disclaimer/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](acknowledgements.md#the-documentation). It is rewritten for opengrowth's licences and the terms of its tables.
