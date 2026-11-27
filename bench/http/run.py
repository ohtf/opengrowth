# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Benchmark rounds 0 to 4 (spec/benchmark.md): the RCPCH server against opengrowth.

Standard library only. Run through s/bench, which supplies the paths:

    python3 -I bench/http/run.py --opengrowth BIN --round0 BIN \\
        --python-server SRC --python-venv VENV --out DIR \\
        [--data-dir DIR] [--cpus 4] [--quick]

Each server in turn gets the same whole physical cores, the fastest first
(bench/cpus.py), and oha runs on the other cores. The Python contestant runs
its production configuration (gunicorn with one uvicorn worker per CPU, as
s/start-prod-server does), not `uvicorn --reload`. Before any timing, both
servers answer the same requests and the answers are compared outside
provenance: a fast wrong answer scores zero, and report.py will not publish
a run where they differ.

Writes results.json, which s/bench-page publishes and which holds no local
paths; answers.json, every response body, for looking into a mismatch; and
report.md.
"""

import argparse
import datetime
import hashlib
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
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path[:0] = [str(HERE), str(HERE.parent)]

import cpus  # noqa: E402
import report  # noqa: E402

TICKS = os.sysconf("SC_CLK_TCK")

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
CHART = {"sex": "female", "measurement_method": "height", "centile_format": [0.4, 2, 9, 25, 50, 75, 91, 98, 99.6]}
MID_PARENTAL = {"height_paternal": 180, "height_maternal": 165, "sex": "male"}


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
    "One calculation, 8 years": ("/uk-who/calculation", SINGLE),
    "One calculation, preterm, 6 weeks": ("/uk-who/calculation", PRETERM),
    "Bulk, 1 observation": ("/uk-who/bulk-calculation", bulk(1)),
    "Bulk, 200 observations": ("/uk-who/bulk-calculation", bulk(200)),
    "Chart coordinates, 9 centiles": ("/uk-who/chart-coordinates", CHART),
    "Mid-parental height": ("/utilities/mid-parental-height", MID_PARENTAL),
}
LOAD_ENDPOINT = "One calculation, 8 years"
HEADLINE = [LOAD_ENDPOINT, "Bulk, 200 observations", "Chart coordinates, 9 centiles"]


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


def stat(pid):
    """The fields of /proc/PID/stat after the command name."""
    return Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()


def process_tree(pid):
    """The process and its children, such as gunicorn's workers."""
    pids = [pid]
    for child in Path("/proc").iterdir():
        if child.name.isdigit():
            try:
                if int(stat(child.name)[1]) == pid:
                    pids.append(int(child.name))
            except (FileNotFoundError, ProcessLookupError, IndexError, ValueError):
                pass
    return pids


class Server:
    def __init__(self, name, command, cwd, env, server_cpus):
        self.name = name
        self.port = free_port()
        self.url = f"http://127.0.0.1:{self.port}"
        command = ["taskset", "-c", server_cpus, *(c.replace("{port}", str(self.port)) for c in command)]
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

    def cpu_seconds(self):
        """User and system CPU time used so far by the server's processes."""
        ticks = 0
        for pid in process_tree(self.process.pid):
            try:
                fields = stat(pid)
                ticks += int(fields[11]) + int(fields[12])
            except (FileNotFoundError, ProcessLookupError, IndexError, ValueError):
                pass
        return ticks / TICKS

    def memory(self):
        """Resident memory in KiB: RSS, and PSS, which shares out the pages
        the processes have in common, such as those of forked workers."""
        totals = {"rss_kib": 0, "pss_kib": 0}
        for pid in process_tree(self.process.pid):
            try:
                for line in Path(f"/proc/{pid}/smaps_rollup").read_text().splitlines():
                    field = line.split(":")[0]
                    if field in ("Rss", "Pss"):
                        totals[f"{field.lower()}_kib"] += int(line.split()[1])
            except (FileNotFoundError, ProcessLookupError, PermissionError):
                pass
        return totals

    def stop(self):
        try:
            os.killpg(self.process.pid, signal.SIGTERM)
            self.process.wait(timeout=10)
        except (ProcessLookupError, subprocess.TimeoutExpired):
            os.killpg(self.process.pid, signal.SIGKILL)


OHA = ["oha", "--no-tui", "--output-format", "json", "-w", "-m", "POST", "-T", "application/json"]


