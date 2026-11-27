# Benchmark plan: Python dGC API vs opengrowth

How `R29` measures the speed, load behaviour, concurrency, and cost of the Python Digital Growth Charts server against the opengrowth Rust server. The aim is evidence for the performance claim in `standard.md` section 1. It should also be fun to watch.

Every round answers one question, uses the same inputs for both contestants, and checks the answers as well as the timings. A fast wrong answer scores zero.

## The contestants

| Contestant | Build | Process model |
| --- | --- | --- |
| **Python** | `digital-growth-charts-server` at a pinned commit, `rcpchgrowth` 4.6.5 | Its production configuration, `s/start-prod-server`: gunicorn with uvicorn workers, one per CPU. **Not** the Dockerfile's `uvicorn --reload`, which is a development setting and would be an unfair strawman. `uvloop` and `httptools` installed. |
| **Python, tuned** | Same, plus whatever a reasonable Python engineer would do in an afternoon: response model caching, `orjson` responses | Same |
| **Rust** | `opengrowth serve`, release build, pinned commit | One process, Tokio multi-threaded runtime with one worker per CPU |

The tuned Python contestant makes sure we are comparing implementations, not comparing Rust with neglected Python.

## Fairness rules

- Same machine, same container runtime, same `--cpus` and `--memory` limits for each contestant, pinned with `--cpuset-cpus`. The load generator runs on separate cores, or a separate machine for rounds 2 and 3.
- Pin by physical core, not by CPU number. A contestant gets whole cores, the fastest first, so the load generator never shares a core's hyperthreads with it, and on a hybrid processor both contestants get the same fast cores (`bench/cpus.py`).
- Measure memory as PSS, not RSS: RSS counts the pages that forked workers share once for each worker, which overstates the Python contestant.
- Run each configuration at 1, 2, and 4 CPUs.
- Inputs come from the conformance corpus (the R-vector children plus the boundary matrix), cycled so neither side can cache one answer.
- 30 s warm-up, then 3 measured repeats per point; report the median and the spread.
- Use open-loop load (a fixed arrival rate, as with `oha -q`, k6 `constant-arrival-rate`, or wrk2) for latency figures, so slow responses are not hidden by coordinated omission. Closed-loop load is used only for the concurrency sweep, and is labelled as such.
- **Correctness under load:** a 1% sample of responses from both servers is diffed. Rust must be byte-identical to Python (Profile B) or semantically identical (Profile A) for every sampled response; any other difference fails the run.
- Record hardware, kernel, container runtime, both commit SHAs, tool versions, and the exact command lines with the results.

## Rounds

### Round 0: engine in-process (available now)

Just the calculation, with no HTTP. `s/bench-round0` computes chronological and corrected UK-WHO SDS for every R-vector child, in a loop, on one core.

First result, 2026-11-17, Linux x86_64, one core:

| Engine | Per SDS | Per second |
| --- | ---: | ---: |
| opengrowth `sds_for_measurement` | 432 ns | 2.3 million |
| `rcpchgrowth` `sds_for_measurement` | 77 µs | 12,900 |
| `rcpchgrowth` `Measurement` (2 SDS plus comments, bands, validation) | 908 µs per child | 1,100 |

Same work, bit-identical answers, about 180 times faster. This says nothing yet about the API, where HTTP, JSON, and request validation dominate.

Since 2026-11-27 both engines run pinned to the same CPU on the fastest core, the whole measurement is timed for opengrowth (`measure`) as well as for `rcpchgrowth`, and both parse their dates before timing starts. `s/bench` runs this round too, and the documentation site publishes its figures.

### First HTTP results (smoke run)

`s/bench --quick`, 2026-11-17: one 22-CPU Linux x86_64 workstation, each server pinned to CPUs 0-3 with `taskset`, `oha` 1.14 on CPUs 4-21, no containers, 4 to 5 s per step, one request body per endpoint. Python is `digital-growth-charts-server` 29bb4e9 under gunicorn with 4 uvicorn workers. Both servers returned identical answers outside provenance before timing began.

| | Python | Rust |
| --- | ---: | ---: |
| Single calculation, idle, p50 | 3.2 ms | 0.5 ms |
| Bulk, 200 observations, idle, p50 | 221 ms | 3.8 ms |
| Highest open-loop rate with p99 < 100 ms | 1,139/s | 98,526/s |
| Closed loop, 64 connections | 1,790/s, p99 50 ms | 134,898/s, p99 1.0 ms |
| Closed loop, 1,024 connections | 1,313/s, p99 3,981 ms | 121,002/s, p99 18.7 ms |
| Memory idle | 507 MB | 18 MB |
| Start to first answer | 0.9 s | 0.1 s |

Treat these as indicative. They are short runs on one machine with the load generator sharing it, and the Python contestant is untuned. On this machine CPUs 0-3 are not whole cores either: CPU 0 shares a core with CPU 5, and CPU 3 with CPU 4, so the load generator ran on hyperthreads of two of the server's cores. `s/bench` has pinned by core since 2026-11-27. A published result needs full-length runs, the tuned Python contestant, containers with equal limits, and a separate load machine, as set out above.

