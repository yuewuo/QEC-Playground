import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [11, 13]
p = 0.1
# bias_eta_vec = [0.5, 1, 3, 10, 30, 100, 300, 1000, 3000, 10000, "+inf"]
bias_eta_vec = [0.5] + [1 * (10 ** (i / 4)) for i in range(4 * 4 + 1)] + ["+inf"]
min_error_cases = 6000
# min_error_cases = 10  # debug

max_N = 100000000

UF_parameters = f"-p0 --decoder UF --max_half_weight 100 --time_budget 3600 --use_xzzx_code --shallow_error_on_bottom --only_count_logical_z".split(" ")  # a maximum 20min for each point
MWPM_parameters = f"-p0 --time_budget 3600 --use_xzzx_code --shallow_error_on_bottom".split(" ")

for (filename_prefix, paramters) in [("UF", UF_parameters), ("MWPM", MWPM_parameters)]:
    for di in di_vec:
        filename = os.path.join(os.path.dirname(__file__), f"ZL_{filename_prefix}_d{di}_p{p}.txt")
        
        results = []
        for bias_eta in bias_eta_vec:
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [di], [0], paramters + ["--bias_eta", f"{bias_eta}"], max_N=max_N, min_error_cases=min_error_cases)
            print(" ".join(command))

            # run experiment
            stdout, returncode = run_qec_playground_command_get_stdout(command)
            print("\n" + stdout)
            assert returncode == 0, "command fails..."

            # full result
            full_result = stdout.strip(" \r\n").split("\n")[-1]
            lst = full_result.split(" ")
            total_rounds = int(lst[3])
            error_count = int(lst[4])
            error_rate = float(lst[5])
            confidence_interval = float(lst[7])

            # record result
            print_result = f"{bias_eta} {p} {di} {total_rounds} {error_count} {error_rate} {confidence_interval}"
            results.append(print_result)
            print(print_result)
        
        print("\n\n")
        print("\n".join(results))
        print("\n\n")

        with open(filename, "w", encoding="utf-8") as f:
            f.write("\n".join(results) + "\n")
