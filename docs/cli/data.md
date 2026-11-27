<!-- Written by s/cli-docs from the program's own help (src/cli/reference.rs). Do not edit by hand. -->

# opengrowth data

```text
Inspect and import reference tables

Usage: opengrowth data [OPTIONS] <COMMAND>

Commands:
  status    Show which reference tables are present and verified
  import    Verify reference tables in DIR and copy them into opengrowth's data directory
  manifest  List every table opengrowth knows, with its origin and terms
  help      Print this message or the help of the given subcommand(s)

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
```

## opengrowth data status

```text
Show which reference tables are present and verified

Usage: opengrowth data status [OPTIONS]

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
```

## opengrowth data import

```text
Verify reference tables in DIR and copy them into opengrowth's data directory.

DIR uses the rcpchgrowth `data_tables` layout; an installed rcpchgrowth package's `data_tables`
directory works as-is. Every file must match its published checksum; one bad file aborts the import
before anything is written.

Usage: opengrowth data import [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>
          Directory to import from

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
```

## opengrowth data manifest

```text
List every table opengrowth knows, with its origin and terms

Usage: opengrowth data manifest [OPTIONS]

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
```
