import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [3, 5, 7, 9, 11, 13]
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
print(p_vec)
min_error_cases = 0

# debug configurations
# di_vec = [3, 5]
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(3)]
# min_error_cases = 100

max_N = 0

# test examples
# cargo run --release -- tool fault_tolerant_benchmark [3] --djs [3] [3] -m100000000 -e1000 [0] -p0 --decoder UF --max_half_weight 10 --time_budget 3600 --use_xzzx_code --pes [0.2] --error_model OnlyGateErrorCircuitLevel
# cargo run --release -- tool fault_tolerant_benchmark [5] --djs [5] [5] -m100000000 -e1000 [0] -p0 --decoder UF --max_half_weight 10 --time_budget 3600 --use_xzzx_code --pes [0.2] --error_model OnlyGateErrorCircuitLevel
# cargo run --release -- tool fault_tolerant_benchmark [7] --djs [7] [7] -m100000000 -e1000 [0] -p0 --decoder UF --max_half_weight 10 --time_budget 3600 --use_xzzx_code --pes [0.2] --error_model OnlyGateErrorCircuitLevel

time_budget = 180
# time_budget = 10  # debug
UF_parameters = f"-p0 --decoder UF --max_half_weight 10 --time_budget {time_budget} --use_xzzx_code --error_model OnlyGateErrorCircuitLevel --use_fast_benchmark".split(" ")  # a maximum 3min for each point

results = []
for di in di_vec:
    local_results = []
    filename = os.path.join(os.path.dirname(__file__), f"d_{di}_{di}.txt")
    for p in p_vec:
        p_pauli = 0
        p_erasure = p
        UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [di], UF_parameters + ["--pes", f"[{p_erasure}]"], max_N=max_N, min_error_cases=min_error_cases)
        print(" ".join(UF_command))

        # run experiment
        stdout, returncode = run_qec_playground_command_get_stdout(UF_command)
        print("\n" + stdout)
        assert returncode == 0, "command fails..."

        # full result
        full_result = stdout.strip(" \r\n").split("\n")[-1]
        lst = full_result.split(" ")
        error_rate = float(lst[8])
        confidence_interval = float(lst[8])

        # record result
        print_result = f"{p} " + full_result
        local_results.append(print_result)
        results.append(print_result)
        print(print_result)

    print("\n\n")
    print("\n".join(local_results))
    print("\n\n")

    with open(filename, "w", encoding="utf-8") as f:
        f.write("\n".join(local_results) + "\n")

    results.append("")

print("\n\n")
print("\n".join(results))
