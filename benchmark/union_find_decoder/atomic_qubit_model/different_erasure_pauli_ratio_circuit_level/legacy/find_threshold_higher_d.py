import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (15, 15, 15), (19, 19, 19) ]  # (di, dj, T)
# the speed of 15 and 19 is about 1:3, thus the time for them is different
# I have time budgets of 3 days, which is 259200s; there are 25 points, each point requires 5 different p, that's 125 cases, each has 2000s
# d=15 has 500s budget, d=19 has 1500s budget
parameters_basic = "-p0 --decoder UF --max_half_weight 10 --use_xzzx_code --noise_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")
parameters_15 = parameters_basic + ["--time_budget", "500"]
parameters_19 = parameters_basic + ["--time_budget", "1500"]

# result:
"""

"""

# customize simulator runner
def make_simulator_runner(pauli_ratio):
    def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
        di, dj, T = pair_one
        min_error_cases = min_error_cases if is_rough_test else max_N
        p_pauli = p * pauli_ratio
        p_erasure = p * (1 - pauli_ratio)
        if di == 15:
            parameters = parameters_15
        elif di == 19:
            parameters = parameters_19
        else:
            assert 0, "unrecognized code distance"
        command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [dj], [T], parameters + ["--pes", f"[{p_erasure}]"], max_N, min_error_cases)
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
        return error_rate, confidence_interval, full_result + f" {p} {pauli_ratio}"
    return simulator_runner

results = []
with open("./thresholds.txt", "r", encoding="utf8") as f:
    for line in f.readlines():
        line = line.strip("\r\n ")
        if line == "":
            continue
        pauli_ratio, old_threshold, _relative_confidence_interval = line.split(" ")
        simulator_runner = make_simulator_runner(float(pauli_ratio))
        evaluator = AutomatedThresholdEvaluator(pair, parameters=None, simulator_runner=simulator_runner)
        evaluator.searching_lower_bound = float(old_threshold)
        evaluator.searching_upper_bound = float(old_threshold)
        evaluator.target_threshold_accuracy = 0.02
        threshold, relative_confidence_interval = evaluator.evaluate_threshold()
        print(f"pair: {pair}")
        print(f"parameters: {parameters_basic}")
        print(f"threshold = {threshold}")
        print(f"relative_confidence_interval = {relative_confidence_interval}")
        results.append(f"{pauli_ratio} {threshold} {relative_confidence_interval}")
        print("\n\n")

print("\n".join(results))

