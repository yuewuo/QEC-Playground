import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

pairs = [ (4, 12, 12), (5, 15, 15), (6, 18, 18), (7, 21, 21), (8, 24, 24) ]  # (di, dj, T)
p_vec = [0.01 * (10 ** (- i / 2)) for i in range(6)]

di_vec = [e[0] for e in pairs]
dj_vec = [e[1] for e in pairs]
T_vec = [e[2] for e in pairs]

max_N = 100000  # this is rarely achieved because p is large enough

# min_error_cases = 100  # for debugging
min_error_cases = max_N  # all error cases

ENABLE_MULTITHREADING = True
num_threads = os.cpu_count() - 3 if ENABLE_MULTITHREADING else 1
print(num_threads)

UF_parameters = f"-p{num_threads} --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta 1 --decoder UF --max_half_weight 1".split(" ")
UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, UF_parameters + ["--log_runtime_statistics", "target/decoding_time_UF_multiple_p_max_half_weight_1_no_bias.txt"], max_N=max_N, min_error_cases=min_error_cases)
print(" ".join(UF_command))

# UF
print("UF running...")
stdout, returncode = run_qec_playground_command_get_stdout(UF_command)
print("\n" + stdout)
assert returncode == 0, "command fails..."
