import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [3, 5, 7, 9, 11, 13]
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
print(p_vec)
min_error_cases = 1000

# debug configurations
# di_vec = [3, 5]
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(3)]
# min_error_cases = 100

max_N = 100000000

UF_parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 1200 --use_xzzx_code --noise_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")  # a maximum 20min for each point

results = []
for di in di_vec:
    local_results = []
    for p in p_vec:
        p_pauli = p * 0.05
        p_erasure = p * 0.95
        UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [di], UF_parameters + ["--pes", f"[{p_erasure}]"], max_N=max_N, min_error_cases=min_error_cases)
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
        local_results.append(print_result)
        results.append(print_result)
        print(print_result)

        if error_count < 100:
            break  # next is not trust-worthy, ignore every p behind it

    print("\n\n")
    print("\n".join(local_results))
    print("\n\n")

    results.append("")

print("\n\n")
print("\n".join(results))
