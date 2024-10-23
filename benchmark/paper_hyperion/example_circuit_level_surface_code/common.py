import os
from dataclasses import dataclass

# # for debugging
# p = 0.001
# d = 7
# min_error_cases = 10
# split_job = 10


p = 0.001
d = 7
min_error_cases = 4000
split_job = 100  # split the job by 100 pieces, each only responsible for finding 40 logical errors
max_half_weight = 100
tuning_cluster_size_limit = 50  # for MWPF only
max_N = 1_000_000_000_000


@dataclass
class Configuration:
    name: str
    decoder_parameter: str


configurations = [
    Configuration(
        name="mwpm",
        decoder_parameter=f'--decoder fusion --decoder-config {{"max_half_weight":{max_half_weight}}}',
    ),
    Configuration(
        name="weighted_uf",
        decoder_parameter=f'--decoder fusion --decoder-config {{"max_half_weight":{max_half_weight},"max_tree_size":0}}',
    ),
    Configuration(
        name="unweighted_uf",
        decoder_parameter=f'--decoder fusion --decoder-config {{"max_half_weight":1,"max_tree_size":0}}',
    ),
    Configuration(
        name="unweighted_hyper_uf",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":1,"hyperion_config":{{"tuning_cluster_size_limit":0}}}}',
    ),
    Configuration(
        name="hyper_uf",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"hyperion_config":{{"tuning_cluster_size_limit":0}}}}',
    ),
    Configuration(
        name="mwpf",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"hyperion_config":{{"tuning_cluster_size_limit":{tuning_cluster_size_limit}}}}}',
    ),
    Configuration(
        name="hyper_uf_simple_graph",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"substitute_with_simple_graph":true,"hyperion_config":{{"tuning_cluster_size_limit":0}}}}',
    ),
    Configuration(
        name="mwpf_simple_graph",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"substitute_with_simple_graph":true,"hyperion_config":{{"tuning_cluster_size_limit":{tuning_cluster_size_limit}}}}}',
    ),
    Configuration(
        name="bp_mwpf",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"use_bp":true,"hyperion_config":{{"tuning_cluster_size_limit":{tuning_cluster_size_limit}}}}}',
    ),
    Configuration(
        name="bp_mwpf_simple_graph",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"substitute_with_simple_graph":true,"use_bp":true,"hyperion_config":{{"tuning_cluster_size_limit":{tuning_cluster_size_limit}}}}}',
    ),
]


profile_parent = os.path.dirname(__file__)
if "SLURM_DISTRIBUTE_SCRATCH" in os.environ:
    profile_parent = os.environ["SLURM_DISTRIBUTE_SCRATCH"]
profile_folder = os.path.join(profile_parent, "hyperion_1006_profiles")
if not os.path.exists(profile_folder):
    os.mkdir(profile_folder)
