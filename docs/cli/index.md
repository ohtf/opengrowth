<!-- Written by s/cli-docs from the program's own help (src/cli/reference.rs). Do not edit by hand. -->

# Command line

opengrowth is one program, `opengrowth`, with a short alias, `og`. Each page here is a command's own `--help`.

| Command | What it does |
| --- | --- |
| [`calculate`](calculate.md) | Calculate SDS and centiles for one measurement |
| [`data`](data.md) | Inspect and import reference tables |
| [`serve`](serve.md) | Serve the HTTP API, compatible with the RCPCH Digital Growth Charts API |
| [`openapi`](openapi.md) | Print the OpenAPI document describing the HTTP API |
| [`completions`](completions.md) | Print a shell completion script |

```text
Growth calculations for children: SDS and centiles against UK-WHO and other growth references.

Not for clinical use. See SAFETY.md.

Usage: opengrowth [OPTIONS] <COMMAND>

Commands:
  calculate    Calculate SDS and centiles for one measurement
  data         Inspect and import reference tables
  serve        Serve the HTTP API, compatible with the RCPCH Digital Growth Charts API
  openapi      Print the OpenAPI document describing the HTTP API
  completions  Print a shell completion script
  help         Print this message or the help of the given subcommand(s)

Options:
      --format <FORMAT>
          Output format

          Possible values:
          - text: Aligned, human-readable text
          - json: Pretty-printed JSON, one document per invocation

          [default: text]

      --data-dir <DATA_DIR>
          Directory of reference tables to use, instead of the imported ones

          [env: OPENGROWTH_REFERENCE_DATA_DIR]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
