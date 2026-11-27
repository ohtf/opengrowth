<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Questions from clinicians

## How is this different from a paper chart?

With a paper or PDF chart, a person plots the measurement and reads off the centile. opengrowth calculates the centile and SDS from the measurement itself, more precisely than they can be read off a chart, and corrects for gestation at birth without anyone having to work it out ([Ages and gestation](ages.md)). It also calculates mid-parental height, and carries a bone age and notes of events, such as starting growth hormone, through to the chart. opengrowth does not draw the chart: the system showing it does, from the lines and points opengrowth calculates ([Growth charts](charts.md)).

## If the centile is calculated, why keep the curved-line charts at all?

A good question. Traditional growth charts were partly a paper calculator for the centile: the clinician plotted the age and measurement, saw which centile lines it fell between, and recorded that. A calculated centile makes that step unnecessary.

But a chart had another job: showing the trend in a child's growth over time, which a single centile does not. Some form of chart is still needed for that. At first, clinicians will want the traditional chart, because it is familiar. In time, SDS charts may replace it, showing height, weight, head circumference, and BMI on one chart, and better ways of showing growth over time may be developed. How growth trends are shown is open to new ideas.

## Can I use opengrowth for patients?

Not yet. opengrowth is pre-alpha software and is not for clinical use: its quality management system is not yet in force, and the UK90 tables that UK-WHO depends on are included only for research and development, while a licence is applied for ([Safety](../safety.md)). The RCPCH Digital Growth Charts API, which opengrowth reimplements, is registered with the MHRA as a Class I medical device, and clinicians use it through the patient record systems that integrate it.

## Can I try it?

Yes, on your own computer. [Getting started](../getting-started.md) shows how to calculate a measurement with the command-line tool, and how to run the API.

## What does it cost?

Nothing. opengrowth is open-source software under the AGPL, and anyone may use, change, and share it ([Licences](../about/licences.md)). An organisation that runs it as a service has the costs of running a server, and, once it can be used clinically, of its own assurance and clinical safety work.

## Where is the safety documentation?

In the repository: [SAFETY.md](https://github.com/ohtf/opengrowth/blob/main/SAFETY.md) says how hazards are handled, and the [quality management system](https://github.com/ohtf/opengrowth/tree/main/qms) holds the intended purpose and the rest of the technical file as it is written. To report anything that could affect a calculated value, open an [issue](https://github.com/ohtf/opengrowth/issues) labelled `hazard`.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Frequently Asked Questions for Clinicians](https://growth.rcpch.ac.uk/clinician/faqs-for-clinicians/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). The questions about the RCPCH service, its prices and how to get it, are answered for opengrowth instead.