def oha(server, path, body, load_cpus, args):
    """One oha run against the server, with the CPU time the server used."""
    with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
        json.dump(body, f)
        body_path = f.name
    command = ["taskset", "-c", load_cpus, *OHA, "-D", body_path, *args, server.url + path]
    before = server.cpu_seconds()
    try:
        result = subprocess.run(command, capture_output=True, text=True, check=True)
    finally:
        os.unlink(body_path)
    used = server.cpu_seconds() - before
    data = json.loads(result.stdout)
    codes = data.get("statusCodeDistribution", {})
    errors = data.get("errorDistribution", {})
    requests = sum(codes.values()) + sum(errors.values())
    p = data["latencyPercentiles"]
    ms = lambda v: None if v is None else v * 1000  # noqa: E731
    return {
        "requests": requests,
        "seconds": data["summary"]["total"],
        "rps": data["summary"]["requestsPerSec"],
        "success": codes.get("200", 0) / requests if requests else 0.0,
        "p50_ms": ms(p.get("p50")),
        "p90_ms": ms(p.get("p90")),
        "p99_ms": ms(p.get("p99")),
        "p99.9_ms": ms(p.get("p99.9")),
        "server_cpu_s": used,
        "errors": errors,
    }


def ramp_connections(rate):
    """Connections for an open-loop step: twice the concurrency the rate needs
    at the 100 ms limit, from 16 to 512. More, at a low rate, leave most
    requests the first on their connection, which inflates p99."""
    return min(512, max(16, round(rate * 0.2)))


def run(*command, **kwargs):
    return subprocess.run(command, capture_output=True, text=True, **kwargs).stdout.strip()


def processor():
    for line in Path("/proc/cpuinfo").read_text().splitlines():
        if line.startswith("model name"):
            return line.split(":", 1)[1].strip()
    return platform.processor() or platform.machine()


def environment(args, server, load, settings):
    """Where and with what the run was made. No local paths."""
    python = f"{args.python_venv}/bin/python"
    versions = run(python, "-I", "-c", "import importlib.metadata as m, json, sys; print(json.dumps({'python': sys.version.split()[0], **{p: m.version(p) for p in ('rcpchgrowth', 'fastapi', 'pydantic', 'gunicorn', 'uvicorn', 'uvloop', 'httptools')}}))")
    governor = Path(f"/sys/devices/system/cpu/cpu{server[0]}/cpufreq/scaling_governor")
    built_from = ("src", "build.rs", "Cargo.toml", "Cargo.lock", "data")
    return {
        "date": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "processor": processor(),
        "os": platform.platform(),
        "cpus_total": os.cpu_count(),
        "governor": governor.read_text().strip() if governor.exists() else None,
        "server_cpus": cpus.describe(server),
        "load_cpus": cpus.describe(load),
        "load_average": [round(x, 2) for x in os.getloadavg()],
        "oha": run("oha", "--version"),
        "opengrowth": run(args.opengrowth, "--version"),
        "opengrowth_commit": run("git", "-C", str(ROOT), "rev-parse", "HEAD"),
        "opengrowth_modified": bool(run("git", "-C", str(ROOT), "status", "--porcelain", "--", *built_from)),
        "python_server_commit": run("git", "-C", args.python_server, "rev-parse", "HEAD"),
        "python_versions": json.loads(versions),
        **settings,
    }


