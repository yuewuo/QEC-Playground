

# Change Log

See [this](https://keepachangelog.com/en/1.0.0/) for format of writing change logs.

For major updates, please run the [auto profiler](benchmark/auto_profiler/auto_profiler.md) to test performance and verify correctness

- [ ] deprecate ftqec module and migrate scripts
- [ ] write book
- [ ] update 3D tool to support visualizing data structure from backend
- [ ] add SQLite support to cache intermediate results, to allow broken-point continuous simulation with a random or specified UUID

## 0.1.6 (2022.6.22)

- [x] publish `qecp` package to crate.io
- [x] add more advanced threshold evaluation script `benchmark/threshold_analyzer`
- [x] remove blossom V library in the repo to avoid license issue, now user has to download the blossom V package themselves
- [x] both UF decoder and MWPM decoder (new) support decoding erasure errors
- [x] start writing  using `mdbook`

## 0.1.5 (2022.4.12)

- [x] real-weighted union-find decoder
- [x] make union-find algorithm generic typed to enable advanced information to be computed efficiently
- [x] change blossom V library to use integer matching, because floating-point matching sometimes enter deadlock

## 0.1.4 (2022.3.30)

- [x] build complete model graph using customized Dijkstra's algorithm
- [x] mwpm decoder
- [x] tailored mwpm decoder
- [x] remove all unnecessary .jobout, .joberror, sbatch files and clean up the git repo

## 0.1.3 (2022.3.21)

- [x] remove `mwpm_approx.rs` because we don't need it anymore
- [x] redesign the simulator for improved locality and faster simulation (the current bottleneck), use Box for optional fields, for example correlated error rate

## 0.1.2 (2022.3.21)

- [x] remove `BatchZxError`, `ZxError`, `ZxCorrection`, `ZxMeasurement` and several obsolete types and related functions
- [x] delete unused and obsolete benchmark tools
- [x] remove features `MWPM_reverse_order` and `MWPM_shuffle`
- [x] remove feature `reordered_css_gates` because it's now the default option
- [x] remove feature `noserver` because it doesn't work as expected to speed up compilation

## 0.1.1 (2022.3.20)

- [x] design a set of profiling test cases to both test correctness of existing decoders and record performance for future optimization compares
- [x] upgrade dependencies to latest version

## 0.1.0 (2020.11.8 - 2022.3.20)

**Release Highlights**

This project is under development for 1.5 years as an internal tool to learn, implement and benchmark quantum error correction using surface code. During this period we've made it a versatile tool that supports multiple different decoders (including

- MWPM decoder
- Offer decoder
- Union-Find decoder
- Distributed Union-Find decoder

) on several variants of surface codes (including

- standard and rotated CSS surface code
- rectangular and rotated XZZX surface code
- rotated tailored surface code

) under different error types (including

- single-qubit Pauli errors (pX, pY, pZ) and erasure errors (pE)
- two-qubit correlated Pauli errors (pXI, pXX, pXZ, pXY, pYI, pYX, ...) and erasure errors (pEI, pIE, pEE)

) with highly configurable error model (including

- code capacity error model (errors only on data qubits, without any measurement errors)
- phenomenological error model (errors only on data qubits and pure measurement errors)
- circuit-level error model (errors on both data qubits and ancilla qubits and between any gates)
- any error model, not limited to i.i.d. ones

). But, when I tried to implement an improved MWPM decoder for tailored surface code, I found I need a major upgrade of the code structure to implement it. Also, we need to isolate the functionality of surface code simulator and decoders, for better code structure and to ease maintenance.

