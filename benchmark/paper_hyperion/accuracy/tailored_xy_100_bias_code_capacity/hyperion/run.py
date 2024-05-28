import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from tailored_xy_100_bias_code_capacity import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import tailored_xy_100_bias_code_capacity


slurm_distribute.SLURM_DISTRIBUTE_TIME = "5:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '32G'

decoder_config = {
    "hyperion_config": {
        "primal": {
            "timeout": 10 * 60,  # 10min
        }
    }
}
parameters = simulation_parameters + \
    f"--time-budget {CH(50)} --decoder hyperion --decoder-config {json.dumps(decoder_config,separators=(',', ':'))}".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
