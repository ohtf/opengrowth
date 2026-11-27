# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check that the differences page names every difference the tests enumerate (R53).

The conformance tests list each case where opengrowth deliberately answers
differently from upstream, in a `KNOWN_DIFFERENCES` constant, and the chart
tests in `STALE_CHART_FILES`. Each case's name must appear on
docs/differences.md, in code formatting, so no difference a test allows can
go undocumented.

    python3 -I qms/tools/differences.py          check; exit 1 on a missing case
    python3 -I qms/tools/differences.py --list   print every case, by test file
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAGE = ROOT / "docs" / "differences.md"
CONSTANTS = ("KNOWN_DIFFERENCES", "STALE_CHART_FILES")
LITERAL = r'"((?:[^"\\]|\\.)*)"'


def enumerated():
    """(test file, case name) for every difference the tests enumerate."""
    cases = []
    for path in sorted((ROOT / "tests").rglob("*.rs")):
        text = path.read_text()
        for constant in CONSTANTS:
            for match in re.finditer(rf"const {constant}\s*:\s*([^=]+)=\s*(.*?);\n", text, re.S):
                kind, body = match.group(1), match.group(2)
                if kind.strip().startswith("&[(") or kind.strip().startswith("("):
                    if kind.strip().startswith("("):
                        names = re.findall(rf"^\s*\(\s*{LITERAL}", body, re.M) or [re.findall(LITERAL, body)[0]]
                    else:
                        names = re.findall(rf"\(\s*{LITERAL}", body)
                else:
                    names = re.findall(LITERAL, body)
                cases += [(str(path.relative_to(ROOT)), name) for name in names]
    return cases


def main():
    cases = enumerated()
    if "--list" in sys.argv[1:]:
        for test, name in cases:
            print(f"{test}: {name}")
        return 0
    if not cases:
        print("no enumerated differences found: has the tests' format changed?", file=sys.stderr)
        return 1
    page = PAGE.read_text()
    missing = [(test, name) for test, name in cases if f"`{name}`" not in page]
    for test, name in missing:
        print(f"{PAGE.relative_to(ROOT)} does not name `{name}`, a difference {test} allows")
    if missing:
        return 1
    print(f"differences OK: {len(cases)} cases named on {PAGE.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
