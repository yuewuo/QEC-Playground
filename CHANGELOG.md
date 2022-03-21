

# Change Log

See [this](https://keepachangelog.com/en/1.0.0/) for format of writing change logs.

For major updates, please run the [auto profiler](benchmark/auto_profiler/auto_profiler.md) to test performance and verify correctness

## 0.2.0 (2022.3.20)

- [ ] design a set of profiling test cases to both test correctness of existing decoders and record performance for future optimization compares
- [ ] delete unused and obsolete benchmark tools to avoid confusion
- [ ] redesign the simulator for much smaller memory usage for faster simulation (the current bottleneck), use Box for optional fields, for example correlated error rate
- [ ] split the project into using `workspace`, to improve compile speed by letting different packages be compiled individually

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

). But, when I tried to implement an improved MWPM decoder for tailored surface code, I found I need a major upgrade of the code structure to implement it. Also, we need to isolate the funtionality of surface code simulator and decoders, for better code structure and to ease maintenance.