### Round 1: one request at a time

**Question:** how long does one request take when the server is idle?

One connection, each request sent as soon as the last is answered, for 60 s. Report p50, p90, p99, and p99.9 for:

- `POST /uk-who/calculation`: one measurement
- `POST /uk-who/bulk-calculation`: 1 and 200 observations
- `POST /uk-who/chart-coordinates`: the heaviest endpoint
- `POST /utilities/mid-parental-height`

Changed on 2026-11-27 from a low fixed rate (10 requests/s over 4 connections), which the smoke run of that day showed is not idle for the Python contestant's heavier requests: alone, a nine-centile chart takes it about 270 ms, but at 10 requests/s its median was 1,369 ms, because those requests kept its workers busy and queued. A single connection cannot queue. Spaced requests are slower again, because the CPUs slow down and sleep between them; round 2's lowest step shows that cost.

### Round 2: how far can it go?

**Question:** what is the highest request rate each server sustains with p99 under 100 ms and errors under 0.1%?

Open-loop ramp in steps of 10% of the last good rate, 60 s per step, until a step fails. The headline number is the last step that passed. Plot latency percentiles against offered load; the shape of the knee is as interesting as where it is.

Scale the load generator's connections with the rate: one for each 5 requests/s, from 16 to 512, which is twice the concurrency a rate needs at the 100 ms limit. With 512 connections at every rate, most requests at a low rate are the first on their connection: on 2026-11-27, opengrowth's p99 at 100 requests/s was 2.93 ms with 512 connections, 0.82 ms with 64, and 0.49 ms with 4, while at 1,000 requests/s the three were within 0.07 ms of each other. `s/bench` steps by 1.5 times for now, 15 s per step.

### Round 3: many clients at once

**Question:** how does each server behave as concurrent clients increase?

Closed-loop sweep at 1, 2, 4, ... 1,024 connections, 60 s each. Plot throughput, p50, p99, and error rate against connections. Look for where Python's worker pool saturates and requests start to queue, and whether either server sheds load gracefully or falls over.

### Round 4: what does it cost?

**Question:** how much machine does each server need?

- Requests per CPU-second at the round-2 operating point.
- Resident memory idle, at the operating point, and at peak.
- Time from process start to first successful response (cold start, which matters for scale-to-zero hosting).
- Container image size.
- **Cost per million calculations** at a named cloud instance's on-demand price, with the price and date recorded.
- **Energy per thousand calculations**, from RAPL counters (`perf stat -e power/energy-pkg/`) where the hardware exposes them.

### Round 5: the long run

**Question:** does anything drift?

30 minutes at 70% of each server's round-2 maximum. Plot memory and p99 over time. Anything that grows is a finding.

### Round 6: for fun

- **The Raspberry Pi round:** rounds 1 and 2 on a Raspberry Pi 5. How many children's growth charts can a £60 computer serve?
- **The thundering herd:** 0 to 5,000 requests per second in one second. Who recovers, and how fast?
- **The CLI race:** `hyperfine` timing a bulk calculation of every R-vector child through `opengrowth` and through the Python CLI, including process start. Check first that `rcpchgrowth-python-cli` has a working release; if not, use a short Python script calling `rcpchgrowth` and say so.

## Tools

| Tool | Used for |
| --- | --- |
| [`oha`](https://github.com/hatoo/oha) | Open-loop and closed-loop HTTP load, with a live terminal view to watch the race |
| [k6](https://k6.io) | Scripted scenarios: ramps, bursts, mixed endpoints, response sampling |
| [`hyperfine`](https://github.com/sharkdp/hyperfine) | CLI and cold-start timing |
| `docker stats`, cgroup counters | CPU and memory per contestant |
| `perf stat` | Energy (RAPL) |
| `py-spy`, `samply` | Flame graphs of both servers at the operating point, to explain the results rather than just report them |

## Outputs

`s/bench` runs the rounds and writes `results.json`, `answers.json` (every response body, for looking into a mismatch), and `report.md` to `.private/bench/<date>/`, which is gitignored. `results.json` holds no local paths. `s/bench-page` publishes a run on the documentation site's [benchmarks page](https://ohtf.github.io/opengrowth/benchmarks/), with its method, keeping the run's results in `docs/benchmarks.json`; it refuses a run whose answers differed, and CI checks that the page matches the results (`R54`). A first published result is a good milestone for an announcement post.

## Prerequisites

- Round 0: available now, with `s/bench-round0` or as part of `s/bench`.
- Rounds 1 to 3 and part of 4: available now with `s/bench` (UK-WHO endpoints, and mid-parental height). Round 4's cost and energy figures, round 5, and containerised runs are still to do.
- Round 6 CLI race: the bulk command in the CLI (`R14`).
