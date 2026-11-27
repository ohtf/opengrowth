# Reference tables

opengrowth calculates from each reference's LMS tables, the same files that `rcpchgrowth` 4.6.5 publishes. They are compiled into the program, so it needs no data files to run.

## What is in use

```sh
opengrowth data status      # every table: embedded or imported, and its terms
opengrowth data manifest    # every table's source, file, checksum, and terms
```

`data status` also shows opengrowth's data directory, where imported tables are kept, and whether each table there is verified.

## Checksums

opengrowth records the SHA-256 checksum of every table file. Each table is checked against it the first time it is read: a table that does not match is refused, and opengrowth never substitutes another table or another reference. Every result's provenance lists the tables it read, with their checksums, and whether each was embedded or imported ([Persisting results](../integrating/persisting-results.md)).

## Importing tables

```sh
opengrowth data import DIR
```

checks every table file in `DIR` against its recorded checksum, and copies them into opengrowth's data directory, where they are used in place of the embedded tables. `DIR` uses the layout of `rcpchgrowth`'s `data_tables` directory, so an installed `rcpchgrowth` package's directory works as it is. One file that does not match stops the import before anything is written.

Because every file must match its recorded checksum, an import cannot change opengrowth's data: it supplies an identical copy, from a source you are entitled to use. To use a directory directly, without importing it, pass `--data-dir DIR`, or set `OPENGROWTH_REFERENCE_DATA_DIR`.

## UK90

The UK90 tables, which UK-WHO uses up to 42 weeks' gestation and from 4 years, belong to UK Research and Innovation. opengrowth includes them for research and development only, while a licence application to UKRI is pending ([Licences](../about/licences.md)). If UKRI refuses, they will come out of the program, and anyone who holds a licence for UK90 will import their own copy, for example from the `data_tables` directory of an installed `rcpchgrowth` 4.6.5:

```sh
opengrowth data import /path/to/rcpchgrowth/data_tables
opengrowth data status      # the UK90 tables now show as imported
```

That route already works, and is tested.
