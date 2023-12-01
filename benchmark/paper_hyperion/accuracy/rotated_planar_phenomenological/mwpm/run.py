import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from rotated_planar_phenomenological import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import rotated_planar_phenomenological


parameters = parameters = simulation_parameters + \
    f"--time-budget {CH(10)} --decoder fusion".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
