<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Persisting results

!!! danger "Store the provenance with every calculated result"

    If your system stores a centile, SDS, or any other calculated result, store the complete `provenance` object unchanged with that result and the measurement it came from. Do not discard it, shorten its commit hashes, replace it with your configured version, or rewrite it when your software is upgraded.

    Provenance is to a result what the batch number is to a medicine: it names the exact code and data that produced it, so that if a calculation is ever found to be wrong in a particular version, the affected results can be found and calculated again.

## What opengrowth returns

Every calculation result, chart, and mid-parental height carries provenance like this:

```json
{
  "growth_reference": "uk-who",
  "calculation_engine": {
    "name": "opengrowth",
    "version": "0.1.0",
    "commit": "<40-character commit hash>"
  },
  "api_server": {
    "name": "opengrowth-server",
    "version": "0.1.0",
    "commit": "<40-character commit hash>"
  },
  "reference_data": [
    {
      "table": "uk90-child",
      "file": "uk90_child.json",
      "sha256": "4a6f4e9f258e57682224bdbb03dec151219bb52c5dee181b483745f60fe87441",
      "source": "embedded"
    }
  ]
}
```

- `growth_reference` is the reference the result was calculated against, one of `uk-who`, `trisomy-21`, `trisomy-21-aap`, `turners-syndrome`, `cdc`, and `who`: the same values as the RCPCH server's, so a chart can compare it with the reference it draws.
- `calculation_engine` and `api_server` name the code, its version, and the full commit it was built from. A build made without its commit recorded reports `unknown` ([Integrating opengrowth](index.md#build-it-so-that-results-name-their-code)).
- `reference_data` lists every table the calculation read, with the SHA-256 checksum of its file and whether it was compiled in (`embedded`) or imported (`imported`). It is opengrowth's addition: tables can be imported, so a result records exactly which data it used.

Together, they pin down the calculation as a unique device identifier pins down a device. They support traceability, and do not replace any formal identifier the law requires on a device's label.

## Why you must keep it

opengrowth keeps nothing: it calculates, answers, and forgets. Only your system can link a stored result to the code and data that produced it. If a serious calculation error were found, the provenance would let you:

1. Find the stored results by engine or server version and commit, or by table checksum.
2. Find which children's measurements may be affected.
3. Calculate them again with a corrected version.
4. Keep an audit trail linking each original result and its provenance to its replacement.
5. Support any corrective action, recall, clinical review, or communication that is needed.

Without stored provenance, an organisation may not be able to tell affected results from unaffected ones, and may have to review or recalculate far more.

## Storing it

The safest approach is to store the complete response unchanged. If your data model breaks the response up, it must still keep every provenance field with the same measurement and result:

- `growth_reference`
- `calculation_engine.name`, `.version`, and `.commit`
- `api_server.name`, `.version`, and `.commit`
- every entry of `reference_data`: `table`, `file`, `sha256`, and `source`

Your system should be able to search by version, by commit, and by table checksum. Store full commit hashes and checksums, as strings: shortened ones are not enough.

Treat provenance as a fixed historical record. An upgrade of your application, the chart component, or opengrowth must not rewrite the provenance of an existing result. If you calculate a measurement again, create a new result with its own provenance, and keep the original as your clinical record and audit policies require.

## Bulk calculations and older results

Each successful item of a bulk calculation carries its own provenance; store it with its measurement. An error object in a bulk response is not a result, and has none.

Results stored before your system had provenance may not contain it. Keep them as they are, as results whose provenance is unknown, and do not fill it in from your current configuration or from an assumption about what was deployed at the time. Your system should be able to tell such a result from one carrying recorded provenance. Results from the RCPCH server carry its own provenance, which names its own engine and server: keep it unchanged too.

## Charts and provenance

RCPCH's chart component, from version 7.6.0, compares each measurement's `provenance.growth_reference` with the reference the chart draws ([Chart component](chart-component.md)). Its policy, as RCPCH documents it:

| Measurement's provenance | What the chart does |
| --- | --- |
| Present, and matching | Draws it as usual. |
| Missing, from an older result | Draws it, treating the provenance as unknown. Component versions before this change was released may still show a warning. |
| Present, with a value the component does not recognise | Draws it, with a warning that its reference is unverified. |
| Present, with a recognised value that differs from the chart's | Leaves out the point, its tooltip, SDS, and centile, keeps the reference lines, and shows a prominent warning. |
| A mixture of older and matching results | Draws both; the mixture alone is not an error. |

The component's `reference` prop calls Turner syndrome `turner`, while provenance uses `turners-syndrome`; the component treats the two as the same only when comparing. Do not rewrite stored provenance, or change the server's response, to `turner`.

Changing the reference of a chart means calculating every measurement on it again. Relabelling an existing result is unsafe: the label would change, but not the calculation that produced the centile or SDS. Cache keys must include the reference, and a slow response for the previous reference must not replace results calculated for the new one. Code examples, fixtures, and new stored records should use the provenance a calculation actually returned: tests must not make up a matching value just to make a chart draw.

## Ready for a recall

Before an integration is used clinically, check that you can:

- Retrieve the provenance of any stored result.
- Find every result produced by a given engine version or commit, server version or commit, or table checksum.
- Calculate a result again from its original measurement without changing the original record.
- Record which new result replaces which original.
- Export a list for clinical safety review, without exposing more patient data than it needs.
- Reach the people in your organisation responsible for clinical safety, the system, and its operation.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Persisting API Results And Provenance](https://growth.rcpch.ac.uk/integrator/persisting-api-results/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](../about/acknowledgements.md#the-documentation). It is rewritten for opengrowth's provenance, which adds the tables read, and for a server you run yourself.
