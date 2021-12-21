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
0.0097029703 11 11 1000548 82433 0.08238785145740135 11 6.5e-3 0 0.009702970297029703
0.00975136446 11 11 1000549 84877 0.08483042809497586 11 6.4e-3 0 0.009751364464057893
0.0098 11 11 1000543 86426 0.08637909615079012 11 6.4e-3 0 0.009799999999999998
0.00984887811 11 11 1000530 89477 0.08942960231077529 11 6.3e-3 0 0.00984887810869847
0.009898 11 11 1000538 91181 0.09113197099960221 11 6.2e-3 0 0.009897999999999997
configuration 2:
0.0097029703 15 15 494978 40159 0.08113289883590785 15 9.4e-3 0 0.009702970297029703
0.00975136446 15 15 484958 40761 0.08405057757579007 15 9.3e-3 0 0.009751364464057893
0.0098 15 15 492842 42536 0.08630757930533518 15 9.1e-3 0 0.009799999999999998
0.00984887811 15 15 664913 59247 0.08910489041423464 15 7.7e-3 0 0.00984887810869847
0.009898 15 15 485273 44924 0.09257469506854904 15 8.8e-3 0 0.009897999999999997
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '100', '--time_budget', '3600', '--use_xzzx_code', '--error_model', 'GenericBiasedWithBiasedCX', '--bias_eta', '100', '--error_model_configuration', '{"initialization_error_rate":0}']
threshold = 0.00982052141533423
relative_confidence_interval = 0.004381348619072411
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
