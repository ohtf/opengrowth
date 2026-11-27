<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Chart component

RCPCH publishes an open-source React component that draws growth charts from the API's responses: [digital-growth-charts-react-component-library](https://github.com/rcpch/digital-growth-charts-react-component-library). It was built to the [client specification](client-specification.md), and is the simplest way to draw charts that meet it.

!!! warning "The component's chart data need a licence from RCPCH"

    The component's code is under the MIT licence, but the chart line data bundled with it are copyright RCPCH and excluded from that licence: RCPCH licenses them to subscribers of its API. Using the component with opengrowth needs that licence, or an arrangement with RCPCH. Check its [licence](https://github.com/rcpch/digital-growth-charts-react-component-library) before you build on it.

## Using it with opengrowth

The component takes the calculation results as a prop and draws them as a familiar growth chart. opengrowth's results are the RCPCH server's, outside provenance. The component's check of provenance compares only `provenance.growth_reference` with the chart's reference, not the engine's name, and opengrowth gives `growth_reference` the same values as the RCPCH server. Nobody has yet tested the component with opengrowth end to end, so test your integration before relying on it.

Your application calls opengrowth, then passes each result to the component. The component draws one chart per instance: to show height and weight, use two. Its main props:

| Prop | What it is |
| --- | --- |
| `reference` | The reference to draw: `uk-who`, `turner`, `trisomy-21`, `trisomy-21-aap`, `cdc`, or `who`. Turner syndrome is `turner` here, but `turners-syndrome` in provenance. |
| `measurementMethod` | `height`, `weight`, `ofc`, or `bmi`. |
| `sex` | `male` or `female`. |
| `measurements` | The calculation results, by measurement method. |
| `chartType` | `centile` or `sds`. |
| `midParentalHeightData` | A mid-parental height result, to draw on height charts. |
| `clinicianFocus` | Whether the tooltips are written for clinicians, or more simply. |
| `theme`, `customThemeStyles` | Its look: monochrome by default, or one of RCPCH's other themes, or your own. |
| `logoVariant` | Where it shows the RCPCH logo, or an acknowledgement of RCPCH instead. |

The component's [type definitions](https://github.com/rcpch/digital-growth-charts-react-component-library/blob/live/src/RCPCHChart/RCPCHChart.types.ts) are the full and current list.

## Without React

The component is also published as a script, on [jsDelivr](https://www.jsdelivr.com/package/npm/@rcpch/digital-growth-charts-react-component-library) and unpkg, for pages that do not use React. It then needs React's own scripts first, and exposes `RCPCHGrowthCharts.render()`, which takes the same props and the `id` of the element to draw into. Load it with its integrity hash, which RCPCH publishes with each version as `build/sri-hash.txt`, so that the browser refuses any script but the one you chose: the component handles patient data.

## Versions

The component follows semantic versioning, with documentation for each release, and needs your application to be rebuilt to update. From version 7.6.0 it compares each measurement's provenance with the chart's reference ([Persisting results](persisting-results.md#charts-and-provenance)). It does not yet support small screens.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Using the Chart Component](https://growth.rcpch.ac.uk/integrator/using-the-chart-component/) and [React Component](https://growth.rcpch.ac.uk/products/react-component/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It keeps what an integrator needs to use the component with opengrowth, with the licence of its chart data, and leaves out RCPCH's demonstration site, Storybook, and services.
