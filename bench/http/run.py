# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Benchmark rounds 1 to 4 (spec/benchmark.md): the FastAPI server against opengrowth.

Standard library only. Run through s/bench, which supplies the paths:

    python3 -I bench/http/run.py --opengrowth BIN --python-server SRC --python-venv VENV \
        --out DIR [--data-dir DIR] [--cpus 4] [--quick]

Each server gets the same pinned CPU set; oha runs on a disjoint set. The
Python contestant runs its production configuration (gunicorn with one
uvicorn worker per CPU, as s/start-prod-server does), not `uvicorn --reload`.
Before any timing, both servers answer the same requests and the answers are
compared outside provenance; a fast wrong answer scores zero.
"""

import argparse
import datetime
import json
import os
import platform
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

SINGLE = {
    "birth_date": "2020-04-12",
    "observation_date": "2028-06-12",
    "observation_value": 115,
    "sex": "female",
    "gestation_weeks": 40,
    "gestation_days": 0,
    "measurement_method": "height",
}
PRETERM = {
    "birth_date": "2026-10-06",
    "observation_date": "2026-11-17",
    "observation_value": 1.42,
    "sex": "female",
    "gestation_weeks": 27,
    "gestation_days": 3,
    "measurement_method": "weight",
}


def bulk(n):
    return {
        "measurement_method": "height",
        "birth_date": "2020-04-12",
        "sex": "female",
        "observations": [
            {"observation_date": (datetime.date(2021, 1, 1) + datetime.timedelta(days=30 * i)).isoformat(), "observation_value": 80 + i * 0.2}
            for i in range(n)
        ],
    }


ENDPOINTS = {
    "calculation, term 8 y": ("/uk-who/calculation", SINGLE),
    "calculation, preterm 6 wk": ("/uk-who/calculation", PRETERM),
    "bulk, 1 observation": ("/uk-who/bulk-calculation", bulk(1)),
    "bulk, 200 observations": ("/uk-who/bulk-calculation", bulk(200)),
}


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def post(url, body, timeout=30):
    request = urllib.request.Request(url, data=json.dumps(body).encode(), headers={"content-type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return response.status, response.read().decode()
    except urllib.error.HTTPError as error:
        return error.code, error.read().decode()


def mask_provenance(body):
    decoder = json.JSONDecoder()
    key = '"provenance":'
    start = 0
    while (i := body.find(key, start)) != -1:
        if i == 0 or body[i - 1] not in "{,":
            start = i + len(key)
            continue
        _, end = decoder.raw_decode(body, i + len(key))
        body = body[:i] + body[end + 1 :] if end < len(body) and body[end] == "," else body[: i - 1] + body[end:]
        start = max(i - 1, 0)
    return body


def rss_kib(pids):
    total = 0
    for pid in pids:
        try:
            for line in Path(f"/proc/{pid}/status").read_text().splitlines():
                if line.startswith("VmRSS:"):
                    total += int(line.split()[1])
        except FileNotFoundError:
            pass
    return total


def process_tree(pid):
    pids = [pid]
    for child in Path("/proc").iterdir():
        if child.name.isdigit():
            try:
                if int(Path(child, "stat").read_text().split(") ")[1].split()[1]) == pid:
                    pids.append(int(child.name))
            except (FileNotFoundError, IndexError, ValueError):
                pass
    return pids


class Server:
    def __init__(self, name, command, cwd, env, cpus):
        self.name = name
        self.port = free_port()
        self.url = f"http://127.0.0.1:{self.port}"
        command = [c.replace("{port}", str(self.port)) for c in command]
        if shutil.which("taskset"):
            command = ["taskset", "-c", cpus, *command]
        started = time.perf_counter()
        self.process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)
        deadline = started + 120
        while time.perf_counter() < deadline:
            try:
                status, _ = post(self.url + "/uk-who/calculation", SINGLE, timeout=2)
                if status == 200:
                    break
            except OSError:
                time.sleep(0.05)
        else:
            self.stop()
            raise SystemExit(f"{name} did not start")
        self.cold_start = time.perf_counter() - started

    def rss(self):
        return rss_kib(process_tree(self.process.pid))

    def stop(self):
        try:
            os.killpg(self.process.pid, signal.SIGTERM)
            self.process.wait(timeout=10)
        except (ProcessLookupError, subprocess.TimeoutExpired):
            os.killpg(self.process.pid, signal.SIGKILL)


def oha(url, body, load_cpus, args):
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
        json.dump(body, f)
        body_path = f.name
    command = ["oha", "--no-tui", "--output-format", "json", "-w", "-m", "POST", "-T", "application/json", "-D", body_path, *args, url]
    if shutil.which("taskset"):
        command = ["taskset", "-c", load_cpus, *command]
    result = subprocess.run(command, capture_output=True, text=True, check=True)
    os.unlink(body_path)
    data = json.loads(result.stdout)
    codes = data.get("statusCodeDistribution", {})
    total = sum(codes.values()) + sum(data.get("errorDistribution", {}).values())
    ok = codes.get("200", 0)
    p = data["latencyPercentiles"]
    ms = lambda v: None if v is None else v * 1000  # noqa: E731
    return {
        "rps": data["summary"]["requestsPerSec"],
        "success": ok / total if total else 0.0,
        "p50": ms(p.get("p50")),
        "p90": ms(p.get("p90")),
        "p99": ms(p.get("p99")),
        "p99.9": ms(p.get("p99.9")),
        "errors": data.get("errorDistribution", {}),
    }


def fmt(v, unit=""):
    return "-" if v is None else f"{v:,.1f}{unit}" if isinstance(v, float) else f"{v}{unit}"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--opengrowth", required=True)
    parser.add_argument("--python-server", required=True)
    parser.add_argument("--python-venv", required=True)
    parser.add_argument("--data-dir", help="imported tables to layer over the embedded ones (optional)")
    parser.add_argument("--out", required=True)
    parser.add_argument("--cpus", type=int, default=4)
    parser.add_argument("--quick", action="store_true", help="short durations, for a smoke run")
    args = parser.parse_args()

    total_cpus = os.cpu_count() or 2
    if total_cpus < args.cpus * 2:
        raise SystemExit(f"need at least {args.cpus * 2} CPUs to separate server and load generator")
    server_cpus = f"0-{args.cpus - 1}"
    load_cpus = f"{args.cpus}-{total_cpus - 1}"
    seconds = 5 if args.quick else 30
    step_seconds = 4 if args.quick else 15
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    env = dict(os.environ)
    if args.data_dir:
        env["OPENGROWTH_REFERENCE_DATA_DIR"] = args.data_dir
    env.pop("AUTHORIZATION_KEY", None)
    contestants = {
        "Python": (
            [f"{args.python_venv}/bin/gunicorn", "-w", str(args.cpus), "-b", "127.0.0.1:{port}", "-k", "uvicorn.workers.UvicornWorker", "main:app"],
            args.python_server,
        ),
        "Rust": ([args.opengrowth, "serve", "--bind", "127.0.0.1:{port}", "--workers", str(args.cpus)], str(ROOT)),
    }
    results = {"environment": {}, "servers": {}, "round1": {}, "round2": {}, "round3": {}}
    results["environment"] = {
        "date": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "host": platform.platform(),
        "processor": platform.processor() or platform.machine(),
        "cpus_total": total_cpus,
        "server_cpus": server_cpus,
        "load_cpus": load_cpus,
        "oha": subprocess.run(["oha", "--version"], capture_output=True, text=True).stdout.strip(),
        "opengrowth": subprocess.run([args.opengrowth, "--version"], capture_output=True, text=True).stdout.strip(),
        "python_server_commit": subprocess.run(["git", "-C", args.python_server, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip(),
        "quick": args.quick,
    }

    for name, (command, cwd) in contestants.items():
        print(f"== {name}", file=sys.stderr)
        server = Server(name, command, cwd, env, server_cpus)
        try:
            info = {"cold_start_s": server.cold_start, "rss_idle_kib": server.rss()}

            # Correctness first: same answers outside provenance.
            answers = {}
            for label, (path, body) in ENDPOINTS.items():
                answers[label] = post(server.url + path, body)
            info["answers"] = {k: [s, mask_provenance(b)] for k, (s, b) in answers.items()}

            print("  round 1: idle latency", file=sys.stderr)
            for label, (path, body) in ENDPOINTS.items():
                results["round1"].setdefault(label, {})[name] = oha(server.url + path, body, load_cpus, ["-q", "10", "-z", f"{seconds}s", "-c", "4", "--latency-correction"])

            print("  round 2: open-loop ramp", file=sys.stderr)
            rate, ramp, best = 100.0, [], None
            while True:
                r = oha(server.url + "/uk-who/calculation", SINGLE, load_cpus, ["-q", str(int(rate)), "-z", f"{step_seconds}s", "-c", "512", "--latency-correction"])
                r["offered"] = rate
                ramp.append(r)
                print(f"    {int(rate)}/s: p99 {fmt(r['p99'], ' ms')}, success {r['success']:.4f}", file=sys.stderr)
                if r["p99"] is None or r["p99"] > 100 or r["success"] < 0.999:
                    break
                best = rate
                rate *= 1.5
                if rate > 2_000_000:
                    break
            results["round2"][name] = {"ramp": ramp, "max_sustained": best}
            info["rss_loaded_kib"] = server.rss()

            print("  round 3: concurrency sweep", file=sys.stderr)
            sweep = {}
            for connections in (1, 4, 16, 64, 256, 1024):
                sweep[connections] = oha(server.url + "/uk-who/calculation", SINGLE, load_cpus, ["-c", str(connections), "-z", f"{step_seconds}s"])
                print(f"    {connections} connections: {sweep[connections]['rps']:,.0f}/s, p99 {fmt(sweep[connections]['p99'], ' ms')}", file=sys.stderr)
            results["round3"][name] = sweep
            info["rss_after_kib"] = server.rss()
            results["servers"][name] = info
        finally:
            server.stop()

    py, rs = (results["servers"][n]["answers"] for n in ("Python", "Rust"))
    mismatched = [k for k in py if py[k] != rs[k]]
    results["correct"] = not mismatched
    (out / "results.json").write_text(json.dumps(results, indent=1))

    lines = [f"# opengrowth vs FastAPI benchmark, {results['environment']['date']}", ""]
    lines += [f"- {k}: `{v}`" for k, v in results["environment"].items()]
    lines += ["", "## Correctness", "", "Identical answers outside provenance for every endpoint." if not mismatched else f"**MISMATCH** for: {', '.join(mismatched)}. Timings below do not count."]
    lines += ["", "## Round 1: one request at a time (10 requests/s)", "", "| Endpoint | Python p50 | Python p99 | Rust p50 | Rust p99 |", "| --- | ---: | ---: | ---: | ---: |"]
    for label, by in results["round1"].items():
        lines.append(f"| {label} | {fmt(by['Python']['p50'], ' ms')} | {fmt(by['Python']['p99'], ' ms')} | {fmt(by['Rust']['p50'], ' ms')} | {fmt(by['Rust']['p99'], ' ms')} |")
    lines += ["", "## Round 2: highest rate with p99 under 100 ms and 99.9% success", ""]
    for name in ("Python", "Rust"):
        lines.append(f"- {name}: {fmt(results['round2'][name]['max_sustained'])} requests/s")
    lines += ["", "## Round 3: concurrency sweep (closed loop)", "", "| Connections | Python req/s | Python p99 | Rust req/s | Rust p99 |", "| ---: | ---: | ---: | ---: | ---: |"]
    for c in results["round3"]["Python"]:
        p, r = results["round3"]["Python"][c], results["round3"]["Rust"][c]
        lines.append(f"| {c} | {p['rps']:,.0f} | {fmt(p['p99'], ' ms')} | {r['rps']:,.0f} | {fmt(r['p99'], ' ms')} |")
    lines += ["", "## Round 4 (partial): resources", "", "| | Python | Rust |", "| --- | ---: | ---: |"]
    for key, label in [("cold_start_s", "start to first 200 (s)"), ("rss_idle_kib", "RSS idle (KiB)"), ("rss_loaded_kib", "RSS after ramp (KiB)"), ("rss_after_kib", "RSS after sweep (KiB)")]:
        lines.append(f"| {label} | {fmt(results['servers']['Python'][key])} | {fmt(results['servers']['Rust'][key])} |")
    (out / "report.md").write_text("\n".join(lines) + "\n")
    print(out / "report.md")
    sys.exit(0 if not mismatched else 1)


if __name__ == "__main__":
    main()
