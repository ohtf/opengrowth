# Benchmark plan: Python dGC API vs OpenGrowth

How `R29` measures the speed, load behaviour, concurrency, and cost of the Python Digital Growth Charts server against the OpenGrowth Rust server. The aim is evidence for the performance claim in `standard.md` section 1. It should also be fun to watch.

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
| OpenGrowth `sds_for_measurement` | 432 ns | 2.3 million |
| `rcpchgrowth` `sds_for_measurement` | 77 µs | 12,900 |
| `rcpchgrowth` `Measurement` (2 SDS plus comments, bands, validation) | 908 µs per child | 1,100 |

Same work, bit-identical answers, about 180 times faster. This says nothing yet about the API, where HTTP, JSON, and request validation dominate.

### Round 1: one request at a time

**Question:** how long does one request take when the server is idle?

Low fixed rate (10 requests/s) for 60 s. Report p50, p90, p99, and p99.9 for:

- `POST /uk-who/calculation`: one measurement
- `POST /uk-who/bulk-calculation`: 1 and 200 observations
- `POST /uk-who/chart-coordinates`: the heaviest endpoint
- `POST /utilities/mid-parental-height`

### Round 2: how far can it go?

**Question:** what is the highest request rate each server sustains with p99 under 100 ms and errors under 0.1%?

Open-loop ramp in steps of 10% of the last good rate, 60 s per step, until a step fails. The headline number is the last step that passed. Plot latency percentiles against offered load; the shape of the knee is as interesting as where it is.

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

`s/bench` (arrives with `R29`) runs the rounds and writes raw results, charts, and a Markdown report to `.private/bench/<date>/`, which is gitignored. A summary of each published run goes into the documentation site (`R30`) with its method, so anyone can reproduce it. A first published result is a good milestone for an announcement post.

## Prerequisites

- Round 0: available now.
- Rounds 1 to 5: the HTTP server (`R15`), and for the byte-identical correctness check, the Profile B serialiser.
- Round 6 CLI race: the bulk command in the CLI (`R14`).
