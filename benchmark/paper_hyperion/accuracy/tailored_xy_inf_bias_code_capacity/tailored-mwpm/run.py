import os
import sys
import json
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if True:
    from tailored_xy_inf_bias_code_capacity import common_evaluation, STO, CH, slurm_distribute, simulation_parameters
    import tailored_xy_inf_bias_code_capacity


parameters = parameters = simulation_parameters + \
    f"""--time-budget {CH(10)} --decoder tailored-mwpm --decoder-config {{"pcmg":true}}""".split(" ")

common_evaluation(os.path.dirname(__file__), parameters)
