import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout


# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [3] [0] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3,1e-3,5e-4,2e-4,1e-4,5e-5,2e-5,1e-5] -p0-m100000000 --shallow_error_on_bottom --decoder UF --max_half_weight 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [5] [0] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0-m100000000 --shallow_error_on_bottom --decoder UF --max_half_weight 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [7] [0] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0-m100000000 --shallow_error_on_bottom -e1000 --decoder UF --max_half_weight 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [9] [0] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0-m100000000 --shallow_error_on_bottom -e1000 --decoder UF --max_half_weight 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [11] [0] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0-m100000000 --shallow_error_on_bottom -e1000 --decoder UF --max_half_weight 10
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [13] [0] [5e-1,2e-1,1e-1,5e-2,2e-2,1e-2,5e-3,2e-3] -p0-m100000000 --shallow_error_on_bottom -e1000 --decoder UF --max_half_weight 10


di_vec = [3, 5, 7, 9, 11, 13]
p_vec = [0.5 * (10 ** (- i / 10)) for i in range(10 * 2 + 1)]
print(p_vec)
min_error_cases = 1000

# debug configurations
# di_vec = [3, 5]
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(3)]
# min_error_cases = 100

max_N = 100000000

UF_parameters = f"-p0 --shallow_error_on_bottom --decoder UF --max_half_weight 10 --time_budget 3600".split(" ")

results = []
for di in di_vec:
    for p in p_vec:
        p_pauli = p * 0.05
        p_erasure = p * 0.95
        UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [0], UF_parameters + ["--log_runtime_statistics", "target/pm_decoding_time_multiple_p.txt", "--pes", f"[{p_erasure}]"], max_N=max_N, min_error_cases=min_error_cases)
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
