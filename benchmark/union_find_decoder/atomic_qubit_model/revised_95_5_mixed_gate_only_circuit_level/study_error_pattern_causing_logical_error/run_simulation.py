import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

p = 0.023  # so that d=9 gives logical error rate ~ 1%
di_vec = [3, 5, 7, 9]
min_error_cases = 50000


"""
test commands:

RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [9] [9] [0.00115] -p1 -m100000000 --use_xzzx_code --error_model OnlyGateErrorCircuitLevel -e10 --decoder UF --max_half_weight 10 --time_budget 3600 --pes [0.02185] --log_runtime_statistics target/study_error_pattern_causing_logical_error_d_9.txt --log_error_pattern_into_statistics_when_has_logical_error
"""


max_N = 100000000

UF_parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 3600 --use_xzzx_code --error_model OnlyGateErrorCircuitLevel".split(" ")  # a maximum 20min for each point

results = []
for di in di_vec:
    log_filename = f"target/study_error_pattern_causing_logical_error_d_{di}.txt"
    p_pauli = p * 0.05
    p_erasure = p * 0.95
    UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [di], UF_parameters + ["--pes", f"[{p_erasure}]"] + ["--log_runtime_statistics", log_filename, "--log_error_pattern_into_statistics_when_has_logical_error", "--mini_sync_time", "1"], max_N=max_N, min_error_cases=min_error_cases)
    print(" ".join(UF_command))

    # run experiment
    stdout, returncode = run_qec_playground_command_get_stdout(UF_command)
    print("\n" + stdout)
    assert returncode == 0, "command fails..."

    # full result
    full_result = stdout.strip(" \r\n").split("\n")[-1]
    lst = full_result.split(" ")
    error_count = int(lst[4])
    error_rate = float(lst[5])
    confidence_interval = float(lst[7])

    # record result
    print_result = f"{p} " + full_result
    results.append(print_result)
    print(print_result)

    if error_count < 100:
        break  # next is not trust-worthy, ignore every p behind it

    results.append("")

print("\n\n")
print("\n".join(results))
