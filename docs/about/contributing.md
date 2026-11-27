<!-- SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors -->
<!-- SPDX-FileCopyrightText: 2020-2026 Royal College of Paediatrics and Child Health and the Digital Growth Charts documentation contributors -->
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Contributing

opengrowth is developed in the open, at [github.com/ohtf/opengrowth](https://github.com/ohtf/opengrowth). [CONTRIBUTING.md](https://github.com/ohtf/opengrowth/blob/main/CONTRIBUTING.md) is the full guide; this page summarises it, and explains how to write these pages.

## The essentials

- Sign off every commit under the [Developer Certificate of Origin](https://openhealthtechnology.org/community/dco/), with `git commit -s`. CI refuses a commit without it.
- Report anything that could affect a calculated value as an issue labelled `hazard`, and security vulnerabilities privately, as [SECURITY.md](https://github.com/ohtf/opengrowth/blob/main/SECURITY.md) describes.
- A change to a calculated number, a clinical sentence, a reference table, the API, or what opengrowth says it is for needs evidence beyond passing tests. CONTRIBUTING.md lists what to bring for each.

## Writing these pages

The site is built by [Zensical](https://zensical.org/) from the Markdown in `docs/`, as `mkdocs.yml` configures it. To preview it as you edit, set it up once, then serve it:

```sh
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt
s/docs
```

`s/docs` opens the site on the first free port from 8000 and rebuilds it on every save.

To add a page, create it under `docs/` and add it to `nav:` in `mkdocs.yml`. A page's first heading is its title. Before pushing, run what CI runs:

```sh
s/build-docs --strict   # fails on any warning, such as a broken link
s/check-docs-nav        # fails on a page missing from the nav
s/check-wording         # fails on wording outside the intended purpose
```

### Style

- British English, plain and concrete: say what opengrowth calculates, with an example where one helps.
- One paragraph per line, not wrapped, and hyphens rather than em dashes.
- [qms/intended-purpose.md](https://github.com/ohtf/opengrowth/blob/main/qms/intended-purpose.md) reserves some words, such as diagnose, interpret, and refer, which may describe only what opengrowth does not do. `s/check-wording` enforces that; where a use does not describe opengrowth, add an exception with its reason to `qms/tools/wording.py`.

### Pages adapted from the Digital Growth Charts documentation

Much of this site adapts the RCPCH Digital Growth Charts documentation, under its CC BY-SA 4.0 licence. An adapted page carries SPDX headers crediting RCPCH and the documentation's contributors as well as ours, and ends with a note naming the page it adapts and what changed. It uses no RCPCH branding. [spec/docs-port.md](https://github.com/ohtf/opengrowth/blob/main/spec/docs-port.md) records the rules, and what became of every upstream page.

Contributions to these pages are licensed CC BY-SA 4.0.

!!! info "Adapted from the RCPCH Digital Growth Charts documentation"

    This page adapts [Contributing](https://growth.rcpch.ac.uk/developer/contributing/) and [Writing dGC Documentation](https://growth.rcpch.ac.uk/developer/writing-documentation/), CC BY-SA 4.0, copyright 2020-2026 the Royal College of Paediatrics and Child Health and the documentation's [contributors](acknowledgements.md#the-documentation). It is rewritten for opengrowth's process and this site's tools.
