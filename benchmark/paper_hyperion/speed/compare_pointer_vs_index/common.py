import os
from dataclasses import dataclass


max_half_weight = 100
tuning_cluster_size_limit = 50  # for MWPF only

# for debugging
p = 0.001
d = 7
min_error_cases = 10
split_job = 10
max_N = 1_000_000


# p = 0.001
# d = 7
# min_error_cases = 4000
# split_job = 100  # split the job by 100 pieces, each only responsible for finding 40 logical errors
# max_N = 1_000_000_000_000


decoder_parameter = f'--decoder hyperion --decoder-config {{"max_weight":{max_half_weight},"hyperion_config":{{"tuning_cluster_size_limit":{tuning_cluster_size_limit}}}}}'


profile_parent = os.path.dirname(__file__)
if "SLURM_DISTRIBUTE_SCRATCH" in os.environ:
    profile_parent = os.environ["SLURM_DISTRIBUTE_SCRATCH"]
profile_folder = os.path.join(profile_parent, "hyperion_1006_profiles")
if not os.path.exists(profile_folder):
    os.mkdir(profile_folder)
