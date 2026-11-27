# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Render benchmark results as Markdown, and publish them on the site (R54).

run.py writes a run's report.md with this, and s/bench-page publishes a run
on docs/benchmarks.md, between its two marker comments, from the copy of the
run's results kept in docs/benchmarks.json.

    python3 -I bench/http/report.py                 render the page from docs/benchmarks.json
    python3 -I bench/http/report.py RESULTS.json    publish a run from s/bench, then render
    python3 -I bench/http/report.py --check         fail if the page is not current, as CI does
"""

import datetime
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAGE = ROOT / "docs" / "benchmarks.md"
DATA = ROOT / "docs" / "benchmarks.json"
START = "<!-- Written by s/bench-page from docs/benchmarks.json, as far as the closing comment. Do not edit by hand. -->"
END = "<!-- End of s/bench-page. -->"

FORMAT = 2
SERVERS = ("Python", "Rust")
NAMES = {"Python": "RCPCH server", "Rust": "opengrowth"}


def number(value):
    """Three significant figures, or so: 0.43, 12.9, 221, 1,139."""
    if value is None:
        return "-"
    if value >= 100:
        return f"{value:,.0f}"
    if value >= 10:
        return f"{value:.1f}"
    return f"{value:.2f}"


def ms(value):
    """A time given in milliseconds, in µs below 1 ms."""
    if value is None:
        return "-"
    return f"{number(value * 1000)} µs" if value < 1 else f"{number(value)} ms"


def nanoseconds(value):
    if value < 1_000:
        return f"{number(value)} ns"
    if value < 1_000_000:
        return f"{number(value / 1_000)} µs"
    return f"{number(value / 1_000_000)} ms"


def rate(value):
    return "-" if value is None else f"{value:,.0f}/s"


def megabytes(kib):
    return f"{number(kib * 1024 / 1e6)} MB"


def ratio(slower, faster, word):
    """How many times better opengrowth is: "68× faster"."""
    if not slower or not faster:
        return "-"
    times = slower / faster
    return f"{times:,.0f}× {word}" if times >= 10 else f"{times:.1f}× {word}"


def percent(value):
    return f"{value * 100:.0f}%"


def cpu_use(step, workers):
    """The share of the server's CPUs it kept busy during an oha run."""
    return step["server_cpu_s"] / (step["seconds"] * workers)


def short(commit):
    return commit[:7]


def date(results):
    when = datetime.datetime.fromisoformat(results["environment"]["date"])
    return f"{when.day} {when:%B %Y}"


def check(results):
    if results.get("format") != FORMAT:
        raise SystemExit(f"these results are in format {results.get('format')}, and this report reads format {FORMAT}: run s/bench again")
    if not results["correct"]:
        raise SystemExit(f"the servers' answers differed for {', '.join(results['mismatched'])}: a fast wrong answer scores zero, so this run is not published")


