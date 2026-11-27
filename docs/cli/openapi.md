<!-- Written by s/cli-docs from the program's own help (src/cli/reference.rs). Do not edit by hand. -->

# opengrowth openapi

```text
Print the OpenAPI document describing the HTTP API

Usage: opengrowth openapi [OPTIONS]

Options:
      --format <FORMAT>
          Output format

          Possible values:
          - text: Aligned, human-readable text
          - json: Pretty-printed JSON, one document per invocation

          [default: text]

      --server <URL>
          A URL the API is served at, for the document's list of servers. Repeat it for more than
          one. Without it, the list names `opengrowth serve` on this computer

      --data-dir <DATA_DIR>
          Directory of reference tables to use, instead of the imported ones

          [env: OPENGROWTH_REFERENCE_DATA_DIR]

  -h, --help
          Print help (see a summary with '-h')
```
