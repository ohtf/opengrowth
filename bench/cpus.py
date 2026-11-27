# SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Choose the CPUs for a benchmark by physical core (spec/benchmark.md).

A logical CPU shares its core with its hyperthread siblings, and on a hybrid
processor some cores are faster than others. So a contestant gets whole
cores, the fastest first, and the load generator gets only the cores the
contestant does not use. Linux only: it reads /sys/devices/system/cpu.

    python3 -I bench/cpus.py N    print the contestant's N CPUs, then the load
                                  generator's, one line each, as taskset lists
"""

import sys
from pathlib import Path

SYS = Path("/sys/devices/system/cpu")


def parse(text):
    """A CPU list such as "0,5" or "1-4,8", as a sorted list."""
    cpus = set()
    for part in text.strip().split(","):
        if part:
            first, _, last = part.partition("-")
            cpus.update(range(int(first), int(last or first) + 1))
    return sorted(cpus)


def ranges(cpus):
    """A CPU list in taskset's form: [0, 5, 6, 7] is "0,5-7"."""
    runs = []
    for cpu in sorted(cpus):
        if runs and cpu == runs[-1][1] + 1:
            runs[-1][1] = cpu
        else:
            runs.append([cpu, cpu])
    return ",".join(str(a) if a == b else f"{a}-{b}" for a, b in runs)


def cores():
    """Each physical core as (maximum kHz, its online CPUs), fastest first."""
    if not (SYS / "online").exists():
        raise SystemExit("choosing CPUs needs Linux's /sys/devices/system/cpu")
    online = set(parse((SYS / "online").read_text()))
    found = {}
    for cpu in sorted(online):
        topology = SYS / f"cpu{cpu}" / "topology"
        siblings = topology / "core_cpus_list"
        if not siblings.exists():
            siblings = topology / "thread_siblings_list"
        core = tuple(c for c in parse(siblings.read_text()) if c in online)
        frequency = SYS / f"cpu{cpu}" / "cpufreq" / "cpuinfo_max_freq"
        khz = int(frequency.read_text()) if frequency.exists() else 0
        found[core] = max(found.get(core, 0), khz)
    return sorted(((khz, core) for core, khz in found.items()), key=lambda item: (-item[0], item[1]))


def split(count):
    """`count` CPUs on the fastest whole cores, and every CPU on the other
    cores. A sibling left over on the contestant's last core goes to neither."""
    chosen, taken = [], set()
    for _, core in cores():
        if len(chosen) == count:
            break
        chosen.extend(core[: count - len(chosen)])
        taken.update(core)
    if len(chosen) < count:
        raise SystemExit(f"this machine has only {len(chosen)} CPUs")
    rest = sorted(cpu for _, core in cores() for cpu in core if cpu not in taken)
    return sorted(chosen), rest


def describe(cpus):
    """For example "CPUs 1-4: 2 cores, 2 threads each, maximum 5.1 GHz"."""
    used = {}
    for khz, core in cores():
        threads = [cpu for cpu in core if cpu in cpus]
        if threads:
            used[core] = (khz, len(threads))
    counts = sorted({threads for _, threads in used.values()})
    speeds = sorted({khz for khz, _ in used.values()})
    plural = lambda n, word: f"{n} {word}" + ("s" if n != 1 else "")  # noqa: E731
    threads = plural(counts[0], "thread") if len(counts) == 1 else f"{counts[0]} or {counts[-1]} threads"
    if len(used) > 1:
        threads += " each"
    if not speeds or speeds[-1] == 0:
        speed = "speed unknown"
    elif speeds[0] == speeds[-1]:
        speed = f"maximum {speeds[0] / 1e6:.1f} GHz"
    else:
        speed = f"maximum {speeds[0] / 1e6:.1f} to {speeds[-1] / 1e6:.1f} GHz"
    label = "CPUs" if len(cpus) > 1 else "CPU"
    return f"{label} {ranges(cpus)}: {plural(len(used), 'core')}, {threads}, {speed}"


if __name__ == "__main__":
    if len(sys.argv) != 2 or not sys.argv[1].isdigit() or int(sys.argv[1]) < 1:
        print("usage: python3 -I bench/cpus.py N", file=sys.stderr)
        sys.exit(2)
    server, load = split(int(sys.argv[1]))
    print(ranges(server))
    print(ranges(load))