def render(results, raw):
    """The results as Markdown, linking to the raw results at `raw`."""
    check(results)
    env = results["environment"]
    workers = env["workers"]
    servers = results["servers"]
    round1, round2, round3 = results["round1"], results["round2"], results["round3"]
    lines = [f"## Results, {date(results)}", ""]
    lines.append(f"Before any timing, both servers gave the same answers, outside provenance, to every request below. Every figure is from a single run; the [raw results]({raw}) have them all, with each request's body.")
    if env["quick"]:
        lines += ["", "This was a quick run, with steps of a few seconds, so its figures are rougher still."]

    lines += ["", "| | RCPCH server | opengrowth | opengrowth is |", "| --- | ---: | ---: | ---: |"]
    for label in results["headline"]:
        by = round1[label]
        lines.append(f"| {label}, median | {ms(by['Python']['p50_ms'])} | {ms(by['Rust']['p50_ms'])} | {ratio(by['Python']['p50_ms'], by['Rust']['p50_ms'], 'faster')} |")
    best = {name: round2[name]["max_sustained"] for name in SERVERS}
    lines.append(f"| Highest rate with p99 under 100 ms | {rate(best['Python'])} | {rate(best['Rust'])} | {ratio(best['Rust'], best['Python'], 'higher')} |")
    peak = {name: max(step["rps"] for step in round3[name].values()) for name in SERVERS}
    lines.append(f"| Most requests answered per second | {rate(peak['Python'])} | {rate(peak['Rust'])} | {ratio(peak['Rust'], peak['Python'], 'higher')} |")
    idle = {name: servers[name]["memory_idle"]["pss_kib"] for name in SERVERS}
    lines.append(f"| Memory when idle | {megabytes(idle['Python'])} | {megabytes(idle['Rust'])} | {ratio(idle['Python'], idle['Rust'], 'less')} |")
    start = {name: servers[name]["cold_start_s"] for name in SERVERS}
    lines.append(f"| Start to first answer | {number(start['Python'])} s | {number(start['Rust'])} s | {ratio(start['Python'], start['Rust'], 'faster')} |")

    zero = results["round0"]
    if zero:
        py, rs = zero["Python"], zero["Rust"]
        lines += ["", "### Round 0: the engines alone", ""]
        lines.append(f"Each engine in its own process on CPU {zero['cpu']}, with no HTTP, for each of the {rs['children']:,} R-vector children. Each figure is the best of several passes.")
        lines += ["", f"| | rcpchgrowth {py['version']} | opengrowth {rs['version']} | opengrowth is |", "| --- | ---: | ---: | ---: |"]
        lines.append(f"| One SDS | {nanoseconds(py['sds_ns'])} | {nanoseconds(rs['sds_ns'])} | {ratio(py['sds_ns'], rs['sds_ns'], 'faster')} |")
        lines.append(f"| One whole measurement | {nanoseconds(py['measurement_ns'])} | {nanoseconds(rs['measurement_ns'])} | {ratio(py['measurement_ns'], rs['measurement_ns'], 'faster')} |")

    lines += ["", "### Round 1: one request at a time", ""]
    lines.append("Each request in turn, on one connection, sent as soon as the last is answered, so that no request waits for another: each figure is the time a request takes on its own.")
    lines += ["", "| Request | RCPCH median | RCPCH p99 | opengrowth median | opengrowth p99 | opengrowth is |", "| --- | ---: | ---: | ---: | ---: | ---: |"]
    for label, by in round1.items():
        py, rs = by["Python"], by["Rust"]
        lines.append(f"| {label} | {ms(py['p50_ms'])} | {ms(py['p99_ms'])} | {ms(rs['p50_ms'])} | {ms(rs['p99_ms'])} | {ratio(py['p50_ms'], rs['p50_ms'], 'faster')} |")

    lines += ["", "### Round 2: the highest sustained rate", ""]
    lines.append(f"One calculation at a time, sent at a fixed rate whether or not earlier requests have been answered, from 100 a second and 1.5 times higher each step, until the 99th percentile passes 100 ms or more than 0.1% of requests fail. The highest rate each server passed: the RCPCH server {rate(best['Python'])}, opengrowth {rate(best['Rust'])}.")
    steps = {name: {step["offered"]: step for step in round2[name]["ramp"]} for name in SERVERS}
    lowest = {name: round2[name]["ramp"][0] for name in SERVERS}
    lines += ["", f"At the lowest rate, {rate(lowest['Rust']['offered'])}, the median was {ms(lowest['Python']['p50_ms'])} for the RCPCH server and {ms(lowest['Rust']['p50_ms'])} for opengrowth, slower than in round 1: between requests that far apart, the CPUs slow down and sleep, and each request waits for them to wake."]
    lines += ["", "| Requests sent | RCPCH p99 | RCPCH answered | opengrowth p99 | opengrowth answered |", "| ---: | ---: | ---: | ---: | ---: |"]
    for offered in sorted(set(steps["Python"]) | set(steps["Rust"])):
        cells = []
        for name in SERVERS:
            step = steps[name].get(offered)
            cells += [ms(step["p99_ms"]), f"{step['success'] * 100:.2f}%"] if step else ["", ""]
        lines.append(f"| {rate(offered)} | " + " | ".join(cells) + " |")

    lines += ["", "### Round 3: many clients at once", ""]
    lines.append("A closed loop: each client sends its next request as soon as the last is answered, so the rate is as fast as the server goes. CPU is the share of the server's CPUs it kept busy.")
    lines += ["", "| Clients | RCPCH requests | RCPCH p99 | RCPCH CPU | opengrowth requests | opengrowth p99 | opengrowth CPU |", "| ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
    for clients in sorted(round3["Python"], key=int):
        py, rs = round3["Python"][clients], round3["Rust"][clients]
        lines.append(f"| {int(clients):,} | {rate(py['rps'])} | {ms(py['p99_ms'])} | {percent(cpu_use(py, workers))} | {rate(rs['rps'])} | {ms(rs['p99_ms'])} | {percent(cpu_use(rs, workers))} |")
    busiest = max(cpu_use(step, workers) for step in round3["Rust"].values())
    if busiest < 0.9:
        lines += ["", f"opengrowth never kept more than {percent(busiest)} of its CPUs busy, so the load generator, not opengrowth, set its highest rate: on a machine of its own, opengrowth would answer more."]

    lines += ["", "### Round 4: resources", ""]
    lines.append("Memory is the proportional set size (PSS), which shares out the memory that processes have in common, such as that of the RCPCH server's forked workers, rather than counting it once for each.")
    lines += ["", "| | RCPCH server | opengrowth |", "| --- | ---: | ---: |"]
    lines.append(f"| Start to first answer | {number(start['Python'])} s | {number(start['Rust'])} s |")
    for key, label in (("memory_idle", "Memory when idle"), ("memory_after_ramp", "Memory after round 2"), ("memory_after_sweep", "Memory after round 3")):
        lines.append(f"| {label} | {megabytes(servers['Python'][key]['pss_kib'])} | {megabytes(servers['Rust'][key]['pss_kib'])} |")
    cost = {}
    for name in SERVERS:
        step = steps[name].get(best[name])
        cost[name] = None if step is None or not step["requests"] else step["server_cpu_s"] / step["requests"] * 1e6
    lines.append(f"| CPU time per 1,000 calculations, at the highest sustained rate | {ms(cost['Python'])} | {ms(cost['Rust'])} |")

    versions = env["python_versions"]
    python_commit = env["python_server_commit"]
    own_commit = env["opengrowth_commit"]
    commands = results["commands"]
    pinning = " ".join(commands["pinning"])
    lines += ["", "### Conditions", ""]
    lines.append(f"- **Machine:** {env['processor']}, {env['cpus_total']} CPUs, `{env['os']}`, CPU frequency governor `{env['governor']}`.")
    lines.append(f"- **Each server, in turn:** {env['server_cpus']}, with {workers} workers. Other software could also run on these CPUs.")
    lines.append(f"- **Load generator:** {env['oha']}, on {env['load_cpus']}.")
    lines.append(f"- **Other load:** a load average of {env['load_average'][0]} when the run began.")
    lines.append(f"- **RCPCH server:** digital-growth-charts-server [{short(python_commit)}](https://github.com/rcpch/digital-growth-charts-server/commit/{python_commit}), rcpchgrowth {versions['rcpchgrowth']}, Python {versions['python']}, FastAPI {versions['fastapi']}, Pydantic {versions['pydantic']}, gunicorn {versions['gunicorn']} with uvicorn {versions['uvicorn']} workers, uvloop {versions['uvloop']}, and httptools {versions['httptools']}.")
    modified = ", with uncommitted changes" if env["opengrowth_modified"] else ""
    lines.append(f"- **opengrowth:** {env['opengrowth'].removeprefix('opengrowth ')}, built from [{short(own_commit)}](https://github.com/ohtf/opengrowth/commit/{own_commit}){modified}.")
    lines.append(f"- **Durations:** a {env['warm_up_seconds']} s warm-up on each request, then round 1 for {env['round1_seconds']} s per request, and rounds 2 and 3 for {env['step_seconds']} s per step.")
    lines.append(f"- **Commands:** `{pinning} {' '.join(commands['Python'])}` and `{pinning} {' '.join(commands['Rust'])}`, with load from `{' '.join(commands['oha'])}` and, for each round, `{' '.join(commands['round1'])}`, `{' '.join(commands['round2'])}`, and `{' '.join(commands['round3'])}`.")
    return "\n".join(lines) + "\n"


def block(text):
    """The page's text before the generated part, and after it."""
    if text.count(START) != 1 or text.count(END) != 1 or text.index(START) > text.index(END):
        raise SystemExit(f"{PAGE.relative_to(ROOT)} needs one pair of marker comments: {START} ... {END}")
    before, rest = text.split(START, 1)
    _, after = rest.split(END, 1)
    return before, after


def page(results):
    before, after = block(PAGE.read_text())
    return f"{before}{START}\n\n{render(results, DATA.name)}\n{END}{after}"


def main(argv):
    if argv == ["--check"]:
        current = page(json.loads(DATA.read_text()))
        if current != PAGE.read_text():
            print(f"{PAGE.relative_to(ROOT)} does not match {DATA.relative_to(ROOT)}: run s/bench-page", file=sys.stderr)
            return 1
        print(f"benchmarks page OK: rendered from {DATA.relative_to(ROOT)}")
        return 0
    if len(argv) > 1 or (argv and argv[0].startswith("-")):
        print(__doc__.strip().split("\n\n")[-1], file=sys.stderr)
        return 2
    if argv:
        results = json.loads(Path(argv[0]).read_text())
        check(results)
        DATA.write_text(json.dumps(results, indent=1) + "\n")
        print(f"published {argv[0]} as {DATA.relative_to(ROOT)}", file=sys.stderr)
    PAGE.write_text(page(json.loads(DATA.read_text())))
    print(f"wrote {PAGE.relative_to(ROOT)}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
