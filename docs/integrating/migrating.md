<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Moving from the RCPCH API

A system built for the RCPCH Digital Growth Charts API can call an opengrowth server with few changes: the requests and responses are the same, byte for byte outside provenance, apart from the [listed differences](../differences.md). This page is the checklist.

!!! warning "Not for clinical use yet"

    opengrowth is pre-alpha. For now, move development and test systems only ([Safety](../safety.md)).

## What changes

| | RCPCH API | opengrowth |
| --- | --- | --- |
| Address | `https://api.rcpch.ac.uk/growth/v1` | Your own server, such as `http://127.0.0.1:8000`, with no `/growth/v1` unless your gateway adds it |
| Key | `Subscription-Key: <key>`, checked by RCPCH's gateway | `Authorization: Bearer <key>`, if the server has a key |
| A request without a valid key | `404`, as RCPCH documents its gateway | `403`, with `{"detail":"Forbidden"}` |
| Provenance | Names `rcpch/rcpchgrowth-python` and `rcpch/digital-growth-charts-server` | Names `opengrowth` and `opengrowth-server`, and adds `reference_data`, the tables read; `growth_reference` is unchanged |
| Chart coordinates and mid-parental height | No provenance | Provenance first, as in every other result |
| `GET /` | The OpenAPI document | A health check; the document is [published here](../api/reference.md), and `opengrowth openapi` prints it |
| Request bodies | Any size | At most 2 MiB |
| Browsers, while a key is set | Preflight requests fail | Work |

## Steps

1. **Run a server.** [Integrating opengrowth](index.md) shows how, with a key, and how to build it so that results record the commit they came from.
2. **Change the address and the header** in your client. Nothing in the requests changes.
3. **Keep storing provenance.** New results name opengrowth's engine and server; results already stored keep the RCPCH provenance they came with, unchanged ([Persisting results](persisting-results.md)). If your system checks `calculation_engine.name` or `api_server.name`, allow opengrowth's names too.
4. **Store the extra provenance.** `reference_data` lists each table read, with its checksum: keep it with the rest.
5. **Test with your own requests.** Send the requests your system makes to both servers and compare the responses outside provenance. opengrowth's conformance suite does this for more than 34,000 captured requests, but your system's requests are the ones that matter to it.
6. **Treat it as a change of device.** The calculations are the same, but the software that makes them is not. Your organisation's clinical safety process, under DCB0160 in England, should cover the change before opengrowth is used for patient care, once it can be.
