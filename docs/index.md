# opengrowth

opengrowth calculates the standard deviation score (SDS) and centile of a child's height or length, weight, head circumference, or body mass index against a published growth reference, and the coordinates of that reference's centile lines for drawing a growth chart. It is one program: a Rust library, a command-line tool, and an HTTP API that answers byte for byte as the RCPCH Digital Growth Charts API does, apart from a few [listed differences](differences.md).

!!! warning "Experimental: not for clinical use"

    opengrowth is pre-alpha software. Do not use it for anything clinical. Its quality management system is not yet in force, and the UK90 tables that UK-WHO depends on are included for research and development only, while a licence application to UK Research and Innovation is pending.

## What it calculates

- **References:** UK-WHO, WHO, CDC, UK Trisomy 21, AAP Trisomy 21, and Turner syndrome.
- **Measurements:** height or length, weight, head circumference, and body mass index (BMI).
- **Results:** SDS, centile, a sentence placing the measurement among the reference's centile lines, decimal and corrected ages, and chart coordinates.
- **Also:** mid-parental height, and series of fictional children for testing.

Every result carries provenance: the reference it was calculated against, the engine's version, and the checksum of every table it read.

## Where to start

- [Getting started](getting-started.md): install opengrowth, calculate a measurement, and run the API.
- [Differences from the RCPCH server](differences.md) and [benchmarks](benchmarks.md): how opengrowth compares with the server it reimplements.
- [Safety](safety.md): what opengrowth is for, and what it is not.

## Relationship to RCPCH

opengrowth is an independent reimplementation of `rcpchgrowth`, the growth engine the Royal College of Paediatrics and Child Health publishes under AGPL-3.0-or-later, and of the Digital Growth Charts API server built on it. It is not an RCPCH project, and is not affiliated with or endorsed by RCPCH. Pages adapted from the RCPCH Digital Growth Charts documentation say so and credit its authors.
