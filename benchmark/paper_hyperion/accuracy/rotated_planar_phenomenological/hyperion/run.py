import os
import sys
import json
import argparse
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from rotated_planar_phenomenological import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import rotated_planar_phenomenological


slurm_distribute.SLURM_DISTRIBUTE_TIME = "5:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '32G'

decoder_config = {
    "hyperion_config": {
        "primal": {
            "timeout": 3,  # 3 sec
        }
    }
}
parameters = simulation_parameters + \
    f"--time-budget {CH(50)} --decoder hyperion --decoder-config {json.dumps(decoder_config,separators=(',', ':'))}".split(" ")

parser = argparse.ArgumentParser(description="Run QEC benchmark with configurable features")
parser.add_argument(
    "--features",
    type=str,
    default="hyperion mwpf/unsafe_pointer",
    help="Cargo features to compile with (default: 'hyperion mwpf/unsafe_pointer')"
)
parser.add_argument(
    "--customize-filename",
    type=str,
    default="unsafe",
    help="Optional string to customize the output filename"
)
args = parser.parse_args()

features = ["--features", args.features]

common_evaluation(os.path.dirname(__file__), parameters, features, customize_filename=args.customize_filename)
