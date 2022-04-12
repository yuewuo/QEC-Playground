import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(__file__), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
error_model_configuration = f'{{"initialization_error_rate":0,"measurement_error_rate":0}}'
parameters = "-p0 --decoder UF --max_half_weight 100 --time_budget 3600 --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta +inf".split(" ") + ["--error_model_configuration", error_model_configuration]

# result:
"""
configuration 1:
0.0181301723 11 11 1000276 217739 0.21767892061790944 11 3.7e-3 0 0.01813017227108264
0.0182205976 11 11 1000291 221843 0.22177846246742197 11 3.7e-3 0 0.018220597631387538
0.018311474 11 11 1000293 225634 0.2255679086027794 11 3.6e-3 0 0.018311473993793462
0.0184028036 11 11 1000293 229125 0.22905788603939045 11 3.6e-3 0 0.01840280360770141
0.0184945887 11 11 1000301 234211 0.2341405237023656 11 3.5e-3 0 0.018494588733731394
configuration 2:
0.0181301723 15 15 1000107 213730 0.21370713333673297 15 3.8e-3 0 0.01813017227108264
0.0182205976 15 15 1000107 219180 0.21915655024912334 15 3.7e-3 0 0.018220597631387538
0.018311474 15 15 1000110 224323 0.22429832718400977 15 3.6e-3 0 0.018311473993793462
0.0184028036 15 15 1000112 229084 0.22905834546530787 15 3.6e-3 0 0.01840280360770141
0.0184945887 15 15 1000142 233980 0.23394677955730286 15 3.5e-3 0 0.018494588733731394
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p30', '--decoder', 'UF', '--max_half_weight', '100', '--time_budget', '7200', '--use_xzzx_code', '--error_model', 'GenericBiasedWithBiasedCX', '--bias_eta', '+inf', '--error_model_configuration', '{"initialization_error_rate":0,"measurement_error_rate":0}']
threshold = 0.018460486769687697
relative_confidence_interval = 0.0039811089995457575
"""

"""
the gate infidelity is p * (2 + 12/eta)
1.846 * 2 = 3.69
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
