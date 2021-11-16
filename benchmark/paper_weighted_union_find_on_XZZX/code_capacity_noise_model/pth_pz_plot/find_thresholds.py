import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 0), (15, 15, 0) ]  # (di, dj, T)
parameters = "-p0 --decoder UF --max_half_weight 100 --time_budget 3600 --use_xzzx_code --shallow_error_on_bottom".split(" ")
bias_eta_vec = [0.5] + [1 * (10 ** (i / 4)) for i in range(4 * 4 + 1)] + ["+inf"]
p_constant = 0.1


# customize simulator runner
def build_simulator_runner(bias_eta):
    def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
        di, dj, T = pair_one
        min_error_cases = min_error_cases if is_rough_test else max_N
        command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [dj], [T], parameters + ["--bias_eta", f"{bias_eta}"], max_N, min_error_cases)
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
    return simulator_runner

print(f"pair: {pair}")
print(f"parameters: {parameters}")

results = []

for bias_eta in bias_eta_vec:
    evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters, simulator_runner=build_simulator_runner(bias_eta))
    evaluator.searching_lower_bound = 0.1
    evaluator.searching_upper_bound = 0.5
    evaluator.target_threshold_accuracy = 0.01
    threshold, relative_confidence_interval = evaluator.evaluate_threshold()
    px = p_constant / (1. + float(bias_eta)) / 2.
    pz = p_constant - 2. * px
    print_result = f"{bias_eta} {pz} {threshold} {relative_confidence_interval} {pz/threshold}"
    results.append(print_result)
    print(f"\n\n{print_result}\n\n")

filename = os.path.join(os.path.dirname(__file__), f"thresholds.txt")
with open(filename, "w", encoding="utf-8") as f:
    f.write("\n".join(results) + "\n")
