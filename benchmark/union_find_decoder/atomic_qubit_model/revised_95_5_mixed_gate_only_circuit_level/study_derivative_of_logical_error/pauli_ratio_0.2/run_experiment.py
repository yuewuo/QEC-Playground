import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [3,5,7][::-1]
p_threshold = 0.017897  # threshold error rate, which is the starting point of evaluation
p_vec = [p_threshold * (0.95 ** i) for i in range(200)]
print(p_vec)
min_error_cases = 4000  # this will have 3% uncertainty in the logical error

# debug configurations
# di_vec = [3, 5]
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(3)]
# min_error_cases = 100

max_N = 100000000

UF_parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget 3600 --use_xzzx_code --error_model OnlyGateErrorCircuitLevel".split(" ")  # a maximum 20min for each point

results = []
for di in di_vec:
    local_results = []
    for p in p_vec:
        p_pauli = p * 0.20
        p_erasure = p * 0.80
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

        if error_count < min_error_cases:
            break  # to avoid further time if reaching maximum time budget

    print("\n\n")
    print("\n".join(local_results))
    print("\n\n")

    results.append("")

print("\n\n")
print("\n".join(results))