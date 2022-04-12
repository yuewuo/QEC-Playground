import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(__file__), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

dj_vec = [3 + 2 * i for i in range(20)]
p = 0.2
bias_eta = 50
min_error_cases = 100000
# min_error_cases = 10  # debug

max_N = 100000000

UF_parameters = f"-p0 --decoder UF --max_half_weight 100 --time_budget 1800 --use_xzzx_code --shallow_error_on_bottom --only_count_logical_z".split(" ")  # a maximum 20min for each point
MWPM_parameters = f"-p0 --time_budget 1800 --use_xzzx_code --shallow_error_on_bottom --only_count_logical_z".split(" ")

# for (filename_prefix, paramters) in [("UF", UF_parameters), ("MWPM", MWPM_parameters)]:
for (filename_prefix, paramters) in [("MWPM", MWPM_parameters)]:
    results = []
    filename = os.path.join(os.path.dirname(__file__), f"{filename_prefix}_p{p}.txt")

    for dj in dj_vec:

        command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [3], [dj], [0], paramters + ["--bias_eta", f"{bias_eta}"], max_N=max_N, min_error_cases=min_error_cases)
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
        print_result = f"{bias_eta} {p} {dj} {total_rounds} {error_count} {error_rate} {confidence_interval}"
        results.append(print_result)
        print(print_result)
        
        if error_count < min_error_cases * 0.8:
            break  # X logical error is rare when bias is high
    
    print("\n\n")
    print("\n".join(results))
    print("\n\n")

    with open(filename, "w", encoding="utf-8") as f:
        f.write("\n".join(results) + "\n")
