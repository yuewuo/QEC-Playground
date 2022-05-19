import os, sys
from process_data import process_file
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

print("[warning] requiring at least 10GB memory to run because of too large code patch")

# evaluated on i9-9820X CPU with 32GB memory

T = 200
pairs = [ (d, d, T) for d in [3, 4, 5, 6, 8, 10, 12, 16, 20] ]  # (di, dj, T)
# p_vec = [0.0002, 0.0005, 0.001, 0.002, 0.004, 0.008]
p_vec = [0.0002]
# p_vec = [0.004]

di_vec = [e[0] for e in pairs]
dj_vec = [e[1] for e in pairs]
T_vec = [e[2] for e in pairs]

max_N = 1000

# time_field_name = lambda e: e["time_run_to_stable"]  # 2.6
# time_field_name = lambda e: e["time_build_correction"]  # 2.1
# time_field_name = lambda e: e["time_prepare_decoders"]  # 2.2
# time_field_name = lambda e: e["time_uf_grow"]  # 3.0
# time_field_name = lambda e: e["time_uf_grow"]  # 3.0
# time_field_name = lambda e: e["time_uf_merge"]  # 2.6
# time_field_name = lambda e: e["time_uf_remove"]  # 2.5
# time_field_name = lambda e: e["time_uf_update"]  # 2.7
# time_field_name = lambda e: e["count_node_visited"]  # 2.0
# time_field_name = lambda e: e["count_uf_grow"]  # 2.1


time_field_name = lambda e: e["elapsed"]["decode"] - e["time_prepare_decoders"]
# time_field_name = lambda e: e["elapsed"]["decode"] - e["time_prepare_decoders"] - e["time_build_correction"]
# remove prepare decoder because that's just initializing UF decoder, which can be efficiently done by deliberately design the paging and use copy-on-write scheme
# this time can be subtracted from overall time, because it's always O(N) and since p can be very small, including this part may shallow other parts

for p in p_vec:

    log_filepath = os.path.join(os.path.dirname(__file__), f"runtime_statistics_UF_{p}.txt")

    if 'ONLY_PROCESS_DATA' in os.environ and os.environ["ONLY_PROCESS_DATA"] == "TRUE":
        content = process_file(log_filepath, pairs, time_field_name, starting_d=8)
        print(content, end="")
        with open(os.path.join(os.path.dirname(__file__), f"processed_UF_{p}.txt"), "w", encoding="utf8") as f:
            f.write(content)
        continue

    ENABLE_MULTITHREADING = False
    num_threads = os.cpu_count() / 2 if ENABLE_MULTITHREADING else 1
    print(num_threads)

    # UF_parameters = f"-p{num_threads} --code_type StandardXZZXCode --error_model generic-biased-with-biased-cx --bias_eta 100 --decoder union-find --decoder_config {{\"use_real_weighted\":true,\"max_half_weight\":1000000,\"benchmark_skip_building_correction\":true}}".split(" ")
    UF_parameters = f"-p{num_threads} --error_model phenomenological --decoder union-find --decoder_config {{\"use_real_weighted\":true,\"max_half_weight\":1000000,\"benchmark_skip_building_correction\":true}}".split(" ")
    UF_command = qec_playground_benchmark_simulator_runner_vec_command([p], di_vec, dj_vec, T_vec, UF_parameters + ["--log_runtime_statistics", log_filepath], max_N=max_N, min_error_cases=max_N)
    print(" ".join(UF_command))

    # UF
    print("UF running...")
    stdout, returncode = run_qec_playground_command_get_stdout(UF_command, no_stdout = True)
    # print("\n" + stdout)
    assert returncode == 0, "command fails..."
