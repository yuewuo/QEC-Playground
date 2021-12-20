import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")

# result:
"""
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevel']
threshold = 0.03149750906360633
relative_confidence_interval = 0.0038001285503755367
"""

"""    !!!! wrong result! code has bug: mistakenly add initialization errors and measurement errors to data qubits

pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.01014903001434555
relative_confidence_interval = 0.0019504618662941182
"""

"""
configuration 1:
0.000520453282 11 11 1000624 32666 0.03264562912742449 11 1.1e-2 0.02550221079788584 0.026022664079475344
0.000523049075 11 11 1000593 34235 0.034214710676568796 11 1.0e-2 0.025629404658235877 0.026152453732893752
0.000525657814 11 11 1000610 36096 0.036073994863133486 11 1.0e-2 0.02575723290586469 0.026282890720270093
0.000528279565 11 11 1000616 37518 0.03749490313966596 11 9.9e-3 0.02588569870481823 0.026413978270222686
0.000530914393 11 11 1000581 39662 0.03963896975857027 11 9.6e-3 0.026014805234923336 0.02654571962747279
configuration 2:
0.000520453282 15 15 271822 8611 0.03167881922728845 15 2.1e-2 0.02550221079788584 0.026022664079475344
0.000523049075 15 15 270738 9236 0.034114162031188826 15 2.0e-2 0.025629404658235877 0.026152453732893752
0.000525657814 15 15 270073 9778 0.036205026048512806 15 1.9e-2 0.02575723290586469 0.026282890720270093
0.000528279565 15 15 264085 10173 0.038521688092848894 15 1.9e-2 0.02588569870481823 0.026413978270222686
0.000530914393 15 15 269108 10974 0.04077916672859967 15 1.8e-2 0.026014805234923336 0.02654571962747279
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.026234811528762263
relative_confidence_interval = 0.00520590392973419
"""

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p
    init_measurement_error_rate = p
    error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate},"use_correlated_pauli":true}}'
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [dj], [T], parameters + ["--error_model_configuration", error_model_configuration], max_N, min_error_cases)
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
evaluator.searching_upper_bound = 0.05
evaluator.target_threshold_accuracy = 0.01
threshold, relative_confidence_interval = evaluator.evaluate_threshold()
print(f"pair: {pair}")
print(f"parameters: {parameters}")
print(f"threshold = {threshold}")
print(f"relative_confidence_interval = {relative_confidence_interval}")
print("\n\n")
