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
0.000756548398 11 11 1000492 42193 0.04217225125238383 11 9.3e-3 0.03707087152426261 0.03782741992271695 0.005
0.000760321731 11 11 1000453 44775 0.04475472610907259 11 9.1e-3 0.03725576479853915 0.03801608652912158 0.005
0.000764113882 11 11 1000460 46326 0.04630469983807448 11 8.9e-3 0.03744158023950523 0.03820569412194411 0.005
0.000767924948 11 11 1000464 48040 0.048017719778023 11 8.7e-3 0.03762832244652453 0.03839624739441278 0.005
0.000771755021 11 11 1000496 50323 0.0502980521661256 11 8.5e-3 0.03781599604190028 0.038587751063163554 0.005
configuration 2:
0.000756548398 15 15 381163 15894 0.04169869583354103 15 1.5e-2 0.03707087152426261 0.03782741992271695 0.005
0.000760321731 15 15 381672 17117 0.04484740824582364 15 1.5e-2 0.03725576479853915 0.03801608652912158 0.005
0.000764113882 15 15 377675 17584 0.04655854901701198 15 1.4e-2 0.03744158023950523 0.03820569412194411 0.005
0.000767924948 15 15 381492 18922 0.04959999161188177 15 1.4e-2 0.03762832244652453 0.03839624739441278 0.005
0.000771755021 15 15 380245 20008 0.05261870636037292 15 1.3e-2 0.03781599604190028 0.038587751063163554 0.005
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p0', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.03800096976339541
relative_confidence_interval = 0.0037837726837547055
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
