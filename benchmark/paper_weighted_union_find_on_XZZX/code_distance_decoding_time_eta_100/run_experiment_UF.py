import os, sys
from process_data import process_file
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(__file__), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

print("[warning] requiring at least 150GB memory to run because of too large code patch")

pairs = [ (3, 9, 9), (4, 12, 12), (5, 15, 15), (6, 18, 18), (7, 21, 21), (8, 24, 24), (9, 27, 27), (10, 30, 30), (11, 33, 33), (12, 36, 36) ]  # (di, dj, T)
p = 0.008

di_vec = [e[0] for e in pairs]
dj_vec = [e[1] for e in pairs]
T_vec = [e[2] for e in pairs]
p_vec = [p]

max_N = 100000000  # this is rarely achieved because p is large enough

# min_error_cases = 100  # for debugging
min_error_cases = 40000  # real experiment

log_filepath = os.path.join(os.path.dirname(__file__), f"runtime_statistics_UF.txt")
if 'ONLY_PROCESS_DATA' in os.environ and os.environ["ONLY_PROCESS_DATA"] == "TRUE":
    content = process_file(log_filepath, pairs, "time_run_to_stable")
    print(content, end="")
    with open(os.path.join(os.path.dirname(__file__), f"processed_UF.txt"), "w", encoding="utf8") as f:
        f.write(content)
    exit(0)

ENABLE_MULTITHREADING = True
num_threads = os.cpu_count() - 2 if ENABLE_MULTITHREADING else 1
print(num_threads)

UF_parameters = f"-p{num_threads} --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta 100 --decoder UF --max_half_weight 10".split(" ")
UF_command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, UF_parameters + ["--log_runtime_statistics", log_filepath], max_N=max_N, min_error_cases=min_error_cases)
print(" ".join(UF_command))

# UF
print("UF running...")
stdout, returncode = run_qec_playground_command_get_stdout(UF_command, no_stdout = True)
# print("\n" + stdout)
assert returncode == 0, "command fails..."
