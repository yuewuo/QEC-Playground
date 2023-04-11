

generate visualize file:

`cargo run --release -- tool benchmark [3] [3] [0.005] --code_type StandardPlanarCode --noise_model depolarizing-noise --decoder union-find --decoder_config '{"pcmg":true,"max_half_weight":20,"use_real_weighted":true}' --enable_visualizer --visualizer_filename standard-depolarize.json --visualizer_model_graph -m10`
