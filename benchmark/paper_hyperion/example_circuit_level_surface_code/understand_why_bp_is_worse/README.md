## Observation

the MWPF and HUF decoders achieve 1.0e-5 and 1.5e-5 but BP-MWPF and BP-HUF only achieves 8e-5 with single iteration 
and 2.6e-5 with 100 BP iterations.
We would like to understand why this is happening.

## Debug by finding the logical failures

I run simulation with the same seed for both MWPF and BP-MWPF. Let's see what are the failing error patterns.

```sh
# BP-MWPF
cargo run --release --features hyperion -- tool benchmark '[7]' --djs '[7]' '[7]' -m1000000 '[1.00000000e-03]' -p1 --time-budget 36000 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":true,"bp_iteration":1,"hyperion_config":{"tuning_cluster_size_limit":50}}' --deterministic-seed 123 --debug-print failed-error-pattern >failed_bp_mwpm.txt
# MWPF
cargo run --release --features hyperion -- tool benchmark '[7]' --djs '[7]' '[7]' -m1000000 '[1.00000000e-03]' -p1 --time-budget 36000 --code-type rotated-planar-code --noise-model stim-noise-model --decoder hyperion --decoder-config '{"max_weight":100,"use_bp":false,"hyperion_config":{"tuning_cluster_size_limit":50}}' --deterministic-seed 123 --debug-print failed-error-pattern >failed_mwpm.txt

```