import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator

pair = [ (15, 15, 0), (18, 18, 0) ]  # (di, dj, T)
basic_parameters = "-p0 --use_xzzx_code --time_budget 1200 --shallow_error_on_bottom".split(" ")

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
