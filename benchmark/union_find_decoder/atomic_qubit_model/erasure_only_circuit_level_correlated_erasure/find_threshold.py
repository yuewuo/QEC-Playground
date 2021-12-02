import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure --error_model_configuration {{\"use_correlated_pauli\":true}}".split(" ")

# result:
"""
configuration 1:
0 11 11 100019 13527 0.13524430358231937 11 1.6e-2 0.05059051384540442
0 11 11 100019 14194 0.14191303652306062 11 1.5e-2 0.05084283717549087
0 11 11 100019 14797 0.14794189104070227 11 1.5e-2 0.05109641898385845
0 11 11 100019 15365 0.15362081204571132 11 1.5e-2 0.05135126554724577
0 11 11 100019 16248 0.16244913466441377 11 1.4e-2 0.051607383173697036
configuration 2:
0 15 15 20268 2583 0.12744227353463589 15 3.6e-2 0.05059051384540442
0 15 15 20349 2777 0.136468622536734 15 3.5e-2 0.05084283717549087
0 15 15 20419 2950 0.14447328468583182 15 3.3e-2 0.05109641898385845
0 15 15 20501 3184 0.15530949709770256 15 3.2e-2 0.05135126554724577
0 15 15 20643 3447 0.16698154338032262 15 3.0e-2 0.051607383173697036
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p0', '--decoder', 'UF', '--max_half_weight', '100', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.05130317876711061
relative_confidence_interval = 0.004698230713000575
"""

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=100000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([0], [di], [dj], [T], parameters + ["--pes", f"[{p}]"], max_N, min_error_cases)
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
evaluator.searching_lower_bound = 0.0515
evaluator.searching_upper_bound = 0.0515
evaluator.target_threshold_accuracy = 0.01
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
print("\n\n")
