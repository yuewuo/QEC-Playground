# Example of Circuit-level Surface Code

latest result:
```sh
mwpm: average decoding time: 2.520e-05s, pL = 2.842e-05 (confidence = 2.21e-02)
weighted_uf: average decoding time: 2.628e-05s, pL = 3.214e-05 (confidence = 2.21e-02)
unweighted_uf: average decoding time: 2.791e-05s, pL = 5.954e-05 (confidence = 2.21e-02)
unweighted_hyper_uf: average decoding time: 7.495e-04s, pL = 1.289e-04 (confidence = 2.21e-02)
hyper_uf: average decoding time: 7.562e-04s, pL = 1.551e-05 (confidence = 2.21e-02)
mwpf: average decoding time: 2.369e-03s, pL = 1.060e-05 (confidence = 2.21e-02)
hyper_uf_simple_graph: average decoding time: 1.721e-04s, pL = 3.125e-05 (confidence = 2.21e-02)
mwpf_simple_graph: average decoding time: 2.041e-04s, pL = 2.861e-05 (confidence = 2.21e-02)
```

version 0.2.0:
```sh
unweighted_hyper_uf: average decoding time: 8.507e-04s, pL = 1.250e-04 (confidence = 2.21e-02)
hyper_uf: average decoding time: 9.113e-04s, pL = 1.599e-05 (confidence = 2.21e-02)
mwpf: average decoding time: 2.785e-03s, pL = 1.138e-05 (confidence = 2.21e-02)
hyper_uf_simple_graph: average decoding time: 1.930e-04s, pL = 3.157e-05 (confidence = 2.21e-02)
mwpf_simple_graph: average decoding time: 2.424e-04s, pL = 2.844e-05 (confidence = 2.21e-02)
```

version 0.2.1:
```sh
unweighted_hyper_uf: average decoding time: 8.417e-04s, pL = 1.250e-04 (confidence = 2.21e-02)
hyper_uf: average decoding time: 8.901e-04s, pL = 1.599e-05 (confidence = 2.21e-02)
mwpf: average decoding time: 2.641e-03s, pL = 1.138e-05 (confidence = 2.21e-02)
hyper_uf_simple_graph: average decoding time: 1.852e-04s, pL = 3.157e-05 (confidence = 2.21e-02)
mwpf_simple_graph: average decoding time: 2.344e-04s, pL = 2.844e-05 (confidence = 2.21e-02)
```

Since the surface code is currently the most promising code, I want to use it as the example data points in the
framework figure (the figure with 4 points representing MWPF, hyperUF, UF and MWPM on a speed/accuracy graph).
The idea is to use the standard stim noise model and run each decoder at the same configuration.
The code distance shouldn't be too large, otherwise the logical error rate is too high.
I'll stick to $p=0.001$ for consistency with other works, and hopefully reach $d=9$.
If $d=9$ is too hard, then $d=7$ is also fine given that is the largest code that Google has demonstrated in their work.

Here are some heuristic data points at $p=0.001$

```sh
MWPM decoder, d=9, pL ~= 6e-7 (13000/s on M1 Max 10 CPU)
MWPF decoder, d=9, pL ~= 2e-7 (800/s on M1 Max 10 CPU, cluster size limit = 50)

MWPM decoder, d=7, pL ~= 5e-6 (28000/s on M1 Max 10 CPU)
MWPF decoder, d=7, pL ~= 2e-6 (1800/s on M1 Max 10 CPU, cluster size limit = 50)
```

When gathering at least 4000 logical errors, MWPF decoder at $d=7$ would need 2e9 samples, it will need 1e6 seconds which 
is 278 hours on 10 CPU cores, which is roughly 2780 CPU hours.


## to run this:

```sh
# under QEC-Playground
cargo build --release --features hyperion
# under this folder
SLURM_USE_SCAVENGE_PARTITION=1 python3 run.py
# if slurm failed, using the following command to gather the logical error rate data
SLURM_USE_EXISTING_DATA=1 python3 run.py
# gather the distribution data from profile files
python3 gather_time_distribution.py
# up to this point, all data has been gathered

# plot into pdf files
python3 plot.py
```

## example commands for debugging

```sh
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder fusion --decoder-config '{"max_half_weight":100}'
```


## study BP decoder logical error rate

