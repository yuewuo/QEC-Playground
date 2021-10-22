import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator

pair = [ (5, 15, 15), (6, 18, 18) ]  # (di, dj, T)
basic_parameters = "-p0 --use_xzzx_code --error_model GenericBiasedWithBiasedCX --decoder UF --max_half_weight 10".split(" ")

bias_eta_vec = ["+inf"]
for bias_eta in bias_eta_vec:
    parameters = basic_parameters + ["--bias_eta", str(bias_eta)]
    evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters)
    threshold, relative_confidence_interval = evaluator.evaluate_threshold()
    print(f"\n\nresult for (bias_eta = {bias_eta}):")
    print(f"pair: {pair}")
    print(f"parameters: {parameters}")
    print(f"threshold = {threshold}")
    print(f"relative_confidence_interval = {relative_confidence_interval}")
    print("\n\n")
