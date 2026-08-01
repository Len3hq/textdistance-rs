# Benchmark Methodology

## Environment
- **Machine:** macOS, Apple Silicon
- **Rust:** 1.97.1 (release build, `--release` flag)
- **Python:** 3.9.6
- **Measurement tool:** `time` command for wall-clock, Python `time.perf_counter()` for fine-grained

## Workload
A corpus of 1000 random string pairs (length 5-100 chars, mixed ASCII) run through all algorithms.

## Metrics Collected
- **p99 latency:** 99th percentile single-call completion time
- **RSS:** Maximum resident set size during benchmark run (via `/usr/bin/time -l`)
- **Startup time:** Cold-start time to first result (`time` before and after first call)
- **Throughput:** Total calls completed per second over 10-second sampling window

## Methodology
1. Warm up: 100 calls (discarded)
2. Measure: 10,000 calls per algorithm
3. Each measurement repeated 3 times, median reported
4. Original Python: measured via `textdistance` library, external=False
5. Rust port: measured via CLI subprocess (one process per call, matching adapter pattern)

## Notes
- Subprocess overhead per call is measured and reported separately
- Python measurements exclude import time (library already loaded)
- All measurements single-threaded
