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
0.00068462351 11 11 1000586 50522 0.05049241144689212 11 8.5e-3 0.03354655199019948 0.03423117550020355 0.005
0.000688038112 11 11 1000608 52356 0.05232418689436823 11 8.3e-3 0.03371386750189702 0.03440190561418063 0.005
0.000691469745 11 11 1000560 54383 0.05435256256496362 11 8.2e-3 0.03388201751010147 0.03457348725520558 0.005
0.000694918493 11 11 1000590 56667 0.056633586184151354 11 8.0e-3 0.03405100617691598 0.03474592467032243 0.005
0.000698384443 11 11 1000569 58652 0.05861864599043144 11 7.9e-3 0.034220837685202475 0.03491922212775763 0.005
configuration 2:
0.00068462351 15 15 20855 958 0.045936226324622395 15 6.2e-2 0.03354655199019948 0.03423117550020355 0.005
0.000688038112 15 15 20693 1125 0.05436621079592133 15 5.7e-2 0.03371386750189702 0.03440190561418063 0.005
0.000691469745 15 15 20923 1123 0.05367299144482149 15 5.7e-2 0.03388201751010147 0.03457348725520558 0.005
0.000694918493 15 15 21087 1243 0.05894627021387585 15 5.4e-2 0.03405100617691598 0.03474592467032243 0.005
0.000698384443 15 15 20757 1326 0.06388206388206388 15 5.2e-2 0.034220837685202475 0.03491922212775763 0.005
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1800', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.034514934972253664
relative_confidence_interval = 0.004417488061841171
"""

init_measurement_error_rate = 0.005

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