```sh
# BP-MWPF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"cluster_node_limit":50}}'
   Compiling qecp v0.2.7 (/Users/wuyue/Documents/GitHub/QEC-Playground)
    Finished release [optimized + debuginfo] target(s) in 13.73s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"cluster_node_limit":50}}'`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 21186 0 0 7 NaN 0 21186 / 100000000 [>] 0.11 % 517.77/s

# MWPF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"cluster_node_limit":50}}'
    Finished release [optimized + debuginfo] target(s) in 0.40s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"cluster_node_limit":50}}'`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 926456 8 0.000008635056602796031 7 6.9e-1 0 926456 / 100000000 [>] 0.93 % 5230.66/s

# HyperUF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"cluster_node_limit":0}}'
    Finished release [optimized + debuginfo] target(s) in 0.50s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"cluster_node_limit":0}}'`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 586385 14 0.00002387509912429547 7 5.2e-1 0 586385 / 100000000 [>] 1.64 % 17184.78/s

# BP-HyperUF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"cluster_node_limit":0}}' -e1000000000
    Finished release [optimized + debuginfo] target(s) in 0.14s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"cluster_node_limit":0}}' -e1000000000`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 478299 29 0.00006063152964986337 7 3.6e-1 0 478299 / 100000000 [>] 0.48 % 509.87/s
