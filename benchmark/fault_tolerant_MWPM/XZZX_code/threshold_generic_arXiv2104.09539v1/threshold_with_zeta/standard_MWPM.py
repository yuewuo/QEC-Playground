import os, sys
fault_toleran_MWPM_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
# print(fault_toleran_MWPM_dir)
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator

pair = [ (5, 15, 15), (6, 18, 18) ]  # (di, dj, T)
basic_parameters = "-b10 -p0 --use_xzzx_code --error_model GenericBiasedWithStandardCX".split(" ")

bias_eta_vec = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 1e4, 1e5, 1e6, 1e7, 1e8]
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

