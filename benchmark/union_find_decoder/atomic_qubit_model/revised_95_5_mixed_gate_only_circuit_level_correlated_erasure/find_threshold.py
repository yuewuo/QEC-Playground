import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = "-p0 --decoder UF --max_half_weight 100 --time_budget 3600 --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")

# result:
"""
configuration 1:
0.0014981735720367264 11 11 469332 30960 0.06596609649459231 11 1.1e-2 0.029963471440734524
0.0015056458057819495 11 11 472786 32506 0.06875415092663488 11 1.0e-2 0.03011291611563899
0.0015131553077570932 11 11 468133 33558 0.0716847562551668 11 1.0e-2 0.030263106155141863
0.001520702263839769 11 11 468768 34560 0.07372516895351218 11 1.0e-2 0.030414045276795375
0.0015282868608346642 11 11 472733 36576 0.07737137030839819 11 9.8e-3 0.03056573721669328
configuration 2:
0.0014981735720367264 15 15 155268 9812 0.06319396140866115 15 1.9e-2 0.029963471440734524
0.0015056458057819495 15 15 155466 10667 0.0686130729548583 15 1.8e-2 0.03011291611563899
0.0015131553077570932 15 15 155361 11077 0.07129845971640245 15 1.8e-2 0.030263106155141863
0.001520702263839769 15 15 155197 11857 0.07639967267408519 15 1.7e-2 0.030414045276795375
0.0015282868608346642 15 15 155047 12091 0.07798280521390288 15 1.7e-2 0.03056573721669328
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p0', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.030277654518813864
relative_confidence_interval = 0.003600732842069607
"""

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p * 0.05
    p_erasure = p * 0.95
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
evaluator.searching_lower_bound = 0.0302
evaluator.searching_upper_bound = 0.0302
evaluator.target_threshold_accuracy = 0.01
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
print("\n\n")
