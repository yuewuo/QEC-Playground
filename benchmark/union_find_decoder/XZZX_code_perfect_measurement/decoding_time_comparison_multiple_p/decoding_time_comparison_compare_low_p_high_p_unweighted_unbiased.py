import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [int((dif+1)/2)*2-1 for dif in [5 * (1.2 ** i) for i in range(15)]]
# di_vec = [5, 7, 9, 11, 13, 17, 21, 25, 29, 37, 43, 53, 63]
dedup_di_vec = sorted(list(set(di_vec)))
pairs = [ (di, di, 0) for di in dedup_di_vec ]  # (di, dj, T)
p_vec = [0.01, 0.03, 0.1, 0.3]

di_vec = [e[0] for e in pairs]
dj_vec = [e[1] for e in pairs]
T_vec = [e[2] for e in pairs]

max_N = 100000  # this is rarely achieved because p is large enough

# min_error_cases = 100  # for debugging
min_error_cases = max_N  # all error cases

ENABLE_MULTITHREADING = True
num_threads = os.cpu_count() - 3 if ENABLE_MULTITHREADING else 1
print(num_threads)

UF_parameters = f"-b100 -p{num_threads} --shallow_error_on_bottom --use_xzzx_code --bias_eta 1 --decoder UF --max_half_weight 1".split(" ")
UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, UF_parameters + ["--log_runtime_statistics", "target/pm_decoding_time_compare_low_p_high_p_unweighted_unbiased.txt"], max_N=max_N, min_error_cases=min_error_cases)
print(" ".join(UF_command))

# UF
print("UF running...")
stdout, returncode = run_qec_playground_command_get_stdout(UF_command)
print("\n" + stdout)
assert returncode == 0, "command fails..."
