import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = "-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")

# result:
"""
configuration 1:
0.000635267715 11 11 1000587 41705 0.04168053352681976 11 9.4e-3 0.031128118056955065 0.031763385772403126 0.01
0.000638436153 11 11 1000607 43109 0.043082848710832525 11 9.2e-3 0.03128337147919656 0.03192180763183323 0.01
0.000641620393 11 11 1000598 45523 0.045495793515477743 11 9.0e-3 0.031439399237524604 0.03208101963012715 0.01
0.000644820514 11 11 1000571 46745 0.04671832383708902 11 8.9e-3 0.03159620519398853 0.03224102570815156 0.01
0.000648036597 11 11 1000603 48957 0.04892749671947815 11 8.6e-3 0.03175379322989985 0.03240182982642842 0.01
configuration 2:
0.000635267715 15 15 270886 10696 0.03948524471548917 15 1.9e-2 0.031128118056955065 0.031763385772403126 0.01
0.000638436153 15 15 269556 11370 0.04218047455816231 15 1.8e-2 0.03128337147919656 0.03192180763183323 0.01
0.000641620393 15 15 295453 13091 0.044308231766135395 15 1.7e-2 0.031439399237524604 0.03208101963012715 0.01
0.000644820514 15 15 270361 12666 0.04684847296762477 15 1.7e-2 0.03159620519398853 0.03224102570815156 0.01
0.000648036597 15 15 319768 15870 0.04962973155537765 15 1.5e-2 0.03175379322989985 0.03240182982642842 0.01
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.032256762334080244
relative_confidence_interval = 0.0034337588256931925
"""

init_measurement_error_rate = 0.01

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p * 0.02
    p_erasure = p * 0.98
    error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate},"use_correlated_pauli":true}}'
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [dj], [T], parameters + ["--pes", f"[{p_erasure}]", "--error_model_configuration", error_model_configuration], max_N, min_error_cases)
    if verbose:
        print(" ".join(command))
    stdout, returncode = run_qec_playground_command_get_stdout(command)
    if verbose:
        print("")
        print(stdout)
    assert returncode == 0, "command fails..."
    full_result = stdout.strip(" \r\n").split("\n")[-1]
    lst = full_result.split(" ")
    error_rate = float(lst[5])
    confidence_interval = float(lst[7])
    return error_rate, confidence_interval, full_result + f" {p} {init_measurement_error_rate}"

evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters, simulator_runner=simulator_runner)
evaluator.searching_lower_bound = 0.005
evaluator.searching_upper_bound = 0.06
evaluator.target_threshold_accuracy = 0.01
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
