<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Research

Research often needs the SDS and centiles of many children's measurements, in places where a hosted API cannot be reached: inside a trusted research environment or secure data environment, or offline. opengrowth is one program, with its reference tables compiled in, so it runs in such an environment without a network connection or a Python installation.

!!! warning "UK90 terms"

    UK-WHO uses the UK90 tables, which opengrowth includes for research and development only, while a licence application is pending ([Licences](../about/licences.md)). Check that your use falls within those terms.

## Calculating many measurements

The command-line tool calculates one measurement at a time, and `--format json` gives every value at full precision, with provenance:

```sh
opengrowth calculate --sex female --birth-date 2020-04-12 --observation-date 2028-06-12 \
  --gestation 40+0 --measurement height --value 115 --format json
```

For a dataset, run the API on the same computer and send it bulk calculations: up to 200 measurements of one child in each request, with an inline error in the place of any measurement that cannot be calculated. Nothing leaves the machine.

```sh
opengrowth serve &
curl --request POST 'http://127.0.0.1:8000/uk-who/bulk-calculation' \
  --header 'Content-Type: application/json' \
  --data-raw '{
    "measurement_method": "height",
    "birth_date": "2020-04-12",
    "sex": "female",
    "gestation_weeks": 40,
    "gestation_days": 0,
    "observations": [
      {"observation_date": "2024-04-12", "observation_value": 101.5},
      {"observation_date": "2028-06-12", "observation_value": 115}
    ]
  }'
```

Any language that can send HTTP requests can drive it from a notebook or a script, and in the [preliminary benchmarks](../benchmarks.md) one server on four CPUs answered about 100,000 single calculations a second.

## Still to come

- **Bulk input to the command-line tool**, from a file, without a server (`R14`).
- **Python bindings**, so that notebooks can call the engine directly, with an interface close to `rcpchgrowth`'s (`R18`).

Until then, `rcpchgrowth`, RCPCH's Python package, is the established way to calculate in Python, with notebooks of examples. opengrowth's results are identical to its version 4.6.5's, to the last bit, apart from the [listed differences](../differences.md), so results from the two can be compared directly.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [How to Use RCPCHgrowth in Research](https://growth.rcpch.ac.uk/researchers/introduction/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for opengrowth's command-line tool and API.
