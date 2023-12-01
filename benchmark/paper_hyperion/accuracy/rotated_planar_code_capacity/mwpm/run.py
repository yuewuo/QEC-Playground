import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))))

if True:
    from rotated_planar_code_capacity import common_evaluation, STO, CH, slurm_distribute
    import rotated_planar_code_capacity


parameters = f"-p{STO(0)} --time-budget {CH(10)} --code-type rotated-planar-code --decoder fusion".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
