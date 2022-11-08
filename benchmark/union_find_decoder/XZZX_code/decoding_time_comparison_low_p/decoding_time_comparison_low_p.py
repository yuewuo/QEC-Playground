import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

pairs = [ (3, 9, 9), (4, 12, 12), (5, 15, 15), (6, 18, 18), (7, 21, 21), (8, 24, 24) ]  # (di, dj, T)
p = 0.005

di_vec = [e[0] for e in pairs]
dj_vec = [e[1] for e in pairs]
T_vec = [e[2] for e in pairs]
p_vec = [p]

max_N = 100000000  # this is rarely achieved because p is large enough

# min_error_cases = 100  # for debugging
min_error_cases = 1000  # real experiment

ENABLE_MULTITHREADING = True
num_threads = os.cpu_count() - 3 if ENABLE_MULTITHREADING else 1
print(num_threads)

MWPM_parameters = f"-p{num_threads} --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta 100".split(" ")
UF_parameters = MWPM_parameters + "--decoder UF --max_half_weight 10".split(" ")
DUF_parameters = MWPM_parameters + "--decoder DUF --max_half_weight 10".split(" ")
MWPM_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, MWPM_parameters + ["--log_runtime_statistics", "target/decoding_time_MWPM_low_p.txt"], max_N=max_N, min_error_cases=min_error_cases)
print(" ".join(MWPM_command))
UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, UF_parameters + ["--log_runtime_statistics", "target/decoding_time_UF_low_p.txt"], max_N=max_N, min_error_cases=min_error_cases)
print(" ".join(UF_command))
DUF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, DUF_parameters + ["--log_runtime_statistics", "target/decoding_time_DUF_low_p.txt"], max_N=max_N, min_error_cases=min_error_cases)
print(" ".join(DUF_command))

# MWPM
print("MWPM running...")
stdout, returncode = run_qec_playground_command_get_stdout(MWPM_command)
print("\n" + stdout)
assert returncode == 0, "command fails..."

# UF
print("UF running...")
stdout, returncode = run_qec_playground_command_get_stdout(UF_command)
print("\n" + stdout)
assert returncode == 0, "command fails..."

# DUF
print("DUF running...")
stdout, returncode = run_qec_playground_command_get_stdout(DUF_command)
print("\n" + stdout)
assert returncode == 0, "command fails..."
