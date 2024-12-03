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
cluster_node_limit = 50  # for MWPF only
max_N = 1_000_000_000_000


@dataclass
class Configuration:
    name: str
    decoder_parameter: str


configurations = [
    # Configuration(
    #     name="mwpm",
    #     decoder_parameter=f'--decoder fusion --decoder-config {{"max_half_weight":{max_half_weight}}}',
    # ),
    # Configuration(
    #     name="weighted_uf",
    #     decoder_parameter=f'--decoder fusion --decoder-config {{"max_half_weight":{max_half_weight},"max_tree_size":0}}',
    # ),
    # Configuration(
    #     name="unweighted_uf",
    #     decoder_parameter=f'--decoder fusion --decoder-config {{"max_half_weight":1,"max_tree_size":0}}',
    # ),
    Configuration(
        name="unweighted_hyper_uf",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"uniform_weights":true,"hyperion_config":{{"cluster_node_limit":0}}}}',
    ),
    Configuration(
        name="hyper_uf",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"hyperion_config":{{"cluster_node_limit":0}}}}',
    ),
    Configuration(
        name="mwpf",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"hyperion_config":{{"cluster_node_limit":{cluster_node_limit}}}}}',
    ),
    Configuration(
        name="hyper_uf_simple_graph",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"substitute_with_simple_graph":true,"hyperion_config":{{"cluster_node_limit":0}}}}',
    ),
    Configuration(
        name="mwpf_simple_graph",
        decoder_parameter=f'--decoder hyperion --decoder-config {{"substitute_with_simple_graph":true,"hyperion_config":{{"cluster_node_limit":{cluster_node_limit}}}}}',
    ),
    # Configuration(
    #     name="bp_mwpf",
    #     decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"use_bp":true,"hyperion_config":{{"cluster_node_limit":{cluster_node_limit}}}}}',
    # ),
    # Configuration(
    #     name="bp_huf",
    #     decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"use_bp":true,"hyperion_config":{{"cluster_node_limit":0}}}}',
    # ),
    # Configuration(
    #     name="bp_mwpf_ratio",
    #     decoder_parameter=f'--decoder hyperion --decoder-config {{"use_bp":true,"bp_application_ratio":0.1,"hyperion_config":{{"cluster_node_limit":{cluster_node_limit}}}}}',
    # ),
]


# trying to understand the number of iteration of BP
# for bp_iteration in [1, 2, 3, 5, 7, 10, 15, 20, 30, 50, 70, 100]:
#     configurations += [
#         Configuration(
#             name=f"bp_mwpf_it{bp_iteration}",
#             decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"use_bp":true,"bp_iteration":{bp_iteration},"hyperion_config":{{"cluster_node_limit":{cluster_node_limit}}}}}',
#         ),
#         Configuration(
#             name=f"bp_huf_it{bp_iteration}",
#             decoder_parameter=f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"use_bp":true,"bp_iteration":{bp_iteration},"hyperion_config":{{"cluster_node_limit":0}}}}',
#         ),
#     ]


profile_parent = os.path.dirname(__file__)
if "SLURM_DISTRIBUTE_SCRATCH" in os.environ:
    profile_parent = os.environ["SLURM_DISTRIBUTE_SCRATCH"]
profile_folder = os.path.join(profile_parent, "hyperion_1006_profiles")
if not os.path.exists(profile_folder):
    os.mkdir(profile_folder)
