import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = "-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --error_model ErasureOnlyPhenomenological".split(" ")

# result:
"""
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'ErasureOnlyPhenomenological']
threshold = 0.25179614275251894
relative_confidence_interval = 0.0016654951344104603
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
    return error_rate, confidence_interval, full_result


evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters, simulator_runner=simulator_runner)
evaluator.searching_lower_bound = 0.1
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
print("\n\n")
