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
0.0004626184864409159 11 11 671240 84200 0.12543948513199452 11 6.3e-3 0.009252369728818318 0.01
0.0004649258248762406 11 11 685549 86596 0.12631628082018936 11 6.2e-3 0.009298516497524811 0.01
0.000467244671305325 11 11 672857 85672 0.12732571705429238 11 6.3e-3 0.009344893426106499 0.01
0.0004695750831250028 11 11 679361 87960 0.12947460922837783 11 6.2e-3 0.009391501662500057 0.01
0.00047191711801837825 11 11 661000 86004 0.13011195158850228 11 6.2e-3 0.009438342360367565 0.01
configuration 2:
0.0004626184864409159 15 15 194010 23716 0.12224112159167053 15 1.2e-2 0.009252369728818318 0.01
0.0004649258248762406 15 15 189855 23773 0.12521661267809645 15 1.2e-2 0.009298516497524811 0.01
0.000467244671305325 15 15 195990 24962 0.12736364100209194 15 1.2e-2 0.009344893426106499 0.01
0.0004695750831250028 15 15 196219 25112 0.12797945153119727 15 1.2e-2 0.009391501662500057 0.01
0.00047191711801837825 15 15 181722 23898 0.1315085680324892 15 1.2e-2 0.009438342360367565 0.01
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '600', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevel']
threshold = 0.00939940368795293
relative_confidence_interval = 0.008265493750823305
"""

init_measurement_error_rate = 0.01

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
