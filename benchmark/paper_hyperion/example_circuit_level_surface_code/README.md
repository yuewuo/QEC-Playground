# Example of Circuit-level Surface Code

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


# to run this:

```sh
# under QEC-Playground
cargo build --release --features hyperion
```
