# CLARITY achondroplasia growth reference (2021)

The source tables for an achondroplasia growth reference, kept unmodified. They are not yet a growth reference in opengrowth (`R47` in [spec/roadmap.md](../../../spec/roadmap.md)): nothing reads them.

Hoover-Fong JE, Schulze KJ, Alade AY, et al. Growth in achondroplasia including stature, weight, weight-for-height and head circumference from CLARITY: achondroplasia natural history study - a multi-center retrospective cohort study of achondroplasia in the US. *Orphanet J Rare Dis* 2021;16:522. [doi:10.1186/s13023-021-02141-4](https://doi.org/10.1186/s13023-021-02141-4), PMC8697459.

## Files

| File | Supplement | Contents |
| --- | --- | --- |
| `13023_2021_2141_MOESM2_ESM.csv` | Additional file 2, Table S1 | Length or height (cm), weight (kg), and head circumference (cm), by sex (`0` male, `1` female) and month of age from 0 to 239. For each measurement: the number of measurements (`n`), the mean, an SD, and separate SDs below and above the mean (`sd lower`, `sd upper`). Head circumference is `NA` from 60 months. |
| `13023_2021_2141_MOESM3_ESM.csv` | Additional file 3, Table S2 | Weight-for-length or height (kg), by sex and 1 cm of length or height: the number of measurements, the mean, and the SDs below and above it. opengrowth has no weight-for-height measurement, so nothing uses it. |

Both were downloaded on 2026-11-19 from the publisher, `https://static-content.springer.com/esm/art%3A10.1186%2Fs13023-021-02141-4/MediaObjects/<file>`. PubMed Central serves a "Preparing to download" page to scripts instead of the file, so its copy was not used. `SHA256SUMS` records each file's checksum, and `tests/data_sources.rs` checks them. Table S1 has Windows line endings and Table S2 a UTF-8 byte-order mark; both are kept as published.

## Terms

The article is published under CC BY 4.0, and its rights statement, on both the publisher's page and PubMed Central, says: "The Creative Commons Public Domain Dedication waiver (http://creativecommons.org/publicdomain/zero/1.0/) applies to the data made available in this article, unless otherwise stated in a credit line to the data." Neither file has a credit line: apart from the column headers, every cell is a number, `NA`, an interval label such as `[0,1)`, or empty: Table S2 has no values for girls taller than 134 cm, as the paper says. The files are therefore CC0-1.0 (`REUSE.toml`).

## What the paper says about using them

These are the points a growth reference built on the tables has to settle, quoted or summarised from the paper's methods:

- **Z-scores use two SDs.** For weight, "Z-scores were calculated using age- and sex-specific means and (upper or lower, depending on whether the individual weight value was larger or smaller that its age and sex-specific mean weight) standard deviations". That is a two-piece normal model, not LMS. Table S1 gives both SDs for height and head circumference too, but the methods describe them only for weight.
- **Ages covered.** The curves are for length or height and weight "from birth through 18 years", and head circumference "birth through 5 years". Table S1 continues to 239 months, but height has fewer than five measurements in most months after about 15 years, and rows beyond 216 months (18 years) are outside the paper's range.
- **What each row's age means.** Values were estimated "at each month of age", using measurements within a window around it (half a month either side in the first year), but the file labels each row as an interval, `[k, k+1)`.
- **Who is in it.** Children whose birth measurements were above -2 SD, as a surrogate for term gestation; children of unknown gestation only after 2 years; and no data after limb lengthening, growth hormone, or a clinical trial of a drug began.
- **Centile curves.** The paper's charts show smoothed empirical 5th, 25th, 50th, 75th, and 95th percentiles, which will not equal the mean plus or minus 1.645 SD exactly.
