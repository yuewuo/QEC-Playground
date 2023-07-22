#!/bin/sh

# Tailored SC decoder: standard initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --noise-model phenomenological-init --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-standard-init.json --visualizer-model-graph --visualizer-model-hypergraph --visualizer-tailored-model-graph --deterministic-seed 123 -m 200

# Tailored SC decoder: bell initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder tailored-mwpm --decoder-config '{"pcmg":true,"naive_residual_decoding":true,"log_matchings":true}' --noise-model tailored-sc-bell-init-phenomenological --ignore-logical-j --enable-visualizer --visualizer-filename tailored-cc-bell-init.json --visualizer-model-graph --visualizer-tailored-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

# MWPM decoder: standard initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder fusion --decoder-config '{"log_matchings":true}' --noise-model phenomenological-init --ignore-logical-j --enable-visualizer --visualizer-filename mwpm-standard-init.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200

# MWPM decoder: bell initialization
cargo run --release -- tool benchmark '[9]' '[1]' '[0.10]' --bias-eta 1 --code-type rotated-tailored-code --decoder fusion --decoder-config '{"log_matchings":true}' --noise-model tailored-sc-bell-init-phenomenological --ignore-logical-j --enable-visualizer --visualizer-filename mwpm-bell-init.json --visualizer-model-graph --visualizer-model-hypergraph --deterministic-seed 123 -m 200