def round0(args, cpu, env):
    """The engines alone, in-process, each on the same CPU."""
    commands = {
        "Rust": [args.round0, "--json"],
        "Python": [f"{args.python_venv}/bin/python", "-I", str(ROOT / "bench" / "round0" / "python.py"), "--json"],
    }
    results = {}
    for name, command in commands.items():
        print(f"== round 0: {name}", file=sys.stderr)
        output = subprocess.run(["taskset", "-c", str(cpu), *command], capture_output=True, text=True, check=True, cwd=ROOT, env=env).stdout
        results[name] = json.loads(output.strip().splitlines()[-1])
    return {"cpu": cpu, **results}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--opengrowth", required=True)
    parser.add_argument("--round0", help="the round0 example binary; without it, round 0 is left out")
    parser.add_argument("--python-server", required=True)
    parser.add_argument("--python-venv", required=True)
    parser.add_argument("--data-dir", help="imported tables to layer over the embedded ones (optional)")
    parser.add_argument("--out", required=True)
    parser.add_argument("--cpus", type=int, default=4)
    parser.add_argument("--quick", action="store_true", help="short durations, for a smoke run")
    args = parser.parse_args()

    if not shutil.which("taskset"):
        raise SystemExit("needs taskset (util-linux), to give each server its CPUs")
    server, load = cpus.split(args.cpus)
    if len(load) < args.cpus:
        raise SystemExit(f"need at least {args.cpus} CPUs on other cores for the load generator")
    server_cpus, load_cpus = cpus.ranges(server), cpus.ranges(load)
    settings = {
        "quick": args.quick,
        "workers": args.cpus,
        "warm_up_seconds": 2 if args.quick else 5,
        "round1_seconds": 5 if args.quick else 30,
        "step_seconds": 4 if args.quick else 15,
    }
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    env = dict(os.environ)
    if args.data_dir:
        env["OPENGROWTH_REFERENCE_DATA_DIR"] = args.data_dir
    env.pop("AUTHORIZATION_KEY", None)
    workers = str(args.cpus)
    contestants = {
        "Python": (["gunicorn", "-w", workers, "-b", "127.0.0.1:{port}", "-k", "uvicorn.workers.UvicornWorker", "main:app"], args.python_server, f"{args.python_venv}/bin/gunicorn"),
        "Rust": (["opengrowth", "serve", "--bind", "127.0.0.1:{port}", "--workers", workers], str(ROOT), args.opengrowth),
    }
    rounds = {
        "round1": ["-c", "1", "-z", f"{settings['round1_seconds']}s"],
        "round2": ["-q", "{rate}", "-z", f"{settings['step_seconds']}s", "-c", "{connections}", "--latency-correction"],
        "round3": ["-c", "{connections}", "-z", f"{settings['step_seconds']}s"],
    }
    results = {
        "format": report.FORMAT,
        "environment": environment(args, server, load, settings),
        "commands": {
            "pinning": ["taskset", "-c", server_cpus],
            "Python": contestants["Python"][0],
            "Rust": contestants["Rust"][0],
            "oha": ["taskset", "-c", load_cpus, *OHA, "-D", "BODY.json"],
            **rounds,
            "round0": [["target/release/examples/round0", "--json"], ["python", "-I", "bench/round0/python.py", "--json"]],
        },
        "requests": {label: {"path": path, "body": body} for label, (path, body) in ENDPOINTS.items()},
        "headline": HEADLINE,
        "round0": round0(args, server[0], env) if args.round0 else None,
        "servers": {},
        "round1": {},
        "round2": {},
        "round3": {},
    }

    bodies = {}
    for name, (command, cwd, executable) in contestants.items():
        print(f"== {name}", file=sys.stderr)
        live = Server(name, [executable, *command[1:]], cwd, env, server_cpus)
        try:
            info = {"cold_start_s": live.cold_start, "memory_idle": live.memory(), "load_average_before": [round(x, 2) for x in os.getloadavg()]}

            # Correctness first: the same answers outside provenance.
            bodies[name] = {label: post(live.url + path, body) for label, (path, body) in ENDPOINTS.items()}
            info["answers"] = {}
            for label, (status, body) in bodies[name].items():
                masked = mask_provenance(body).encode()
                info["answers"][label] = {"status": status, "bytes": len(masked), "sha256": hashlib.sha256(masked).hexdigest()}

            print("  warm-up", file=sys.stderr)
            for path, body in ENDPOINTS.values():
                oha(live, path, body, load_cpus, ["-c", "8", "-z", f"{settings['warm_up_seconds']}s"])

            print("  round 1: one request at a time", file=sys.stderr)
            for label, (path, body) in ENDPOINTS.items():
                results["round1"].setdefault(label, {})[name] = oha(live, path, body, load_cpus, rounds["round1"])

            print("  round 2: open-loop ramp", file=sys.stderr)
            path, body = ENDPOINTS[LOAD_ENDPOINT]
            rate, ramp, best = 100.0, [], None
            while True:
                connections = ramp_connections(rate)
                step = oha(live, path, body, load_cpus, [a.replace("{rate}", str(int(rate))).replace("{connections}", str(connections)) for a in rounds["round2"]])
                step["offered"] = int(rate)
                step["connections"] = connections
                ramp.append(step)
                print(f"    {int(rate)}/s: p99 {report.number(step['p99_ms'])} ms, success {step['success']:.4f}", file=sys.stderr)
                if step["p99_ms"] is None or step["p99_ms"] > 100 or step["success"] < 0.999:
                    break
                best = int(rate)
                rate *= 1.5
                if rate > 2_000_000:
                    break
            results["round2"][name] = {"ramp": ramp, "max_sustained": best}
            info["memory_after_ramp"] = live.memory()

            print("  round 3: concurrency sweep", file=sys.stderr)
            sweep = {}
            for connections in (1, 4, 16, 64, 256, 1024):
                sweep[str(connections)] = step = oha(live, path, body, load_cpus, [a.replace("{connections}", str(connections)) for a in rounds["round3"]])
                print(f"    {connections} connections: {step['rps']:,.0f}/s, p99 {report.number(step['p99_ms'])} ms", file=sys.stderr)
            results["round3"][name] = sweep
            info["memory_after_sweep"] = live.memory()
            info["load_average_after"] = [round(x, 2) for x in os.getloadavg()]
            results["servers"][name] = info
        finally:
            live.stop()

    answers = {name: results["servers"][name]["answers"] for name in contestants}
    results["mismatched"] = [label for label in ENDPOINTS if answers["Python"][label] != answers["Rust"][label]]
    results["correct"] = not results["mismatched"]
    (out / "results.json").write_text(json.dumps(results, indent=1) + "\n")
    (out / "answers.json").write_text(json.dumps(bodies, indent=1) + "\n")
    if results["correct"]:
        (out / "report.md").write_text(report.render(results, "results.json"))
        print(out / "report.md")
    else:
        print(f"MISMATCH for: {', '.join(results['mismatched'])}; see {out / 'answers.json'}", file=sys.stderr)
    sys.exit(0 if results["correct"] else 1)


if __name__ == "__main__":
    main()
