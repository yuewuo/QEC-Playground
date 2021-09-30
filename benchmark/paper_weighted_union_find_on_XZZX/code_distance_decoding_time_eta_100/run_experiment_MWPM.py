import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

pairs = [ (3, 9, 9), (4, 12, 12), (5, 15, 15), (6, 18, 18), (7, 21, 21), (8, 24, 24) ]  # (di, dj, T)
p = 0.008

di_vec = [e[0] for e in pairs]
dj_vec = [e[1] for e in pairs]
T_vec = [e[2] for e in pairs]
p_vec = [p]

max_N = 100000000  # this is rarely achieved because p is large enough

# min_error_cases = 100  # for debugging
min_error_cases = 2000  # real experiment

ENABLE_MULTITHREADING = True
num_threads = os.cpu_count() - 2 if ENABLE_MULTITHREADING else 1
print(num_threads)

MWPM_parameters = f"-p{num_threads} --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta 100".split(" ")
MWPM_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, MWPM_parameters + ["--log_runtime_statistics", "target/decoding_time_MWPM.txt"], max_N=max_N, min_error_cases=min_error_cases)
print(" ".join(MWPM_command))

# MWPM
print("MWPM running...")
stdout, returncode = run_qec_playground_command_get_stdout(MWPM_command, no_stdout=True)
# print("\n" + stdout)
assert returncode == 0, "command fails..."
