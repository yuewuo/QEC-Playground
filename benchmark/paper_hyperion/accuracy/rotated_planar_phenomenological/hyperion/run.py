import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from rotated_planar_phenomenological import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import rotated_planar_phenomenological


slurm_distribute.SLURM_DISTRIBUTE_TIME = "5:20:00"

decoder_config = {
    "hyperion_config": {
        "primal": {
            "timeout": 10 * 60,  # 10 min
        }
    }
}
parameters = simulation_parameters + \
    f"--time-budget {CH(50)} --decoder hyperion --decoder-config {json.dumps(decoder_config,separators=(',', ':'))}".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
