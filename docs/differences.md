# Differences from the RCPCH server

opengrowth answers every request that the RCPCH Digital Growth Charts API answers with the same status and the same bytes, apart from the response's `provenance` and the differences on this page. Each difference is either designed, where opengrowth has to say what it is or records more, or a fix, where the RCPCH server fails with a `500 Internal Server Error` and opengrowth gives a defined answer instead. The conformance tests check that every difference still holds exactly as described, so none can appear or change unnoticed; each is named below with its test cases.

The comparison is with the RCPCH server at commit [29bb4e9](https://github.com/rcpch/digital-growth-charts-server/commit/29bb4e9e9d93a850a63d5233b927472f808fb9a8) (version 5.1.0, 24 September 2026), with `rcpchgrowth` 4.6.5.

## Designed differences

### Provenance names opengrowth

Every result's `provenance` names the engine and server that produced it: `calculation_engine.name` is `opengrowth`, not `rcpch/rcpchgrowth-python`, and `api_server.name` is `opengrowth-server`. While both servers are in use, a result has to say which one calculated it, or a difference between them could not be traced to either. `growth_reference`, which a chart component compares with the chart it draws, is the same.

### Provenance lists the tables read

`provenance.reference_data` lists every reference table the calculation read: its name, the file it came from, the file's SHA-256 checksum, and whether it was compiled in or imported. Tables can be imported at run time, so a result records exactly which data it used. A client that does not know the field ignores it.

### Charts and mid-parental height carry provenance

The RCPCH server's `chart-coordinates` and `utilities/mid-parental-height` responses have no `provenance`. opengrowth's have it first, as its measurement results do, so a chart's lines name the reference they were calculated from, and a client can confirm that a chart and the results plotted on it come from the same reference. A client that does not know the field ignores it.

### `/` is a health check

The RCPCH server answers `/` with its OpenAPI document. opengrowth answers with its name and version, and publishes its document with this documentation instead ([API reference](api/reference.md)).

### Request bodies are limited to 2 MiB

The RCPCH server reads a request body of any size. opengrowth reads at most 2 MiB, and answers a larger body with `413 Payload Too Large`, so that no client can make it hold an unbounded body in memory. The largest valid request, a bulk calculation of 200 observations, is about 12 KB. Case: `a request body over 2 MiB`.

### Browsers can call it when a token is set

CORS controls which websites' pages may call the API from a browser. The RCPCH server allows every site together with credentials, which browsers do not accept together, and it checks the token before it answers a browser's preflight request, so while a token is set no page on another site can call it. opengrowth allows every site without credentials, and answers preflights before it checks the token, so a browser client works with a token set. In detail, opengrowth never sends `access-control-allow-credentials`, answers preflights with `*` rather than repeating the caller's origin, and, with a token set, answers preflights and adds `access-control-allow-origin: *` to its 403 responses. `--cors-allow-origin` limits the sites allowed, and `--no-cors` stops calls from other sites' pages. `tests/http_cors.rs` checks the RCPCH server differs in exactly these ways.

### Named-format charts are calculated

For a named collection of centile lines, such as the Cole nine-centile format, the RCPCH server sends chart files it last regenerated on 11 January 2025. For UK-WHO, WHO, and CDC these no longer match its own engine: points on the UK-WHO girls' Cole nine-centile height chart, for example, differ by up to 0.0183 cm. opengrowth calculates every chart with the engine that calculates its measurements, so the lines agree with the results plotted on them. The Turner and Trisomy 21 files still match, so those charts are identical. Case: `every named-format chart`, in each chart test.

## Fixes where the RCPCH server fails

For each of these requests the RCPCH server answers with a bare `500 Internal Server Error`, and opengrowth gives the answer its route gives other invalid requests.

| Request | RCPCH server | opengrowth | Cases |
| --- | --- | --- | --- |
| A `birth_date` that is a number or `null`, to any reference's calculation | 500, from a Python `TypeError` | 422 `value_error`, with the same `TypeError` text | `validation birth date not string`, `validation birth date null`, in every reference's calculation test |
| A CDC height of zero | 500, from a division by zero | 422: less than -8 SD | `app error observation zero` |
| UK-WHO fictional child data starting before birth | 500, from a measurement with no corrected age | 422: not possible to create the data | `start before birth` |
| CDC BMI fictional child data at an SDS whose centile rounds to 100 | 500, from an infinite measurement | 422: not possible to create the data | `sds 16 bmi` |
| CDC mid-parental height with a negative height for a parent | 500, from a complex SDS | 422: less than -8 SD | `cdc error negative heights` |
| UK Trisomy 21 chart coordinates for a centile of 0, 100, or more | 500, from an infinite SDS in the response | 422, as for the other references | `error centile 0`, `error centile 100`, `error centile 150` |
| A body labelled as something other than JSON, whose bytes are not UTF-8 | 500, from its error handler | 422, showing the body with replacement characters | the six cases below |

The six cases of the last row are `calculation, UTF-16 with a byte-order mark, Content-Type 'text/plain'`, `calculation, UTF-16LE without a byte-order mark, Content-Type 'text/plain'`, `calculation, UTF-16BE without a byte-order mark, Content-Type 'text/plain'`, `calculation, UTF-32 with a byte-order mark, Content-Type 'text/plain'`, `calculation, UTF-32LE without a byte-order mark, Content-Type 'text/plain'`, and `calculation, Latin-1, which is not UTF-8, Content-Type 'text/plain'`.

Two more inputs, not in the captured requests, are refused where the RCPCH server accepts them: `NaN`, `Infinity`, and lone surrogates in JSON, which CPython's parser accepts and the server then fails on or returns, are refused as `json_invalid`; and a number sent as the string `"nan"` is refused as `float_parsing`, where the RCPCH server fails with a 500.

## For the library

These concern the engine called directly, not the API, which refuses such requests before calculating:

- **An observation dated before birth.** `rcpchgrowth` records Python's `TypeError` text as the `observation_value_error`; opengrowth records none. Case: `observation before birth`.
- **An age outside a table's rows.** `rcpchgrowth` raises an `IndexError` or a SciPy `ValueError`; opengrowth returns a typed error. Neither happens for whole-day ages.

## RCPCH behaviour that opengrowth keeps

Some of the RCPCH server's behaviour looks unintended but changes calculated values, so opengrowth reproduces it exactly until clinicians have reviewed it:

- UK Trisomy 21 BMI data run to 18.92 years, but requests above 18.82 years are refused, so the last two rows for each sex are never used.
- The WHO chart age grid from 2 to 5 years lists 3.083 years twice, so it is out of order at that point.
- The Cole nine-centile collection is drawn at SDS rounded to two-thirds only for UK-WHO; for the other references each line is at its centile's exact SDS.
- WHO SDS lookups interpolate linearly, but WHO's measurement at an SDS does not, so between table ages the two directions are not exact inverses.
- A `null` `gestation_weeks` or `gestation_days` gets a 422 whose message is Python `TypeError` text, which opengrowth repeats until a clearer message is agreed.

The full record, with the reasoning for each, is in section 6 of the [standard](https://github.com/ohtf/opengrowth/blob/main/spec/standard.md), and the evidence is in the [conformance suite](https://github.com/ohtf/opengrowth/blob/main/conformance/README.md).
