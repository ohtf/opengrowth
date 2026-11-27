<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Questions from integrators

## Can we run our own server?

Yes: that is how opengrowth is meant to be used. A server you run is yours to secure, keep running, and keep up to date, and once opengrowth can be used clinically, your organisation is responsible for the deployment's own assurance and clinical safety ([Safety](../safety.md#if-you-deploy-opengrowth), [Security](security.md)).

## Is the gestation required?

The API accepts a request without it, as the RCPCH server does, and calculates as if the baby had been born at 40+0, which is right only for a baby born on their due date. The command-line tool requires it. Send it with every measurement, and always the same value for the same child ([Making calls](making-calls.md#the-parameters)).

## How much work is an integration?

Little, for the calculations. The difficult parts, the statistical tables, choosing the right table for the child's age, and correcting for gestation, are done for you, and the results come back ready to show.

Drawing a growth chart is more work. The chart coordinates operation gives the lines, and RCPCH's open-source [chart component](chart-component.md) draws charts from the results. If you use another technology, its source shows how, and the [client specification](client-specification.md) sets out the rules a chart must follow.

## Does the API return the corrected age?

Yes, from the gestation sent: the corrected and chronological ages, and the SDS and centile at each. UK-WHO corrects throughout childhood, and for babies born at term too, so the chronological results are there for anyone who needs them ([Ages and gestation](../background/ages.md)).

## Should our application check inputs too?

Yes. opengrowth checks every request and says what is wrong, field by field, but a user gets quicker feedback if your application refuses impossible values as they are entered. The limits are those of `rcpchgrowth`, in its [validation constants](https://github.com/rcpch/rcpchgrowth-python/blob/live/rcpchgrowth/constants/validation_constants.py), which opengrowth applies unchanged.

## Can we plot the results on an image of a growth chart?

Perhaps, but avoid it. An image of a chart must never be used to read off a centile, which opengrowth calculates. For showing the trend, an image can work, but it is easy to offset or scale one wrongly, so that some points land in the right place and others do not. Draw the lines and the points with the same vector graphics, and if you do use an image, make sure the right one is chosen for each chart, and test the scaling and offsets clinically, not only in code.

## Where is the code?

All of it is at [github.com/ohtf/opengrowth](https://github.com/ohtf/opengrowth): the engine, the command-line tool, the API, and this documentation.

## Why Rust?

So that one program can be every surface: a library, a command-line tool, and an HTTP API now, with an MCP server for AI agents and Python bindings to come, all from one engine, so a change is made once and tested once. A compiled program needs no runtime to install, which suits research environments and agents, and it answers faster on less hardware ([Benchmarks](../benchmarks.md)). Rust's guarantees about memory make whole classes of security fault impossible in safe code. The method and the numbers are `rcpchgrowth`'s, reproduced exactly.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [FAQs for Integrators](https://growth.rcpch.ac.uk/integrator/faqs-for-integrators/) and [FAQs for Developers](https://growth.rcpch.ac.uk/developer/faqs-for-developers/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It answers for a server you run yourself, where the original advised against running one.
