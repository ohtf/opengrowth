# Getting started

opengrowth is one program, `opengrowth`, with a short alias, `og`. There are no releases yet, so it is built from source.

## Install

You need [rustup](https://rustup.rs/), which installs the Rust version the repository pins the first time you build.

```sh
cargo install --locked --git https://github.com/ohtf/opengrowth
opengrowth            # lists the commands
```

The reference tables are compiled in, and each is checked against its recorded checksum when it is first read:

```sh
opengrowth data status
```

## Calculate a measurement

The command-line tool needs every input that changes the result, sex and gestation included, and assumes none of them:

```sh
opengrowth calculate --sex female --birth-date 2026-10-06 --observation-date 2026-11-17 \
  --gestation 27+3 --measurement weight --value 1.42
```

```text
UK-WHO: weight 1.42 kg, female, born 2026-10-06 at 27+3 weeks, measured 2026-11-17

                 decimal age        SDS   centile  table
chronological         0.1150    -8.0665      0.00  who-2006-infant
corrected            -0.1259    -1.6745      4.70  uk90-preterm

opengrowth 0.1.0; tables: uk90-preterm (sha256 fcd126301d8f…), who-2006-infant (sha256 c23dff79e6d8…)
Not for clinical use.
```

This baby was born at 27 weeks and 3 days and measured six weeks later. The corrected age is still before term, so the corrected values come from the UK90 preterm table, while the chronological ones come from WHO 2006. Add `--format json` for every value at full precision, with provenance.

`--reference` chooses another reference: `who`, `cdc`, `turners-syndrome`, `trisomy-21`, or `trisomy-21-aap`.

## Run the API

```sh
opengrowth serve      # http://127.0.0.1:8000; --bind changes the address
```

The API takes the same requests as the RCPCH Digital Growth Charts API and returns the same responses, so an existing client can be pointed at it unchanged:

```sh
curl -s -X POST http://127.0.0.1:8000/uk-who/calculation \
  -H 'content-type: application/json' \
  -d '{"birth_date": "2020-04-12", "observation_date": "2028-06-12", "observation_value": 115,
       "sex": "female", "gestation_weeks": 40, "gestation_days": 0, "measurement_method": "height"}' \
  | jq '{reference: .provenance.growth_reference, sds: .measurement_calculated_values.corrected_sds,
         centile: .measurement_calculated_values.corrected_centile,
         band: .measurement_calculated_values.corrected_centile_band}'
```

```json
{
  "reference": "uk-who",
  "sds": -2.406593606646068,
  "centile": 0.8051,
  "band": "This height measurement is between the 0.4th and 2nd centiles."
}
```

To require a token, set `AUTHORIZATION_KEY` before starting the server: every request except `/` must then send `Authorization: Bearer <key>`. Browsers on any site can call the API unless you name the sites allowed with `--cors-allow-origin`, or turn browser access off with `--no-cors`.