```


The full result is below. Surprisingly, BP does not improve the logical error rate at all.

```sh
bp_mwpf_it1: average decoding time: 1.514e-02s, pL = 8.859e-05 (confidence = 2.21e-02)
bp_mwpf_it1_time_decode: average decoding time: 1.513e-02s, pL = 8.859e-05 (confidence = 2.21e-02)
bp_mwpf_it1_time_decode_mwpf: average decoding time: 1.260e-02s, pL = 8.859e-05 (confidence = 2.21e-02)
bp_mwpf_it1_time_decode_bp: average decoding time: 2.534e-03s, pL = 8.859e-05 (confidence = 2.21e-02)
bp_huf_it1: average decoding time: 4.656e-03s, pL = 8.882e-05 (confidence = 2.21e-02)
bp_huf_it1_time_decode: average decoding time: 4.651e-03s, pL = 8.882e-05 (confidence = 2.21e-02)
bp_huf_it1_time_decode_mwpf: average decoding time: 2.090e-03s, pL = 8.882e-05 (confidence = 2.21e-02)
bp_huf_it1_time_decode_bp: average decoding time: 2.560e-03s, pL = 8.882e-05 (confidence = 2.21e-02)
bp_mwpf_it2: average decoding time: 7.308e-03s, pL = 2.384e-05 (confidence = 2.21e-02)
bp_mwpf_it2_time_decode: average decoding time: 7.304e-03s, pL = 2.384e-05 (confidence = 2.21e-02)
bp_mwpf_it2_time_decode_mwpf: average decoding time: 3.572e-03s, pL = 2.384e-05 (confidence = 2.21e-02)
bp_mwpf_it2_time_decode_bp: average decoding time: 3.731e-03s, pL = 2.384e-05 (confidence = 2.21e-02)
bp_huf_it2: average decoding time: 4.550e-03s, pL = 2.476e-05 (confidence = 2.21e-02)
bp_huf_it2_time_decode: average decoding time: 4.546e-03s, pL = 2.476e-05 (confidence = 2.21e-02)
bp_huf_it2_time_decode_mwpf: average decoding time: 8.587e-04s, pL = 2.476e-05 (confidence = 2.21e-02)
bp_huf_it2_time_decode_bp: average decoding time: 3.686e-03s, pL = 2.476e-05 (confidence = 2.21e-02)
bp_mwpf_it3: average decoding time: 8.733e-03s, pL = 3.189e-05 (confidence = 2.21e-02)
bp_mwpf_it3_time_decode: average decoding time: 8.728e-03s, pL = 3.189e-05 (confidence = 2.21e-02)
bp_mwpf_it3_time_decode_mwpf: average decoding time: 4.418e-03s, pL = 3.189e-05 (confidence = 2.21e-02)
bp_mwpf_it3_time_decode_bp: average decoding time: 4.308e-03s, pL = 3.189e-05 (confidence = 2.21e-02)
bp_huf_it3: average decoding time: 5.284e-03s, pL = 3.196e-05 (confidence = 2.21e-02)
bp_huf_it3_time_decode: average decoding time: 5.280e-03s, pL = 3.196e-05 (confidence = 2.21e-02)
bp_huf_it3_time_decode_mwpf: average decoding time: 8.714e-04s, pL = 3.196e-05 (confidence = 2.21e-02)
bp_huf_it3_time_decode_bp: average decoding time: 4.407e-03s, pL = 3.196e-05 (confidence = 2.21e-02)
bp_mwpf_it5: average decoding time: 9.375e-03s, pL = 2.664e-05 (confidence = 2.21e-02)
bp_mwpf_it5_time_decode: average decoding time: 9.369e-03s, pL = 2.664e-05 (confidence = 2.21e-02)
bp_mwpf_it5_time_decode_mwpf: average decoding time: 4.259e-03s, pL = 2.664e-05 (confidence = 2.21e-02)
bp_mwpf_it5_time_decode_bp: average decoding time: 5.108e-03s, pL = 2.664e-05 (confidence = 2.21e-02)
bp_huf_it5: average decoding time: 5.832e-03s, pL = 2.702e-05 (confidence = 2.21e-02)
bp_huf_it5_time_decode: average decoding time: 5.828e-03s, pL = 2.702e-05 (confidence = 2.21e-02)
bp_huf_it5_time_decode_mwpf: average decoding time: 8.396e-04s, pL = 2.702e-05 (confidence = 2.21e-02)
bp_huf_it5_time_decode_bp: average decoding time: 4.987e-03s, pL = 2.702e-05 (confidence = 2.21e-02)
bp_mwpf_it7: average decoding time: 8.642e-03s, pL = 2.626e-05 (confidence = 2.21e-02)
bp_mwpf_it7_time_decode: average decoding time: 8.637e-03s, pL = 2.626e-05 (confidence = 2.21e-02)
bp_mwpf_it7_time_decode_mwpf: average decoding time: 3.158e-03s, pL = 2.626e-05 (confidence = 2.21e-02)
bp_mwpf_it7_time_decode_bp: average decoding time: 5.478e-03s, pL = 2.626e-05 (confidence = 2.21e-02)
bp_huf_it7: average decoding time: 6.279e-03s, pL = 2.618e-05 (confidence = 2.21e-02)
bp_huf_it7_time_decode: average decoding time: 6.275e-03s, pL = 2.618e-05 (confidence = 2.21e-02)
bp_huf_it7_time_decode_mwpf: average decoding time: 7.736e-04s, pL = 2.618e-05 (confidence = 2.21e-02)
bp_huf_it7_time_decode_bp: average decoding time: 5.500e-03s, pL = 2.618e-05 (confidence = 2.21e-02)
bp_mwpf_it10: average decoding time: 7.946e-03s, pL = 2.294e-05 (confidence = 2.21e-02)
bp_mwpf_it10_time_decode: average decoding time: 7.942e-03s, pL = 2.294e-05 (confidence = 2.21e-02)
bp_mwpf_it10_time_decode_mwpf: average decoding time: 1.732e-03s, pL = 2.294e-05 (confidence = 2.21e-02)
bp_mwpf_it10_time_decode_bp: average decoding time: 6.208e-03s, pL = 2.294e-05 (confidence = 2.21e-02)
bp_huf_it10: average decoding time: 6.932e-03s, pL = 2.324e-05 (confidence = 2.21e-02)
bp_huf_it10_time_decode: average decoding time: 6.928e-03s, pL = 2.324e-05 (confidence = 2.21e-02)
bp_huf_it10_time_decode_mwpf: average decoding time: 8.254e-04s, pL = 2.324e-05 (confidence = 2.21e-02)
bp_huf_it10_time_decode_bp: average decoding time: 6.101e-03s, pL = 2.324e-05 (confidence = 2.21e-02)
bp_mwpf_it15: average decoding time: 1.019e-02s, pL = 2.934e-05 (confidence = 2.21e-02)
bp_mwpf_it15_time_decode: average decoding time: 1.018e-02s, pL = 2.934e-05 (confidence = 2.21e-02)
bp_mwpf_it15_time_decode_mwpf: average decoding time: 3.237e-03s, pL = 2.934e-05 (confidence = 2.21e-02)
bp_mwpf_it15_time_decode_bp: average decoding time: 6.945e-03s, pL = 2.934e-05 (confidence = 2.21e-02)
bp_huf_it15: average decoding time: 7.919e-03s, pL = 2.917e-05 (confidence = 2.21e-02)
bp_huf_it15_time_decode: average decoding time: 7.915e-03s, pL = 2.917e-05 (confidence = 2.21e-02)
bp_huf_it15_time_decode_mwpf: average decoding time: 8.882e-04s, pL = 2.917e-05 (confidence = 2.21e-02)
bp_huf_it15_time_decode_bp: average decoding time: 7.025e-03s, pL = 2.917e-05 (confidence = 2.21e-02)
bp_mwpf_it20: average decoding time: 9.191e-03s, pL = 2.497e-05 (confidence = 2.21e-02)
bp_mwpf_it20_time_decode: average decoding time: 9.187e-03s, pL = 2.497e-05 (confidence = 2.21e-02)
bp_mwpf_it20_time_decode_mwpf: average decoding time: 1.355e-03s, pL = 2.497e-05 (confidence = 2.21e-02)
bp_mwpf_it20_time_decode_bp: average decoding time: 7.830e-03s, pL = 2.497e-05 (confidence = 2.21e-02)
bp_huf_it20: average decoding time: 8.651e-03s, pL = 2.565e-05 (confidence = 2.21e-02)
bp_huf_it20_time_decode: average decoding time: 8.647e-03s, pL = 2.565e-05 (confidence = 2.21e-02)
bp_huf_it20_time_decode_mwpf: average decoding time: 7.935e-04s, pL = 2.565e-05 (confidence = 2.21e-02)
bp_huf_it20_time_decode_bp: average decoding time: 7.852e-03s, pL = 2.565e-05 (confidence = 2.21e-02)
bp_mwpf_it30: average decoding time: 1.114e-02s, pL = 2.601e-05 (confidence = 2.21e-02)
bp_mwpf_it30_time_decode: average decoding time: 1.114e-02s, pL = 2.601e-05 (confidence = 2.21e-02)
bp_mwpf_it30_time_decode_mwpf: average decoding time: 1.686e-03s, pL = 2.601e-05 (confidence = 2.21e-02)
bp_mwpf_it30_time_decode_bp: average decoding time: 9.453e-03s, pL = 2.601e-05 (confidence = 2.21e-02)
bp_huf_it30: average decoding time: 1.056e-02s, pL = 2.682e-05 (confidence = 2.21e-02)
bp_huf_it30_time_decode: average decoding time: 1.056e-02s, pL = 2.682e-05 (confidence = 2.21e-02)
bp_huf_it30_time_decode_mwpf: average decoding time: 8.581e-04s, pL = 2.682e-05 (confidence = 2.21e-02)
bp_huf_it30_time_decode_bp: average decoding time: 9.698e-03s, pL = 2.682e-05 (confidence = 2.21e-02)
bp_mwpf_it50: average decoding time: 1.462e-02s, pL = 2.649e-05 (confidence = 2.21e-02)
bp_mwpf_it50_time_decode: average decoding time: 1.461e-02s, pL = 2.649e-05 (confidence = 2.21e-02)
bp_mwpf_it50_time_decode_mwpf: average decoding time: 1.685e-03s, pL = 2.649e-05 (confidence = 2.21e-02)
bp_mwpf_it50_time_decode_bp: average decoding time: 1.292e-02s, pL = 2.649e-05 (confidence = 2.21e-02)
bp_huf_it50: average decoding time: 1.415e-02s, pL = 2.632e-05 (confidence = 2.21e-02)
bp_huf_it50_time_decode: average decoding time: 1.414e-02s, pL = 2.632e-05 (confidence = 2.21e-02)
bp_huf_it50_time_decode_mwpf: average decoding time: 8.830e-04s, pL = 2.632e-05 (confidence = 2.21e-02)
bp_huf_it50_time_decode_bp: average decoding time: 1.326e-02s, pL = 2.632e-05 (confidence = 2.21e-02)
bp_mwpf_it70: average decoding time: 1.776e-02s, pL = 2.767e-05 (confidence = 2.21e-02)
bp_mwpf_it70_time_decode: average decoding time: 1.775e-02s, pL = 2.767e-05 (confidence = 2.21e-02)
bp_mwpf_it70_time_decode_mwpf: average decoding time: 1.615e-03s, pL = 2.767e-05 (confidence = 2.21e-02)
bp_mwpf_it70_time_decode_bp: average decoding time: 1.614e-02s, pL = 2.767e-05 (confidence = 2.21e-02)
bp_huf_it70: average decoding time: 1.775e-02s, pL = 2.694e-05 (confidence = 2.21e-02)
bp_huf_it70_time_decode: average decoding time: 1.774e-02s, pL = 2.694e-05 (confidence = 2.21e-02)
bp_huf_it70_time_decode_mwpf: average decoding time: 8.950e-04s, pL = 2.694e-05 (confidence = 2.21e-02)
bp_huf_it70_time_decode_bp: average decoding time: 1.685e-02s, pL = 2.694e-05 (confidence = 2.21e-02)
bp_mwpf_it100: average decoding time: 2.245e-02s, pL = 2.584e-05 (confidence = 2.21e-02)
bp_mwpf_it100_time_decode: average decoding time: 2.245e-02s, pL = 2.584e-05 (confidence = 2.21e-02)
bp_mwpf_it100_time_decode_mwpf: average decoding time: 1.271e-03s, pL = 2.584e-05 (confidence = 2.21e-02)
bp_mwpf_it100_time_decode_bp: average decoding time: 2.117e-02s, pL = 2.584e-05 (confidence = 2.21e-02)
bp_huf_it100: average decoding time: 2.281e-02s, pL = 2.686e-05 (confidence = 2.21e-02)
bp_huf_it100_time_decode: average decoding time: 2.281e-02s, pL = 2.686e-05 (confidence = 2.21e-02)
bp_huf_it100_time_decode_mwpf: average decoding time: 8.272e-04s, pL = 2.686e-05 (confidence = 2.21e-02)
bp_huf_it100_time_decode_bp: average decoding time: 2.198e-02s, pL = 2.686e-05 (confidence = 2.21e-02)
```
## Study Parallel Fusion Hyperion Logical Error rate
```sh
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"use_bp":false,"hyperion_config":{"cluster_node_limit":0}}'
```

```sh
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder parallel-hyper-union-find 
```

```sh
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder parallel-hyper-union-find --deterministic-seed 108 --log-runtime-statistics parallel-try2.txt --decoder-config '{"hyperion_config":{"primal":{"cluster_node_limit":50}}}'
```

## Failed cases
```sh
    Finished release [optimized + debuginfo] target(s) in 0.41s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder parallel-hyper-union-find --log-runtime-statistics parallel-try2.txt --decoder-config '{"hyperion_config":{"primal":{"cluster_node_limit":0}}}' --debug-print failed-error-pattern`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 113 0 0 7 NaN 0 113 / 100000000 [>----------------------------------------------------------------------------------------------------------] 0.00 % 43.30/s 38490m # thread 1, counter: 3
{"[15][8][9]":"Z","[23][5][6]":"X","[27][5][5]":"Z"}
0.001 7 7 147 1 0.006802721088435374 7 2.0e0 0 1 / 10000 [>----------------------------------------------------------------------------------------------] 0.01 % 0.30/s 563m # thread 6, counter: 21
{"[1][5][2]":"X","[22][9][7]":"Z","[26][5][11]":"Y","[31][4][8]":"X"}
0.001 7 7 233 2 0.008583690987124463 7 1.4e0 0 2 / 10000 [>----------------------------------------------------------------------------------------------] 0.02 % 0.41/s 409m # thread 0, counter: 22
{"[27][6][7]":"Y","[27][6][8]":"Z","[35][8][3]":"Z","[38][8][4]":"Y","[38][9][4]":"Z"}
0.001 7 7 298 3 0.010067114093959731 7 1.1e0 0 3 / 10000 [>----------------------------------------------------------------------------------------------] 0.03 % 0.45/s 373m # thread 5, counter: 29
{"[9][4][8]":"X","[23][11][8]":"X","[23][12][8]":"Y","[26][6][6]":"X","[26][7][6]":"Z","[26][7][9]":"Z","[26][8][9]":"Z","[34][4][8]":"Y","[34][4][9]":"Z"}
0.001 7 7 322 4 0.012422360248447204 7 9.7e-1 0 4 / 10000 [>---------------------------------------------------------------------------------------------] 0.04 % 0.54/s 311m # thread 3, counter: 30
{"[20][9][3]":"X","[20][10][3]":"Z","[27][6][7]":"Y","[27][6][8]":"X","[33][7][2]":"X","[41][5][10]":"X","[41][9][4]":"Z"}
0.001 7 7 350 5 0.014285714285714285 7 8.7e-1 0 5 / 10000 [>---------------------------------------------------------------------------------------------] 0.05 % 0.63/s 266m # thread 1, counter: 48
{"[2][8][10]":"Z","[2][9][10]":"X","[10][4][4]":"Y","[10][4][5]":"Y","[13][7][4]":"Z","[16][5][7]":"Y","[17][4][9]":"Z","[17][5][9]":"X","[22][11][6]":"Z","[22][11][7]":"Z"}
0.001 7 7 360 6 0.016666666666666666 7 7.9e-1 0 6 / 10000 [>---------------------------------------------------------------------------------------------] 0.06 % 0.71/s 236m # thread 5, counter: 42
{"[11][8][11]":"Z","[11][9][11]":"Y","[21][11][9]":"Z","[21][11][10]":"Y","[33][12][5]":"Z","[33][12][6]":"X","[37][4][11]":"Y"}
0.001 7 7 391 7 0.017902813299232736 7 7.3e-1 0 7 / 10000 [>---------------------------------------------------------------------------------------------] 0.07 % 0.75/s 221m # thread 1, counter: 54
{"[1][11][6]":"X","[5][4][11]":"Z","[13][8][6]":"Y","[16][5][11]":"X","[19][12][8]":"Y","[29][7][6]":"Y"}
0.001 7 7 392 8 0.02040816326530612 7 6.9e-1 0 8 / 10000 [>----------------------------------------------------------------------------------------------] 0.08 % 0.84/s 198m # thread 0, counter: 44
{"[1][9][9]":"X","[5][8][6]":"X","[25][9][11]":"X","[28][7][10]":"X","[35][3][8]":"X","[35][4][8]":"Y","[35][5][7]":"Z","[41][6][3]":"Y","[41][7][3]":"Z"}
0.001 7 7 464 9 0.01939655172413793 7 6.5e-1 0 9 / 10000 [>----------------------------------------------------------------------------------------------] 0.09 % 0.79/s 209m # thread 4, counter: 101
{"[26][10][5]":"Z","[35][8][1]":"Y"}
0.001 7 7 470 10 0.02127659574468085 7 6.1e-1 0 10 / 10000 [>--------------------------------------------------------------------------------------------] 0.10 % 0.86/s 193m # thread 3, counter: 46
{"[10][3][8]":"X","[10][3][9]":"Y","[23][8][1]":"Z","[23][9][10]":"Y","[35][4][7]":"Z"}
# thread 5, counter: 67
{"[1][6][7]":"Y","[23][10][9]":"Z","[23][11][9]":"X","[29][6][9]":"Y","[31][4][8]":"Z","[31][6][3]":"X","[31][12][5]":"X"}
0.001 7 7 512 12 0.0234375 7 5.6e-1 0 12 / 10000 [>------------------------------------------------------------------------------------------------------] 0.12 % 0.97/s 171m # thread 0, counter: 52
{"[19][9][9]":"Y","[20][11][10]":"Z","[28][4][4]":"Y","[28][4][5]":"Z","[39][9][7]":"Z","[39][9][8]":"Z"}
# thread 3, counter: 59
{"[8][9][8]":"X","[23][5][4]":"X","[25][6][5]":"Z","[34][4][6]":"Z","[34][4][7]":"Y"}
0.001 7 7 534 14 0.026217228464419477 7 5.2e-1 0 14 / 10000 [>-------------------------------------------------------------------------------------------] 0.14 % 1.09/s 153m # thread 2, counter: 57
{"[13][9][3]":"Y","[14][12][5]":"Z","[20][6][12]":"X","[20][7][12]":"Z","[27][4][5]":"X","[27][4][6]":"Z","[35][7][2]":"Y"}
0.001 7 7 559 15 0.026833631484794274 7 5.0e-1 0 15 / 10000 [>-------------------------------------------------------------------------------------------] 0.15 % 1.10/s 151m # thread 1, counter: 73
{"[13][5][9]":"Y","[21][11][9]":"X","[21][11][10]":"X","[23][8][11]":"X","[23][9][11]":"X","[29][4][11]":"Y","[31][8][9]":"Y","[32][9][7]":"Z","[32][10][7]":"Z","[33][13][7]":"Z","[33][13][8]":"Y"}
0.001 7 7 580 16 0.027586206896551724 7 4.8e-1 0 16 / 10000 [>-------------------------------------------------------------------------------------------] 0.16 % 1.13/s 147m ^C
```