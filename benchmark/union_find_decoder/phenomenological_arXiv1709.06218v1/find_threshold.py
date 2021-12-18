import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --error_model PauliZandErasurePhenomenological".split(" ")

# result:
"""
configuration 1:
0.0255941889 11 11 1000315 62414 0.06239434578107896 11 7.6e-3 0 0.025594188946101954
0.0257218416 11 11 1000309 64883 0.06486295734618003 11 7.4e-3 0 0.02572184155317918
0.0258501308 11 11 1000323 66651 0.06662947867838688 11 7.3e-3 0 0.02585013083556297
0.02597906 11 11 1000303 69160 0.06913905086758712 11 7.2e-3 0 0.025979059968710964
0.0261086321 11 11 1000323 71143 0.07112002823088143 11 7.1e-3 0 0.0261086321439186
configuration 2:
0.0255941889 15 15 367744 22717 0.06177395144448312 15 1.3e-2 0 0.025594188946101954
0.0257218416 15 15 366225 23669 0.06462966755409925 15 1.2e-2 0 0.02572184155317918
0.0258501308 15 15 367996 25002 0.06794095587995522 15 1.2e-2 0 0.02585013083556297
0.02597906 15 15 366961 25914 0.07061785857352688 15 1.2e-2 0 0.025979059968710964
0.0261086321 15 15 366920 26882 0.07326392674152404 15 1.2e-2 0 0.0261086321439186
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p0', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '2400', '--error_model', 'PauliZandErasurePhenomenological']
threshold = 0.025703777374466955
relative_confidence_interval = 0.004468744146356419
"""

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p
    p_erasure = 0
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
    return error_rate, confidence_interval, full_result + f" {p}"


evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters, simulator_runner=simulator_runner)
evaluator.searching_lower_bound = 0.02
evaluator.searching_upper_bound = 0.03
evaluator.target_threshold_accuracy = 0.01
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
print("\n\n")
