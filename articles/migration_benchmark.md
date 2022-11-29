# Migration record

Here I record all the migration commands and their expected outputs, to ensure the migration wouldn't break existing functionalities.


## flamegraph example

```
cargo run --release -- tool fault_tolerant_benchmark '[13]' --djs '[13]' '[1]' -m100000000 -e1000 '[0.01]' -p1 --time_budget 60 --rotated_planar_code

0.01 13 1 290819 526 0.0018086851271753221 13 8.5e-2 0
```

```
cargo run --release -- tool benchmark '[13]' --djs '[13]' '[1]' -m100000000 -e1000 '[0.01]' -p1 --time_budget 60 --code_type RotatedPlanarCode --decoder mwpm --decoder_config '{"pcmg":true}'

0.01 13 1 252614 1000 0.003958608786528063 13 6.2e-2 0
```

The difference in logical error rate probably comes from the additional physical error at the first layer.

## benchmark/union_find_decoder/atomic_qubit_model/erasure_only_circuit_level/run_experiment_long_scale.py

```
cargo run --release -- tool fault_tolerant_benchmark [5] --djs [5] [5] -m100000000 -e1000 [0] -p0 --decoder UF --max_half_weight 10 --time_budget 3600 --use_xzzx_code --pes [0.04] --noise_model OnlyGateErrorCircuitLevel
cargo run --release -- tool fault_tolerant_benchmark [7] --djs [7] [7] -m100000000 -e1000 [0] -p0 --decoder UF --max_half_weight 10 --time_budget 3600 --use_xzzx_code --pes [0.04] --noise_model OnlyGateErrorCircuitLevel

0 5 5 22015 1174 0.05332727685668862 5 5.6e-2 0.04
0 7 7 35077 1013 0.028879322633064402 7 6.1e-2 0.04
```

```
cargo run --release -- tool benchmark [5] --djs [5] [5] -m100000000 -e1000 [0] -p0 --decoder union-find --decoder_config '{"pcmg":true,"max_half_weight":10}' --time_budget 3600 --code_type StandardXZZXCode --pes [0.04] --noise_model only-gate-error-circuit-level
cargo run --release -- tool benchmark [7] --djs [7] [7] -m100000000 -e1000 [0] -p0 --decoder union-find --decoder_config '{"pcmg":true,"max_half_weight":10}' --time_budget 3600 --code_type StandardXZZXCode --pes [0.04] --noise_model only-gate-error-circuit-level

0 5 5 20088 1000 0.04978096375945838 5 6.0e-2 0.04
0 7 7 35852 1000 0.027892446725426755 7 6.1e-2 0.04
```

The difference is within the expected range


