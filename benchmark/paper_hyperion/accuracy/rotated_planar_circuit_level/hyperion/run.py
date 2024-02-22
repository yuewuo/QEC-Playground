import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from rotated_planar_circuit_level import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import rotated_planar_circuit_level


slurm_distribute.SLURM_DISTRIBUTE_TIME = "10:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '96G'

decoder_config = {
    "hyperion_config": {
        "primal": {
            "timeout": 1,  # 1 sec for each cluster
        }
    }
}
parameters = simulation_parameters + \
    f"--time-budget {CH(100)} --decoder hyperion --decoder-config {json.dumps(decoder_config,separators=(',', ':'))}".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
