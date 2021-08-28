import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = "-p0 --decoder UF --max_half_weight 10 --time_budget 600 --use_xzzx_code --error_model OnlyGateErrorCircuitLevel".split(" ")

# result:
"""
configuration 1:
0.0014858678660484362 11 11 637285 55806 0.08756835638685988 11 7.9e-3 0.02971735732096872 0.001
0.0014932787243207098 11 11 633511 57891 0.0913812072718548 11 7.8e-3 0.029865574486414193 0.001
0.0015007265447089203 11 11 631393 59524 0.09427408919642759 11 7.6e-3 0.030014530894178403 0.001
0.0015082115115639166 11 11 627108 61558 0.09816172015027715 11 7.5e-3 0.03016423023127833 0.001
0.0015157338101560091 11 11 642282 65342 0.10173412924540934 11 7.3e-3 0.030314676203120186 0.001
configuration 2:
0.0014858678660484362 15 15 188384 16298 0.08651477832512315 15 1.5e-2 0.02971735732096872 0.001
0.0014932787243207098 15 15 189652 17301 0.09122498049058275 15 1.4e-2 0.029865574486414193 0.001
0.0015007265447089203 15 15 184430 17609 0.09547795911728027 15 1.4e-2 0.030014530894178403 0.001
0.0015082115115639166 15 15 184797 18679 0.10107848071126696 15 1.4e-2 0.03016423023127833 0.001
0.0015157338101560091 15 15 188651 19945 0.10572432693174168 15 1.3e-2 0.030314676203120186 0.001
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '600', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevel']
threshold = 0.029854638556969574
relative_confidence_interval = 0.0037588641453433157
"""

init_measurement_error_rate = 0.001

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p * 0.05
    p_erasure = p * (1 - 0.05)
    error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate}}}'
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
