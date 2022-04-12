import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(__file__), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [di for di in range(3, 20, 2)]
print(di_vec)
p_vec = [0.02, 0.002]
print(p_vec)
min_error_cases = 3000

# debug configurations
# di_vec = [3, 5]
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(3)]
# min_error_cases = 100

max_N = 100000000

UF_parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 2400 --use_xzzx_code --error_model OnlyGateErrorCircuitLevel".split(" ")  # a maximum 20min for each point

global_results = []

for is_mixed in [True, False]:

    results = []
    for p in p_vec:
        local_results = []
        for di in di_vec:
            p_pauli = p * 0.05 if is_mixed else p
            p_erasure = p * 0.95 if is_mixed else 0
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

    global_results.append("\n".join(results))

for e in global_results:
    print("is_mixed:", is_mixed)
    print(e)
    print("\n\n\n")
