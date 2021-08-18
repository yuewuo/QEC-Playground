import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di = 3
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
print(p_vec)
min_error_cases = 1000

# debug configurations
# di = 3
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(6)]
# min_error_cases = 100

max_N = 10000000000

UF_parameters = f"-p0 --shallow_error_on_bottom --decoder UF --max_half_weight 10 --time_budget 3600".split(" ")

results = []
for (pauli_ratio, erasure_ratio, name) in [(0.05, 0, "only_pauli"), (0, 0.95, "only_erasure"),(0.05, 0.95, "both")]:
    local_results = []
    results.append(name)
    local_results.append(name)
    for p in p_vec:

        p_pauli = p * pauli_ratio
        p_erasure = p * erasure_ratio
        UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [0], UF_parameters + ["--pes", f"[{p_erasure}]"], max_N=max_N, min_error_cases=min_error_cases)
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
