# Auto Profiler

This should be run after major updates to both test performance and also verify correctness.

## Rust

Profiling rust simulator and decoder

need to install https://github.com/flamegraph-rs/flamegraph

run `python3 benchmark.py` to profile decoders, by testing the logical error rate of several well-known cases and also the run time

run `COMPARE_WITH_VERSION=0.1.0 python3 benchmark.py` to compare with a specific version. This can help you to pinpoint possible erroneous logical error rate or performance drop.
