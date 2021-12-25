import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")

# result:
"""
configuration 1:
0.000564336319 11 11 1000471 24814 0.024802318108171052 11 1.2e-2 0.027652479640686713 0.0282168159598844
0.000567150982 11 11 1000447 26290 0.02627825362063158 11 1.2e-2 0.02779039810044791 0.028357549082089706
0.000569979682 11 11 1000499 27399 0.02738533471797573 11 1.2e-2 0.027929004437093573 0.028498984119483237
0.000572822491 11 11 1000457 28799 0.028785844868894916 11 1.1e-2 0.028068302081452383 0.028641124572910598
0.000575679479 11 11 1000488 30656 0.030641047168981538 11 1.1e-2 0.02820829448146451 0.02878397396067807
configuration 2:
0.000564336319 15 15 383772 9088 0.02368072709838133 15 2.0e-2 0.027652479640686713 0.0282168159598844
0.000567150982 15 15 378795 9627 0.025414802201718607 15 2.0e-2 0.02779039810044791 0.028357549082089706
0.000569979682 15 15 380678 10628 0.027918608377684027 15 1.9e-2 0.027929004437093573 0.028498984119483237
0.000572822491 15 15 381424 11270 0.029547170602793743 15 1.8e-2 0.028068302081452383 0.028641124572910598
0.000575679479 15 15 381049 11824 0.03103012998328299 15 1.8e-2 0.02820829448146451 0.02878397396067807
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p0', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.028533226903382943
relative_confidence_interval = 0.003045279360513855
"""

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p * 0.02
    p_erasure = p * 0.98
    init_measurement_error_rate = p
    error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate},"use_correlated_pauli":true}}'
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [dj], [T], parameters + ["--pes", f"[{p_erasure}]"] + ["--error_model_configuration", error_model_configuration], max_N, min_error_cases)
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
    return error_rate, confidence_interval, full_result + f" {p}"


evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters, simulator_runner=simulator_runner)
evaluator.searching_lower_bound = 0.005
evaluator.searching_upper_bound = 0.05
evaluator.target_threshold_accuracy = 0.01
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
print("\n\n")
