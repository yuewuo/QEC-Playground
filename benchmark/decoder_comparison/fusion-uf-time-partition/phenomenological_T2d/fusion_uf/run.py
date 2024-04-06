import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))))

if True:
    from common import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import common
    from phenomenological_T2d import noise_model_parameters

parameters = simulation_parameters + noise_model_parameters + \
    f"--time-budget {CH(10)}".split(" ")

# , p_vec=[0.01], di_vec=[5], T_vec=[10]
common_evaluation(os.path.dirname(__file__), parameters, use_partitioned_decoder_config={
    "primal_dual_config": {
        "primal": {
            "max_tree_size": 0  # UF
        }
    },
})
