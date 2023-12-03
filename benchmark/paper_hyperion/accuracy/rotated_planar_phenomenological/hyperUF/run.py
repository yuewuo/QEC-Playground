import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from rotated_planar_phenomenological import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import rotated_planar_phenomenological

slurm_distribute.SLURM_DISTRIBUTE_TIME = "5:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '32G'

parameters = parameters = simulation_parameters + \
    f"--time-budget {CH(10)} --decoder hyper-union-find".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
