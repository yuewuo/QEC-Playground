import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))))

if True:
    from rotated_planar_code_capacity import common_evaluation, STO, CH, slurm_distribute
    import rotated_planar_code_capacity


slurm_distribute.SLURM_DISTRIBUTE_TIME = "5:20:00"

decoder_config = {
    "hyperion_config": {
        "primal": {
            "timeout": 10 * 60,  # 10 min
        }
    }
}
parameters = f"-p{STO(0)} --time-budget {CH(50)} --code-type rotated-planar-code --decoder hyperion --decoder-config {json.dumps(decoder_config,separators=(',', ':'))}".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
