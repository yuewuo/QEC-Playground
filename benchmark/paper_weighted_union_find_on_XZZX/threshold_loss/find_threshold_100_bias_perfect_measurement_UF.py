import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator

pair = [ (15, 15, 0), (18, 18, 0) ]  # (di, dj, T)
basic_parameters = "-p0 --use_xzzx_code --decoder UF --max_half_weight 100 --time_budget 1200 --shallow_error_on_bottom".split(" ")

bias_eta_vec = [100]
for bias_eta in bias_eta_vec:
    parameters = basic_parameters + ["--bias_eta", str(bias_eta)]
    evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters)
    evaluator.searching_lower_bound = 0.1
    evaluator.searching_upper_bound = 0.5
    evaluator.target_threshold_accuracy = 0.02
    evaluator.do_not_believe_logical_error_rate_above = 0.6  # logical error rate can go pretty high, so just increase the search space
    threshold, relative_confidence_interval = evaluator.evaluate_threshold()
    print(f"\n\nresult for (bias_eta = {bias_eta}):")
    print(f"pair: {pair}")
    print(f"parameters: {parameters}")
    print(f"threshold = {threshold}")
    print(f"relative_confidence_interval = {relative_confidence_interval}")
    print("\n\n")

"""
configuration 1:
0.28447445451103154 15 0 100521 37753 0.3755732632982163 15 8.0e-3
0.2858932885222079 15 0 100531 37997 0.3779630163830062 15 7.9e-3
0.2873191990561418 15 0 100523 38340 0.3814052505396775 15 7.9e-3
0.28875222140742995 15 0 100527 38682 0.3847921453937748 15 7.8e-3
0.2901923910467032 15 0 100529 39357 0.39149897044633886 15 7.7e-3
configuration 2:
0.28447445451103154 18 0 100327 36565 0.36445822161531793 18 8.2e-3
0.2858932885222079 18 0 100330 37151 0.3702880494368584 18 8.1e-3
0.2873191990561418 18 0 100332 38045 0.3791910855958219 18 7.9e-3
0.28875222140742995 18 0 100333 38089 0.379625845933043 18 7.9e-3
0.2901923910467032 18 0 100330 38962 0.3883384830060799 18 7.8e-3
[warning] found unreasonable intersection point (threshold) value p = 0.61687360454252, algorithm may fail
[warning] extrapolated threshold value even after retry: 0.2928588793953686 0.04322539156513716


result for (bias_eta = 100):
pair: [(15, 15, 0), (18, 18, 0)]
parameters: ['-p30', '--use_xzzx_code', '--decoder', 'UF', '--max_half_weight', '100', '--time_budget', '1200', '--shallow_error_on_bottom', '--bias_eta', '100']
threshold = 0.2928588793953686
relative_confidence_interval = 0.04322539156513716
"""
