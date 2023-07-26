#!/bin/sh

# Tailored SC decoder: standard initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --noise-model phenomenological-init --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-standard-init.json --visualizer-model-graph --visualizer-model-hypergraph --visualizer-tailored-model-graph --deterministic-seed 123 -m 200

# Tailored SC decoder: bell initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --noise-model tailored-sc-bell-init-phenomenological --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-bell-init.json --visualizer-model-graph --visualizer-tailored-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

# MWPM decoder: standard initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder fusion --decoder-config '{"log_matchings":true}' --noise-model phenomenological-init --ignore-logical-j --enable-visualizer --visualizer-filename mwpm-standard-init.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

# MWPM decoder: bell initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder fusion --decoder-config '{"log_matchings":true}' --noise-model tailored-sc-bell-init-phenomenological --ignore-logical-j --enable-visualizer --visualizer-filename mwpm-bell-init.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

# Tailored SC decoder: standard initialization with high bias
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1e100 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --noise-model phenomenological-init --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-standard-init-high-bias.json --visualizer-model-graph --visualizer-model-hypergraph --visualizer-tailored-model-graph --deterministic-seed 123 -m 200

# Tailored SC decoder: bell initialization with high bias
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1e100 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --noise-model tailored-sc-bell-init-phenomenological --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-bell-init-high-bias.json --visualizer-model-graph --visualizer-tailored-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

# Tailored SC decoder: code capacity with high bias
cargo run --release -- tool benchmark '[7]' '[0]' '[0.10]' --bias-eta 1e100 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-no-init-high-bias.json --visualizer-model-graph --visualizer-tailored-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

# a group of simple cases
cargo run --release -- tool benchmark '[3]' '[3]' '[0.01]' --code-type rotated-planar-code --noise-model stim-noise-model --decoder fusion --decoder-config '{"log_matchings":true}' --enable-visualizer --visualizer-filename rotated-surface-code-decoding-graph-d3.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

cargo run --release -- tool benchmark '[3]' '[3]' '[0.01]' --code-type standard-planar-code --noise-model stim-noise-model --decoder fusion --decoder-config '{"log_matchings":true}' --enable-visualizer --visualizer-filename standard-surface-code-decoding-graph-d3.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

cargo run --release -- tool benchmark '[5]' '[5]' '[0.01]' --code-type rotated-planar-code --noise-model stim-noise-model --decoder fusion --decoder-config '{"log_matchings":true}' --enable-visualizer --visualizer-filename rotated-surface-code-decoding-graph-d5.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

cargo run --release -- tool benchmark '[5]' '[5]' '[0.01]' --code-type standard-planar-code --noise-model stim-noise-model --decoder fusion --decoder-config '{"log_matchings":true}' --enable-visualizer --visualizer-filename standard-surface-code-decoding-graph-d5.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200


# test the new optimization flag: use_unfixed_stabilizer_edges

cargo run --release -- tool benchmark '[7]' '[1]' '[0.10]' --bias-eta 1e300 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true,"use_unfixed_stabilizer_edges":true}' --noise-model tailored-sc-bell-init-phenomenological --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-bell-init-with-opt.json --visualizer-model-graph --visualizer-tailored-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

cargo run --release -- tool benchmark '[7]' '[1]' '[0.10]' --bias-eta 1e300 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --noise-model tailored-sc-bell-init-phenomenological --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-bell-init-no-opt.json --visualizer-model-graph --visualizer-tailored-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200
