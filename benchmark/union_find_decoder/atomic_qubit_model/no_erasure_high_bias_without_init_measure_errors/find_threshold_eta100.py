import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
error_model_configuration = f'{{"initialization_error_rate":0,"measurement_error_rate":0}}'
parameters = "-p0 --decoder UF --max_half_weight 100 --time_budget 3600 --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta 100".split(" ") + ["--error_model_configuration", error_model_configuration]

# result:
"""
configuration 1:
0.0105053702 11 11 1000301 88409 0.08838239689853354 11 6.3e-3 0 0.010505370235827647
0.0105577664 11 11 1000308 90061 0.0900332697529161 11 6.2e-3 0 0.010557766422389328
0.0106104239 11 11 1000286 92499 0.09247255284988494 11 6.1e-3 0 0.010610423938185922
0.0106633441 11 11 1000294 95029 0.09500106968551246 11 6.0e-3 0 0.01066334408661322
0.0107165282 11 11 1000306 97070 0.09704030566646606 11 6.0e-3 0 0.010716528177567781
configuration 2:
0.0105053702 15 15 735035 63061 0.08579319352139694 15 7.5e-3 0 0.010505370235827647
0.0105577664 15 15 578434 51014 0.08819329430842586 15 8.3e-3 0 0.010557766422389328
0.0106104239 15 15 445647 40392 0.09063675958774546 15 9.3e-3 0 0.010610423938185922
0.0106633441 15 15 962171 90903 0.09447696927053507 15 6.2e-3 0 0.01066334408661322
0.0107165282 15 15 965837 94132 0.09746157995603813 15 6.1e-3 0 0.010716528177567781
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p30', '--decoder', 'UF', '--max_half_weight', '100', '--time_budget', '7200', '--use_xzzx_code', '--error_model', 'GenericBiasedWithBiasedCX', '--bias_eta', '100', '--error_model_configuration', '{"initialization_error_rate":0,"measurement_error_rate":0}']
threshold = 0.010704439659780856
relative_confidence_interval = 0.004424250121153981
"""

"""
the gate infidelity is p * (2 + 12/eta)
1.07 * (2+0.12) = 2.27
"""

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [dj], [T], parameters, max_N, min_error_cases)
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
evaluator.searching_upper_bound = 0.03
evaluator.target_threshold_accuracy = 0.01
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
print("\n\n")
