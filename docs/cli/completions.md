<!-- Written by s/cli-docs from the program's own help (src/cli/reference.rs). Do not edit by hand. -->

# opengrowth completions

```text
Print a shell completion script

Usage: opengrowth completions [OPTIONS] [SHELL]
       opengrowth completions <COMMAND>

Commands:
  install  Install completions for the current user, for both `opengrowth` and `og`
  help     Print this message or the help of the given subcommand(s)

Arguments:
  [SHELL]
          Shell to generate completions for

          [possible values: bash, elvish, fish, powershell, zsh]

Options:
      --format <FORMAT>
          Output format

          Possible values:
          - text: Aligned, human-readable text
          - json: Pretty-printed JSON, one document per invocation

          [default: text]

  -d, --dir <DIR>
          Write the scripts to this directory instead of printing to stdout

      --data-dir <DATA_DIR>
          Directory of reference tables to use, instead of the imported ones

          [env: OPENGROWTH_REFERENCE_DATA_DIR]

  -h, --help
          Print help (see a summary with '-h')
```

## opengrowth completions install

```text
Install completions for the current user, for both `opengrowth` and `og`

Usage: opengrowth completions install [OPTIONS]

Options:
      --format <FORMAT>
          Output format

          Possible values:
          - text: Aligned, human-readable text
          - json: Pretty-printed JSON, one document per invocation

          [default: text]

      --shell <SHELL>
          Shell to install for. Read from $SHELL when omitted

          [possible values: bash, elvish, fish, powershell, zsh]

  -d, --dir <DIR>
          Directory to install to. Defaults to the shell's per-user location

      --data-dir <DATA_DIR>
          Directory of reference tables to use, instead of the imported ones

          [env: OPENGROWTH_REFERENCE_DATA_DIR]

  -h, --help
          Print help (see a summary with '-h')
```
