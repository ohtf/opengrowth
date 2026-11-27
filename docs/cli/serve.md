<!-- Written by s/cli-docs from the program's own help (src/cli/reference.rs). Do not edit by hand. -->

# opengrowth serve

```text
Serve the HTTP API, compatible with the RCPCH Digital Growth Charts API

Usage: opengrowth serve [OPTIONS]

Options:
      --bind <BIND>
          Address to listen on

          [env: OPENGROWTH_BIND]
          [default: 127.0.0.1:8000]

      --format <FORMAT>
          Output format

          Possible values:
          - text: Aligned, human-readable text
          - json: Pretty-printed JSON, one document per invocation

          [default: text]

      --authorization-key <AUTHORIZATION_KEY>
          Require `Authorization: Bearer <KEY>` on every request except `/`. Read from the same
          variable as the upstream server

          [env: AUTHORIZATION_KEY]

      --data-dir <DATA_DIR>
          Directory of reference tables to use, instead of the imported ones

          [env: OPENGROWTH_REFERENCE_DATA_DIR]

      --workers <WORKERS>
          Number of worker threads (default: one per CPU)

      --cors-allow-origin <ORIGIN>
          Origins browsers may call the API from: `*` for any, or exact origins such as
          `https://app.example.org`. Repeat the option, or separate them with commas

          [env: OPENGROWTH_CORS_ALLOW_ORIGINS]
          [default: *]

      --no-cors
          Send no CORS headers, for a deployment whose gateway adds its own

  -h, --help
          Print help (see a summary with '-h')
```
