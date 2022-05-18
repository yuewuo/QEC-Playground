import os, sys
from process_data import process_file
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

T = 100
pairs = [ (d, 3*d, T) for d in [3, 4, 5, 6, 7, 8, 9, 10] ]  # (di, dj, T)
# pairs = [ (d, 3*d, T) for d in [3, 4, 5, 6, 7, 8] ]  # (di, dj, T)
p_vec = [0.0001, 0.0003, 0.001, 0.003]
# p_vec = [0.0001]
# p_vec = [0.003]

di_vec = [e[0] for e in pairs]
dj_vec = [e[1] for e in pairs]
T_vec = [e[2] for e in pairs]

max_N = 1000

# time_field_name = lambda e: e["time_blossom_v"]
time_field_name = lambda e: e["elapsed"]["decode"]

for p in p_vec:

    log_filepath = os.path.join(os.path.dirname(__file__), f"runtime_statistics_MWPM_{p}.txt")

    if 'ONLY_PROCESS_DATA' in os.environ and os.environ["ONLY_PROCESS_DATA"] == "TRUE":
        content = process_file(log_filepath, pairs, time_field_name, starting_d=5)
        print(content, end="")
        with open(os.path.join(os.path.dirname(__file__), f"processed_MWPM_{p}.txt"), "w", encoding="utf8") as f:
            f.write(content)
        continue

    ENABLE_MULTITHREADING = False
    num_threads = os.cpu_count() / 2 if ENABLE_MULTITHREADING else 1
    print(num_threads)

    # use `--parallel_init=0` to force multiple-core initialization
    MWPM_parameters = f"-p{num_threads} --parallel_init=0 --code_type StandardXZZXCode --error_model generic-biased-with-biased-cx --bias_eta 100 --decoder mwpm --decoder_config {{\"pcmg\":true}}".split(" ")
    MWPM_command = qec_playground_benchmark_simulator_runner_vec_command([p], di_vec, dj_vec, T_vec, MWPM_parameters + ["--log_runtime_statistics", log_filepath], max_N=max_N, min_error_cases=max_N)
    print(" ".join(MWPM_command))

    # MWPM
    print("MWPM running...")
    stdout, returncode = run_qec_playground_command_get_stdout(MWPM_command, no_stdout=True)
    # print("\n" + stdout)
    assert returncode == 0, "command fails..."
