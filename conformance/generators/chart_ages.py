# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Write src/chart/ages.rs: the chart age grids from rcpchgrowth's reference constants.

The grids are hundreds of literal ages; generating them from the pinned
engine with repr(), which round-trips exactly, avoids transcription errors.
Run by s/goldens chart_ages.
"""

from pathlib import Path

from rcpchgrowth.constants import reference_constants as rc

from _common import ROOT, environment

GRIDS = [
    ("UK_90_PRETERM_AGES", "UK90 preterm: 23 to 42 weeks gestation."),
    ("WHO_2006_UNDER_TWOS_AGES", "WHO 2006 under twos, as used for the UK-WHO infant and CDC infant segments."),
    ("UK_WHO_2006_OVER_TWOS_AGES", "UK-WHO child segment: WHO 2006, 2 to 4 years."),
    ("WHO_2006_OVER_TWOS_AGES", "WHO 2006 child segment, 2 to 5 years. Contains 3.083333333 twice (spec/standard.md section 6.2)."),
    ("WHO_2007_AGES", "WHO 2007, 5 to 19 years."),
    ("UK90_AGES", "UK90 child: 4 to 20 years."),
    ("CDC_TWO_TWENTY", "CDC child: 2 to 20 years."),
    ("CDC_TO_TWO_AGE", "CDC infant: 0 to 2 years."),
    ("CDC_TO_THREE_AGE", "CDC infant head circumference: 0 to 3 years."),
    ("TURNER_AGES", "Turner syndrome: 1 to 20 years."),
    ("TRISOMY_21_AGES", "UK Trisomy 21: 0 to 20 years."),
    ("TRISOMY_21_AAP_INFANT_AGES", "AAP Trisomy 21 infant: 1 month to 3 years."),
    ("TRISOMY_21_AAP_CHILD_AGES", "AAP Trisomy 21 child: 3 to 20 years."),
]

env = environment()
# The generated file's own header, not this script's.
# REUSE-IgnoreStart
header = [
    "// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors",
    "// SPDX-FileCopyrightText: Royal College of Paediatrics and Child Health",
    "// SPDX-License-Identifier: AGPL-3.0-or-later",
]
# REUSE-IgnoreEnd
lines = [
    *header,
    "",
    "//! Chart age grids: the ages at which centile lines are calculated.",
    "//!",
    f"//! Generated from `rcpchgrowth` {env['rcpchgrowth']} `constants/reference_constants.py` by",
    "//! `conformance/generators/chart_ages.py` (run `s/goldens chart_ages`).",
    "//! Do not edit by hand. Values are written with Python `repr`, so each",
    "//! parses to exactly the `f64` the upstream engine uses.",
    "",
]
for name, doc in GRIDS:
    values = getattr(rc, name)
    lines.append(f"/// {doc}")
    lines.append("#[rustfmt::skip]")
    lines.append(f"pub const {name}: [f64; {len(values)}] = [")
    for i in range(0, len(values), 6):
        chunk = ", ".join(repr(float(v)) for v in values[i : i + 6])
        lines.append(f"    {chunk},")
    lines.append("];")
    lines.append("")
out = ROOT / "src" / "chart" / "ages.rs"
out.write_text("\n".join(lines))
print(f"wrote {out.relative_to(ROOT)}")
