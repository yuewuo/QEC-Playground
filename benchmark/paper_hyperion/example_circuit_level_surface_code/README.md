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
cargo run --release --features hyperion -- benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder fusion --decoder-config '{"max_half_weight":100}'
```


## study BP decoder logical error rate

```sh
# BP-MWPF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"tuning_cluster_size_limit":50}}'
   Compiling qecp v0.2.7 (/Users/wuyue/Documents/GitHub/QEC-Playground)
    Finished release [optimized + debuginfo] target(s) in 13.73s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"tuning_cluster_size_limit":50}}'`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 21186 0 0 7 NaN 0 21186 / 100000000 [>] 0.11 % 517.77/s

# MWPF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"tuning_cluster_size_limit":50}}'
    Finished release [optimized + debuginfo] target(s) in 0.40s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"tuning_cluster_size_limit":50}}'`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 926456 8 0.000008635056602796031 7 6.9e-1 0 926456 / 100000000 [>] 0.93 % 5230.66/s

# HyperUF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"tuning_cluster_size_limit":0}}'
    Finished release [optimized + debuginfo] target(s) in 0.50s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"tuning_cluster_size_limit":0}}'`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 586385 14 0.00002387509912429547 7 5.2e-1 0 586385 / 100000000 [>] 1.64 % 17184.78/s

# BP-HyperUF
cargo run --release --features hyperion -- tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"tuning_cluster_size_limit":0}}' -e1000000000
    Finished release [optimized + debuginfo] target(s) in 0.14s
     Running `target/release/qecp-cli tool benchmark '[7]' '[7]' '[0.001]' -p0 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"hyperion_config":{"tuning_cluster_size_limit":0}}' -e1000000000`
format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>
0.001 7 7 478299 29 0.00006063152964986337 7 3.6e-1 0 478299 / 100000000 [>] 0.48 % 509.87/s
```
