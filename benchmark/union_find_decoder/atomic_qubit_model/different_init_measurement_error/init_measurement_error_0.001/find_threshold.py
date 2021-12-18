import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
parameters = "-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")

# result:
"""
configuration 1:
0.0014858678660484362 11 11 637285 55806 0.08756835638685988 11 7.9e-3 0.02971735732096872 0.001
0.0014932787243207098 11 11 633511 57891 0.0913812072718548 11 7.8e-3 0.029865574486414193 0.001
0.0015007265447089203 11 11 631393 59524 0.09427408919642759 11 7.6e-3 0.030014530894178403 0.001
0.0015082115115639166 11 11 627108 61558 0.09816172015027715 11 7.5e-3 0.03016423023127833 0.001
0.0015157338101560091 11 11 642282 65342 0.10173412924540934 11 7.3e-3 0.030314676203120186 0.001
configuration 2:
0.0014858678660484362 15 15 188384 16298 0.08651477832512315 15 1.5e-2 0.02971735732096872 0.001
0.0014932787243207098 15 15 189652 17301 0.09122498049058275 15 1.4e-2 0.029865574486414193 0.001
0.0015007265447089203 15 15 184430 17609 0.09547795911728027 15 1.4e-2 0.030014530894178403 0.001
0.0015082115115639166 15 15 184797 18679 0.10107848071126696 15 1.4e-2 0.03016423023127833 0.001
0.0015157338101560091 15 15 188651 19945 0.10572432693174168 15 1.3e-2 0.030314676203120186 0.001
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '600', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevel']
threshold = 0.029854638556969574
relative_confidence_interval = 0.0037588641453433157
"""

"""    !!!! wrong result! code has bug: mistakenly add initialization errors and measurement errors to data qubits

configuration 1:
0.000658529139 11 11 1000601 57306 0.05727157978055189 11 7.9e-3 0.03226792780649795 0.032926456945406066 0.001
0.000661813594 11 11 1000630 59587 0.05954948382519013 11 7.8e-3 0.03242886610066126 0.0330906796945523 0.001
0.00066511443 11 11 1000578 62116 0.06208011769197404 11 7.6e-3 0.03259060708456292 0.03325572151486012 0.001
0.00066843173 11 11 1000598 64182 0.06414364210202299 11 7.5e-3 0.03275315476166786 0.03342158649149782 0.001
0.000671765575 11 11 1000566 66584 0.06654633477451763 11 7.3e-3 0.03291651315540854 0.03358827873000872 0.001
configuration 2:
0.000658529139 15 15 359888 19236 0.053449962210465475 15 1.4e-2 0.03226792780649795 0.032926456945406066 0.001
0.000661813594 15 15 276040 15528 0.05625271699753659 15 1.5e-2 0.03242886610066126 0.0330906796945523 0.001
0.00066511443 15 15 267785 16154 0.06032451406912262 15 1.5e-2 0.03259060708456292 0.03325572151486012 0.001
0.00066843173 15 15 272062 17175 0.06312899265608575 15 1.4e-2 0.03275315476166786 0.03342158649149782 0.001
0.000671765575 15 15 280791 18585 0.06618801884675792 15 1.4e-2 0.03291651315540854 0.03358827873000872 0.001
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.03360633590361438
relative_confidence_interval = 0.004037809216847374
"""

"""
configuration 1:
0.000740382988 11 11 1000618 56102 0.056067350377466726 11 8.0e-3 0.03627876640410464 0.037019149391943514 0.001
0.000744075694 11 11 1000529 58556 0.05852504025370579 11 7.9e-3 0.03645970900489509 0.03720378469887253 0.001
0.000747786818 11 11 1000590 61075 0.06103898699767137 11 7.7e-3 0.03664155406814568 0.03738934088586294 0.001
0.000751516451 11 11 1000576 63620 0.06358337597543814 11 7.5e-3 0.03682430609494402 0.03757582254586125 0.001
0.000755264686 11 11 1000595 66054 0.06601472124086169 11 7.4e-3 0.03700796960882714 0.03776323429472157 0.001
configuration 2:
0.000740382988 15 15 267222 14598 0.05462873565799223 15 1.6e-2 0.03627876640410464 0.037019149391943514 0.001
0.000744075694 15 15 268731 15247 0.05673703443220172 15 1.5e-2 0.03645970900489509 0.03720378469887253 0.001
0.000747786818 15 15 344589 20778 0.0602979201309386 15 1.3e-2 0.03664155406814568 0.03738934088586294 0.001
0.000751516451 15 15 336716 21512 0.06388766794568716 15 1.3e-2 0.03682430609494402 0.03757582254586125 0.001
0.000755264686 15 15 269891 18124 0.06715303585521562 15 1.4e-2 0.03700796960882714 0.03776323429472157 0.001
pair: [(11, 11, 11), (15, 15, 15)]
parameters: ['-p60', '--decoder', 'UF', '--max_half_weight', '10', '--time_budget', '1200', '--use_xzzx_code', '--error_model', 'OnlyGateErrorCircuitLevelCorrelatedErasure']
threshold = 0.03753673562880721
relative_confidence_interval = 0.003703929880508001
"""

init_measurement_error_rate = 0.001

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    p_pauli = p * 0.02
    p_erasure = p * 0.98
    error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate},"use_correlated_pauli":true}}'
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
