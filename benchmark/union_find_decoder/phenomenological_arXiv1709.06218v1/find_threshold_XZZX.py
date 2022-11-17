import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
noise_model_configuration = f'{{"also_include_pauli_x":true}}'
parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --noise_model PauliZandErasurePhenomenological".split(" ") + ["--noise_model_configuration", noise_model_configuration]

# result:
"""
configuration 1:
0.0254325523 11 11 1000968 117771 0.11765710791953389 11 5.4e-3 0 0.025432552274664386
0.0255593987 11 11 1000978 120523 0.12040524367168909 11 5.3e-3 0 0.025559398708803225
0.0256868778 11 11 1000950 125063 0.12494430291223338 11 5.2e-3 0 0.025686877797411023
0.0258149927 11 11 1000930 128259 0.12813982995813894 11 5.1e-3 0 0.02581499269589125
0.0259437466 11 11 1000986 132513 0.13238247088370866 11 5.0e-3 0 0.025943746575385136
configuration 2:
0.0254325523 15 15 445778 50708 0.11375168805997604 15 8.2e-3 0 0.025432552274664386
0.0255593987 15 15 447451 53076 0.1186185749948039 15 8.0e-3 0 0.025559398708803225
0.0256868778 15 15 447421 55834 0.12479074518183099 15 7.8e-3 0 0.025686877797411023
0.0258149927 15 15 442352 57196 0.12929974319094295 15 7.6e-3 0 0.02581499269589125
0.0259437466 15 15 442183 60062 0.1358306402552789 15 7.4e-3 0 0.025943746575385136
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--noise_model', 'PauliZandErasurePhenomenological', '--noise_model_configuration', '{"also_include_pauli_x":true}']
threshold = 0.025714973122800536
relative_confidence_interval = 0.0014916472761219658
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
