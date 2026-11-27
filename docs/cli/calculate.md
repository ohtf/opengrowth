<!-- Written by s/cli-docs from the program's own help (src/cli/reference.rs). Do not edit by hand. -->

# opengrowth calculate

```text
Calculate SDS and centiles for one measurement

Usage: opengrowth calculate [OPTIONS] --sex <SEX> --birth-date <DATE> --observation-date <DATE> --gestation <WEEKS+DAYS> --measurement <METHOD> --value <VALUE>

Options:
      --format <FORMAT>
          Output format

          Possible values:
          - text: Aligned, human-readable text
          - json: Pretty-printed JSON, one document per invocation

          [default: text]

      --reference <REFERENCE>
          Growth reference

          [default: uk-who]

      --data-dir <DATA_DIR>
          Directory of reference tables to use, instead of the imported ones

          [env: OPENGROWTH_REFERENCE_DATA_DIR]

      --sex <SEX>
          Sex of the child

      --birth-date <DATE>
          Date of birth, YYYY-MM-DD

      --observation-date <DATE>
          Date the measurement was taken, YYYY-MM-DD

      --gestation <WEEKS+DAYS>
          Gestation at birth as WEEKS+DAYS, for example 40+0 or 32+4

      --measurement <METHOD>
          What was measured

      --value <VALUE>
          The measurement: cm for height and ofc, kg for weight, kg/m² for bmi

  -h, --help
          Print help (see a summary with '-h')
```
