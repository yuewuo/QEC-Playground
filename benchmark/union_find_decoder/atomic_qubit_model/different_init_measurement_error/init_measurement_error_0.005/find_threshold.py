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
0.0009991823120773914 11 11 659011 69991 0.10620611795554247 11 7.0e-3 0.019983646241547828 0.005
0.001004165795920178 11 11 652771 70615 0.10817729341530184 11 7.0e-3 0.02008331591840356 0.005
0.001009174135198165 11 11 659564 72789 0.11035926763740896 11 6.9e-3 0.0201834827039633 0.005
0.0010142074538793797 11 11 663877 74896 0.11281607888208207 11 6.7e-3 0.02028414907758759 0.005
0.0010192658765501468 11 11 645894 74934 0.11601594069615138 11 6.7e-3 0.020385317531002936 0.005
configuration 2:
0.0009991823120773914 15 15 189519 19577 0.10329835003350588 15 1.3e-2 0.019983646241547828 0.005
0.001004165795920178 15 15 194757 21004 0.1078472147342586 15 1.3e-2 0.02008331591840356 0.005
0.001009174135198165 15 15 190263 20830 0.10948003552976669 15 1.3e-2 0.0201834827039633 0.005
0.0010142074538793797 15 15 195904 22177 0.11320340574975499 15 1.2e-2 0.02028414907758759 0.005
0.0010192658765501468 15 15 184971 21690 0.11726162479523818 15 1.3e-2 0.020385317531002936 0.005
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '600', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevel']
threshold = 0.02024608702437262
relative_confidence_interval = 0.00997593693533955
"""

init_measurement_error_rate = 0.005

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
