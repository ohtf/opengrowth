# Benchmarks

How quickly opengrowth answers, and how much load it carries, compared with the RCPCH Digital Growth Charts server it reimplements: the same requests, on the same computer, on the same CPUs.

!!! warning "Preliminary"

    These figures come from a single run on one computer, which also ran the load generator and other software, against the RCPCH server in its production configuration but not tuned further. They show the size of the difference, not its exact amount. Repeated runs in containers with equal limits, with the load from a separate machine, and against a tuned Python server, are still to do ([roadmap](https://github.com/ohtf/opengrowth/blob/main/spec/roadmap.md), `R29`).

<!-- Written by s/bench-page from docs/benchmarks.json, as far as the closing comment. Do not edit by hand. -->

## Results, 27 November 2026

Before any timing, both servers gave the same answers, outside provenance, to every request below. Every figure is from a single run; the [raw results](benchmarks.json) have them all, with each request's body.

| | RCPCH server | opengrowth | opengrowth is |
| --- | ---: | ---: | ---: |
| One calculation, 8 years, median | 1.56 ms | 34.0 µs | 46× faster |
| Bulk, 200 observations, median | 210 ms | 2.30 ms | 91× faster |
| Chart coordinates, 9 centiles, median | 271 ms | 2.14 ms | 126× faster |
| Highest rate with p99 under 100 ms | 1,139/s | 98,526/s | 87× higher |
| Most requests answered per second | 1,499/s | 108,991/s | 73× higher |
| Memory when idle | 380 MB | 19.9 MB | 19× less |
| Start to first answer | 1.06 s | 0.05 s | 21× faster |

### Round 0: the engines alone

Each engine in its own process on CPU 1, with no HTTP, for each of the 3,984 R-vector children. Each figure is the best of several passes.

| | rcpchgrowth 4.6.5 | opengrowth 0.1.0 | opengrowth is |
| --- | ---: | ---: | ---: |
| One SDS | 80.8 µs | 421 ns | 192× faster |
| One whole measurement | 935 µs | 3.18 µs | 294× faster |

### Round 1: one request at a time

Each request in turn, on one connection, sent as soon as the last is answered, so that no request waits for another: each figure is the time a request takes on its own.

| Request | RCPCH median | RCPCH p99 | opengrowth median | opengrowth p99 | opengrowth is |
| --- | ---: | ---: | ---: | ---: | ---: |
| One calculation, 8 years | 1.56 ms | 2.56 ms | 34.0 µs | 87.5 µs | 46× faster |
| One calculation, preterm, 6 weeks | 1.31 ms | 2.14 ms | 36.0 µs | 251 µs | 36× faster |
| Bulk, 1 observation | 1.64 ms | 2.77 ms | 34.7 µs | 84.3 µs | 47× faster |
| Bulk, 200 observations | 210 ms | 255 ms | 2.30 ms | 3.74 ms | 91× faster |
| Chart coordinates, 9 centiles | 271 ms | 321 ms | 2.14 ms | 3.39 ms | 126× faster |
| Mid-parental height | 86.3 ms | 108 ms | 939 µs | 1.72 ms | 92× faster |

### Round 2: the highest sustained rate

One calculation at a time, sent at a fixed rate whether or not earlier requests have been answered, from 100 a second and 1.5 times higher each step, until the 99th percentile passes 100 ms or more than 0.1% of requests fail. The highest rate each server passed: the RCPCH server 1,139/s, opengrowth 98,526/s.

At the lowest rate, 100/s, the median was 2.30 ms for the RCPCH server and 236 µs for opengrowth, slower than in round 1: between requests that far apart, the CPUs slow down and sleep, and each request waits for them to wake.

| Requests sent | RCPCH p99 | RCPCH answered | opengrowth p99 | opengrowth answered |
| ---: | ---: | ---: | ---: | ---: |
| 100/s | 4.68 ms | 100.00% | 518 µs | 100.00% |
| 150/s | 4.15 ms | 100.00% | 480 µs | 100.00% |
| 225/s | 4.54 ms | 100.00% | 423 µs | 100.00% |
| 337/s | 3.74 ms | 100.00% | 414 µs | 100.00% |
| 506/s | 4.72 ms | 100.00% | 374 µs | 100.00% |
| 759/s | 12.7 ms | 100.00% | 345 µs | 100.00% |
| 1,139/s | 17.1 ms | 100.00% | 350 µs | 100.00% |
| 1,708/s | 2,078 ms | 100.00% | 335 µs | 100.00% |
| 2,562/s |  |  | 353 µs | 100.00% |
| 3,844/s |  |  | 363 µs | 100.00% |
| 5,766/s |  |  | 374 µs | 100.00% |
| 8,649/s |  |  | 376 µs | 100.00% |
| 12,974/s |  |  | 703 µs | 100.00% |
| 19,461/s |  |  | 848 µs | 100.00% |
| 29,192/s |  |  | 1.04 ms | 100.00% |
| 43,789/s |  |  | 1.73 ms | 100.00% |
| 65,684/s |  |  | 2.40 ms | 100.00% |
| 98,526/s |  |  | 37.6 ms | 100.00% |
| 147,789/s |  |  | 4,013 ms | 100.00% |

### Round 3: many clients at once

A closed loop: each client sends its next request as soon as the last is answered, so the rate is as fast as the server goes. CPU is the share of the server's CPUs it kept busy.

| Clients | RCPCH requests | RCPCH p99 | RCPCH CPU | opengrowth requests | opengrowth p99 | opengrowth CPU |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 627/s | 2.39 ms | 25% | 27,319/s | 79.4 µs | 16% |
| 4 | 1,499/s | 3.41 ms | 98% | 79,263/s | 89.4 µs | 65% |
| 16 | 1,401/s | 17.3 ms | 74% | 104,865/s | 261 µs | 91% |
| 64 | 1,403/s | 54.7 ms | 99% | 108,858/s | 1.16 ms | 92% |
| 256 | 1,437/s | 263 ms | 98% | 108,991/s | 4.66 ms | 92% |
| 1,024 | 1,305/s | 2,796 ms | 98% | 101,665/s | 20.7 ms | 91% |

### Round 4: resources

Memory is the proportional set size (PSS), which shares out the memory that processes have in common, such as that of the RCPCH server's forked workers, rather than counting it once for each.

| | RCPCH server | opengrowth |
| --- | ---: | ---: |
| Start to first answer | 1.06 s | 0.05 s |
| Memory when idle | 380 MB | 19.9 MB |
| Memory after round 2 | 484 MB | 39.3 MB |
| Memory after round 3 | 549 MB | 47.8 MB |
| CPU time per 1,000 calculations, at the highest sustained rate | 2,188 ms | 35.8 ms |

### Conditions

- **Machine:** Intel(R) Core(TM) Ultra 9 185H, 22 CPUs, `Linux-7.3.5-2-cachyos-x86_64-with-glibc2.44`, CPU frequency governor `powersave`.
- **Each server, in turn:** CPUs 1-4: 2 cores, 2 threads each, maximum 5.1 GHz, with 4 workers. Other software could also run on these CPUs.
- **Load generator:** oha 1.14.0, on CPUs 0,5-21: 14 cores, 1 or 2 threads each, maximum 2.5 to 4.8 GHz.
- **Other load:** a load average of 4.34 when the run began.
- **RCPCH server:** digital-growth-charts-server [29bb4e9](https://github.com/rcpch/digital-growth-charts-server/commit/29bb4e9e9d93a850a63d5233b927472f808fb9a8), rcpchgrowth 4.6.5, Python 3.13.7, FastAPI 0.116.1, Pydantic 2.11.7, gunicorn 23.0.0 with uvicorn 0.35.0 workers, uvloop 0.23.0, and httptools 0.9.0.
- **opengrowth:** 0.1.0, built from [c3cd1ad](https://github.com/ohtf/opengrowth/commit/c3cd1ad3e60802bf9302ab122675d7304a8984e4).
- **Durations:** a 5 s warm-up on each request, then round 1 for 30 s per request, and rounds 2 and 3 for 15 s per step.
- **Commands:** `taskset -c 1-4 gunicorn -w 4 -b 127.0.0.1:{port} -k uvicorn.workers.UvicornWorker main:app` and `taskset -c 1-4 opengrowth serve --bind 127.0.0.1:{port} --workers 4`, with load from `taskset -c 0,5-21 oha --no-tui --output-format json -w -m POST -T application/json -D BODY.json` and, for each round, `-c 1 -z 30s`, `-q {rate} -z 15s -c {connections} --latency-correction`, and `-c {connections} -z 15s`.

<!-- End of s/bench-page. -->

## What is compared

- **The RCPCH server** is the Digital Growth Charts API server at the commit opengrowth is tested against, run as its own production script runs it: gunicorn, with one uvicorn worker process per CPU, and uvloop and httptools installed. It is not run as its Dockerfile runs it, with `uvicorn --reload`, a setting for development that would make it slower.
- **opengrowth** is `opengrowth serve`, from a release build: one process, with one worker thread per CPU.

Both servers answer these requests with the same bytes outside provenance. Each run checks that before timing anything, and a run where the answers differ is not published: a fast wrong answer is worth nothing.

## How it is measured

Each server runs in turn on the same CPUs, chosen as whole physical cores, the fastest first, so that it shares no core with the load generator, [oha](https://github.com/hatoo/oha), which runs on the other cores. Before its rounds, each server answers every request for a few seconds, untimed, to warm up.

- **Round 0, the engines alone.** opengrowth's engine and `rcpchgrowth`, each in its own process on the same CPU, with no HTTP, calculate every R-vector child: the SDS on its own, and the whole measurement with its ages, centiles, and descriptions.
- **Round 1, one request at a time.** Each request in turn, on one connection, sent as soon as the last is answered, so that none waits for another. The figures are the median and the 99th percentile (p99).
- **Round 2, the highest sustained rate.** One calculation at a fixed rate, starting at 100 a second and 1.5 times higher at each step, until p99 passes 100 ms or more than 0.1% of requests fail. This is open-loop load, as from many independent clients, whose requests keep arriving however slowly the server answers. Times are corrected for coordinated omission: a request that was due while the server was still busy is timed from when it was due, not from when it could be sent. The load generator uses one connection for each 5 requests a second, from 16 to 512, so that requests reuse their connections.
- **Round 3, many clients at once.** From 1 to 1,024 clients, each sending its next request as soon as the last is answered. This closed loop shows how throughput and waiting change as clients are added, and how busy each server's CPUs are.
- **Round 4, resources.** Memory, CPU time per calculation, and the time from starting the server to its first answer.

The conditions under the results give the durations, versions, CPUs, and command lines of the run. The raw results give each request's body and every figure oha reported.

## Reading the results

- The **median** is the time that half of requests beat. **p99** is the time that 99 in 100 beat, which is closer to what users of a busy service notice.
- A server **sustains** a rate when it answers at that rate without falling behind: past that, requests queue, and waiting grows without limit.
- The RCPCH server runs one Python process per CPU, and each calculates one request at a time, as Python's global interpreter lock allows; opengrowth runs one process whose threads share all the requests. The figures compare the two servers as they are deployed, not the languages alone. Round 0 compares the engines alone.

## Run it yourself

The benchmark needs Linux, for `taskset` and the CPU topology, and the pinned RCPCH server.

```sh
s/reference-server    # once: the pinned RCPCH server and its Python environment
cargo install oha     # once
s/bench               # about 15 minutes, or about 5 with --quick
s/bench-page .private/bench/<time>/results.json   # publish that run on this page
```

`s/bench-round0` runs round 0 on its own, in under a minute. The plan, with the rounds still to do, is in [spec/benchmark.md](https://github.com/ohtf/opengrowth/blob/main/spec/benchmark.md).
