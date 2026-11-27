# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Check user-facing prose against the wording rules in qms/intended-purpose.md.

That statement reserves some words: they may describe only what opengrowth
does not do. This finds each of them in the documentation site and the
README, outside code and link targets, and fails unless its sentence is a
negation or an exception below names it with a reason (R44, R55).

    python3 -I qms/tools/wording.py     list every finding; exit 1 if any
"""

import fnmatch
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STATEMENT = ROOT / "qms" / "intended-purpose.md"
FILES = ("README.md", "docs/**/*.md", "docs/api/openapi.json")
# Pages whose code blocks are prose a user reads: the command-line reference
# holds each command's help in a code block (R56).
CODE_IS_PROSE = ("docs/cli/*.md",)

# The forms of each reserved word that carry its meaning. Look-alikes with
# another meaning are left out: "reference" for refer, "identifier" for
# identify, and "screenshot" for screen.
FORMS = {
    "diagnose": "diagnose|diagnoses|diagnosed|diagnosing|diagnosis|diagnostic|diagnostics",
    "detect": "detect|detects|detected|detecting|detection",
    "identify": "identify|identifies|identified|identifying|identification",
    "screen": "screen|screens|screened|screening",
    "monitor": "monitor|monitors|monitored|monitoring",
    "assess": "assess|assesses|assessed|assessing|assessment|assessments",
    "interpret": "interpret|interprets|interpreted|interpreting|interpretation|interpretations",
    "decide": "decide|decides|decided|deciding|decision|decisions",
    "predict": "predict|predicts|predicted|predicting|prediction|predictions|predictive",
    "alert": "alert|alerts|alerted|alerting",
    "flag": "flag|flags|flagged|flagging",
    "recommend": "recommend|recommends|recommended|recommending|recommendation|recommendations",
    "refer": "refer|refers|referred|referring|referral|referrals",
    "risk": "risk|risks",
}

# A sentence that says what opengrowth does not do may use the words.
NEGATION = re.compile(r"\b(not|no|never|nor|neither|without|cannot|rather than|instead of)\b|n't\b", re.I)

# Uses allowed for a stated reason: (files, pattern in the sentence, reason).
# Each must match something, so the list cannot go stale.
EXCEPTIONS = [
    ("docs/about/contributing.md", r"reserves some words", "Names some of the reserved words in order to explain the rule."),
    ("docs/background/papers.md", r"^\|", "The titles of published papers, in the tables, quoted as published: they describe the papers, not opengrowth."),
]


def reserved_words():
    """The reserved words, read from the statement's "Do not use" rule."""
    text = STATEMENT.read_text()
    line = next(l for l in text.splitlines() if l.startswith("- **Do not use**"))
    listed = line.split(":", 1)[1].split(".", 1)[0]
    return [w.strip() for w in listed.split(",")]


def prose_lines(path, code_is_prose=False):
    """(line number, text) for each line of prose, with code removed unless
    `code_is_prose`."""
    fence = None
    for number, line in enumerate(path.read_text().splitlines(), 1):
        stripped = line.strip()
        marker = re.match(r"(`{3,}|~{3,})", stripped)
        if marker:
            if fence is None:
                fence = marker.group(1)[0] * 3
            elif stripped.startswith(fence):
                fence = None
            continue
        if fence is not None and not code_is_prose:
            continue
        text = re.sub(r"`[^`]*`", " ", line)
        text = re.sub(r"\]\([^)]*\)", "]", text)
        text = re.sub(r"<!--.*?-->", " ", text)
        text = re.sub(r"<[^>]+>", " ", text)
        yield number, text


def json_prose(path):
    """(location, text) for each description, summary, and title in a JSON
    document such as the OpenAPI document."""
    import json

    def walk(node, where):
        if isinstance(node, dict):
            for key, value in node.items():
                if key in ("description", "summary", "title") and isinstance(value, str):
                    yield f"{where}/{key}", value
                yield from walk(value, f"{where}/{key}")
        elif isinstance(node, list):
            for index, value in enumerate(node):
                yield from walk(value, f"{where}/{index}")

    yield from walk(json.loads(path.read_text()), "")


def sentences(text):
    return re.split(r"(?<=[.!?])\s+", text)


def main():
    words = reserved_words()
    if sorted(words) != sorted(FORMS):
        print(f"the reserved words in {STATEMENT.relative_to(ROOT)} have changed: update FORMS in {Path(__file__).name}", file=sys.stderr)
        print(f"  statement: {', '.join(words)}", file=sys.stderr)
        return 1
    pattern = re.compile(r"\b(" + "|".join(FORMS.values()) + r")\b", re.I)
    paths = sorted({p for glob in FILES for p in ROOT.glob(glob)})
    used = set()
    findings = []
    for path in paths:
        relative = str(path.relative_to(ROOT))
        code_is_prose = any(fnmatch.fnmatch(relative, pattern) for pattern in CODE_IS_PROSE)
        lines = json_prose(path) if path.suffix == ".json" else prose_lines(path, code_is_prose)
        for number, text in lines:
            for sentence in sentences(text):
                for match in pattern.finditer(sentence):
                    if NEGATION.search(sentence[: match.start()]):
                        continue
                    excused = False
                    for index, (files, phrase, _reason) in enumerate(EXCEPTIONS):
                        if fnmatch.fnmatch(relative, files) and re.search(phrase, sentence, re.I):
                            used.add(index)
                            excused = True
                    if not excused:
                        findings.append(f"{relative}:{number}: \"{match.group(0)}\" in: {sentence.strip()[:160]}")
    for index, (files, phrase, _reason) in enumerate(EXCEPTIONS):
        if index not in used:
            findings.append(f"exception matches nothing, remove it: {files} {phrase!r}")
    for finding in findings:
        print(finding)
    sys.stdout.flush()
    if findings:
        print(f"\n{len(findings)} use(s) of words qms/intended-purpose.md reserves. Rephrase to say what opengrowth calculates, or, where a use does not describe opengrowth, add an exception with its reason to {Path(__file__).relative_to(ROOT)}.", file=sys.stderr)
        return 1
    print(f"wording OK: {len(paths)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
