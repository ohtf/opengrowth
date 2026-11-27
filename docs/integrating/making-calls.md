<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Making calls

Every operation is a `POST` of a JSON body to a path under your server's address. The paths are relative to that address: on a server started with `opengrowth serve`, a UK-WHO calculation is `http://127.0.0.1:8000/uk-who/calculation`. The [API reference](../api/reference.md) gives every operation's request and response, and can send requests to a server on your own computer.

!!! note "The base address"

    opengrowth serves its paths at the root of its address, as the RCPCH server does. The RCPCH service's `/growth/v1` prefix comes from its gateway; a gateway in front of your own server can add a prefix in the same way.

## With cURL

[cURL](https://curl.se/) sends requests from the command line. This request sends only the parameters a result needs:

```bash
curl --request POST 'http://127.0.0.1:8000/uk-who/calculation' \
  --header 'Authorization: Bearer YOUR_KEY' \
  --header 'Content-Type: application/json' \
  --data-raw '{
    "birth_date": "2020-04-12",
    "observation_date": "2028-06-12",
    "observation_value": 115,
    "sex": "female",
    "gestation_weeks": 40,
    "gestation_days": 0,
    "measurement_method": "height"
  }'
```

Leave out the `Authorization` header if the server has no key. On Windows, Command Prompt needs `^` to continue a line and double quotes, escaped inside the JSON; in PowerShell, call `curl.exe`, since `curl` there is another command:

```bat
curl --request POST "http://127.0.0.1:8000/uk-who/calculation" ^
  --header "Authorization: Bearer YOUR_KEY" ^
  --header "Content-Type: application/json" ^
  --data-raw "{\"birth_date\": \"2020-04-12\", \"observation_date\": \"2028-06-12\", \"observation_value\": 115, \"sex\": \"female\", \"gestation_weeks\": 40, \"gestation_days\": 0, \"measurement_method\": \"height\"}"
```

The response is one long line of JSON. [jq](https://jqlang.org/) lays it out, or picks fields from it, if you pipe the response into it:

```bash
curl ... | jq '.measurement_calculated_values'
```

!!! danger "Do not discard the provenance"

    Every result includes a `provenance` object: the growth reference, the engine and server versions and the commits they were built from, and every table the calculation read. If you store the result, store this object with it, unchanged ([Persisting results](persisting-results.md)).

### Dates

Dates in requests and responses are written `YYYY-MM-DD`, without a time, which is what RCPCH's chart component expects. If your system converts the response, keep the dates in that form. Store the whole response, provenance included, so that each measurement needs only one call and its result stays traceable.

## The parameters

A calculation needs these:

| Parameter | What it is |
| --- | --- |
| `birth_date` | The child's date of birth, `YYYY-MM-DD`. |
| `observation_date` | The date of the measurement, `YYYY-MM-DD`. |
| `observation_value` | The measurement: height or length in cm, weight in kg, BMI in kg/m², or head circumference in cm. |
| `sex` | `male` or `female`. |
| `gestation_weeks` | Completed weeks of gestation at birth: `40` for a baby born on their due date. |
| `gestation_days` | Extra days of gestation at birth, 0 to 6. |
| `measurement_method` | `height`, `weight`, `bmi`, or `ofc` (head circumference). |

The API accepts a request without the gestation, as the RCPCH server does, and calculates as if the baby had been born at 40+0. That is right only for a baby born on their due date, so send the gestation every time.

!!! warning "Gestation is fixed at birth: never change it between measurements"

    `gestation_weeks` and `gestation_days` say how far through pregnancy the baby was when born: 35 weeks and 0 days for a baby born five weeks early, 40 and 0 for one born on their due date. Gestation does not change for the rest of the child's life, so every measurement of the same child must send the same values.

    The gestation sets the child's corrected age ([Ages and gestation](../background/ages.md)). If measurements of the same child are sent with different gestations, each is corrected differently, and plotted together they show a growth pattern that is not the child's. Store the gestation once, in the child's record, and send it with every measurement.

Everything else is optional, and most measurements need none of it:

- **Bone age.** `bone_age`, `bone_age_centile`, `bone_age_sds`, `bone_age_text`, and `bone_age_type` are returned with the result, so that a chart can plot the bone age beside the measurement. opengrowth does not calculate a bone age.
- **Events.** `events_text` attaches notes to a measurement, such as "Growth hormone start". They are returned with the result for a chart to show as markers, and do not change any calculation.
- **Other references.** The same request goes to another reference by changing the first part of the path, for example `/who/calculation` or `/trisomy-21/calculation`. Turner syndrome is `/turner` ([Growth references](../background/growth-references.md)).

Here is the earlier request with bone age and events:

```bash
curl --request POST 'http://127.0.0.1:8000/uk-who/calculation' \
  --header 'Authorization: Bearer YOUR_KEY' \
  --header 'Content-Type: application/json' \
  --data-raw '{
    "birth_date": "2020-04-12",
    "observation_date": "2028-06-12",
    "observation_value": 115,
    "sex": "female",
    "gestation_weeks": 40,
    "gestation_days": 0,
    "measurement_method": "height",
    "bone_age": 10,
    "bone_age_centile": 98,
    "bone_age_sds": 2.0,
    "bone_age_text": "This bone age is advanced",
    "bone_age_type": "greulich-pyle",
    "events_text": ["Growth hormone start"]
  }'
```

## When a request fails

| Status | When |
| --- | --- |
| `403` | The server has a key, and the request has no `Authorization: Bearer` header with it. The body is `{"detail":"Forbidden"}`. |
| `404`, `405` | The path does not exist, or does not take that method. |
| `413` | The body is larger than 2 MiB. |
| `422` | The request is not valid, or opengrowth cannot calculate it: `detail` says why, field by field, in the RCPCH server's words. |

In a bulk calculation, a malformed request is refused whole, with `422`, but an observation that cannot be calculated becomes an error object in its place in `results`, and the others are still calculated.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Making calls to the Digital Growth Charts API](https://growth.rcpch.ac.uk/integrator/making-api-calls/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for your own server and its bearer token, with the API's errors added; the Postman collection, which targets RCPCH's service, is left out.
