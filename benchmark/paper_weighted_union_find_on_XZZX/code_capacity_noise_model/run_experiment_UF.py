import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout


# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [11] [0] [0.1] -p0 -m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --decoder UF --max_half_weight 10 --bias_eta 0.5
# RUST_BACKTRACE=full cargo run --release -- tool fault_tolerant_benchmark [11] [0] [0.1] -p0 -m100000000 --use_xzzx_code --shallow_error_on_bottom -e1000000 --decoder UF --max_half_weight 10 --bias_eta +inf

di_vec = [11, 15]
p = 0.1
bias_eta_vec = [0.5] + [1 * pow(10, i / 3) for i in range(3 * 4 + 1)] + ["+inf"]
print(bias_eta_vec)
min_error_cases = 100000

max_N = 100000000

# UF_parameters = f"-p0 --shallow_error_on_bottom --use_xzzx_code --decoder UF --max_half_weight 10 --time_budget 3600".split(" ")

# for debug
UF_parameters = f"-p0 --shallow_error_on_bottom --use_xzzx_code --decoder UF --max_half_weight 10 --time_budget 10".split(" ")

results = []
for di in di_vec:
    for bias_eta in bias_eta_vec:
        UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [di], [0], UF_parameters + ["--bias_eta", f"{bias_eta}"], max_N=max_N, min_error_cases=min_error_cases)
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
        print_result = f"{bias_eta} " + full_result
        results.append(print_result)
        print(print_result)

        if error_count < 100:
            break  # next is not trust-worthy, ignore every p behind it

    results.append("")

print("\n\n")
print("\n".join(results))
