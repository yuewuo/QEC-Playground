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

"""
configuration 1:
0.3368060666911686 15 0 100728 48753 0.4840064331665475 15 6.4e-3
0.3384859078685192 15 0 100691 48945 0.4860911104269498 15 6.4e-3
0.34017412735808017 15 0 100699 48966 0.48626103536281395 15 6.3e-3
0.3418707669472043 15 0 100711 48999 0.48653076625194863 15 6.3e-3
0.343575868631661 15 0 100671 49377 0.4904788866704413 15 6.3e-3
configuration 2:
0.3368060666911686 18 0 100283 48594 0.48456867066202647 18 6.4e-3
0.3384859078685192 18 0 100277 48608 0.4847372777406584 18 6.4e-3
0.34017412735808017 18 0 100271 48800 0.4866810942346242 18 6.4e-3
0.3418707669472043 18 0 100276 49049 0.48913997367266343 18 6.3e-3
0.343575868631661 18 0 100262 49509 0.4937962538150047 18 6.3e-3
[warning] found unreasonable intersection point (threshold) value p = 6.972481197598168, algorithm may fail
[warning] found unreasonable intersection point (threshold) value p = 0.0806514940588393, algorithm may fail
[warning] found unreasonable intersection point (threshold) value p = 0.7295596313472856, algorithm may fail
[warning] found unreasonable intersection point (threshold) value p = 0.6869296975408246, algorithm may fail


result for (bias_eta = 100):
pair: [(15, 15, 0), (18, 18, 0)]
parameters: ['-p30', '--use_xzzx_code', '--time_budget', '1200', '--shallow_error_on_bottom', '--bias_eta', '100']
threshold = 0.3387104513900567
relative_confidence_interval = 0.10014156439828462


"""

