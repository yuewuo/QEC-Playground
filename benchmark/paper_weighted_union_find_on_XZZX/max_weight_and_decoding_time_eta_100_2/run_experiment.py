
# python3 run_experiment.py
# PROCESS_DATA=1 python3 run_experiment.py

import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout
import json
import numpy as np
import math, random, scipy.stats

# evaluated on i9-9820X CPU with 32GB memory

d = 12
T = 150
p = 0.002
bias_eta_vec = ["10", "15", "30", "100", "1000", "inf"]
max_half_weight_vec = [1, 2, 3, 4, 5, 6, 7] + [8, 10, 12, 14] + [16, 20, 24, 28] + [2 ** i for i in range(5, 11)]

max_N = 30000 // T  # so that there are 30000 measurement rounds

# time_field_name = lambda e: e["time_run_to_stable"]  # 2.6
# time_field_name = lambda e: e["time_build_correction"]  # 2.9
# time_field_name = lambda e: e["time_prepare_decoders"]  # 2.0
# time_field_name = lambda e: e["time_uf_grow"]  # 2.7
# time_field_name = lambda e: e["time_uf_merge"]  # 2.2
# time_field_name = lambda e: e["time_uf_remove"]  # 2.0
# time_field_name = lambda e: e["time_uf_update"]  # 2.6
# time_field_name = lambda e: e["count_node_visited"]
# time_field_name = lambda e: e["count_uf_grow"]
# time_field_name = lambda e: e["count_iteration"]

# time_field_name = lambda e: e["elapsed"]["decode"]
# time_field_name = lambda e: e["elapsed"]["decode"] - e["time_prepare_decoders"]
# time_field_name = lambda e: e["elapsed"]["decode"] - e["time_prepare_decoders"] - e["time_build_correction"]
# remove prepare decoder because that's just initializing UF decoder, which can be efficiently done by deliberately design the paging and use copy-on-write scheme
# this time can be subtracted from overall time, because it's always O(N) and since p can be very small, including this part may shallow other parts


time_field_name = lambda e: [e["time_run_to_stable"], e["count_memory_access"]]

starting_max_half_weight = 64
ending_max_half_weight = 1000000

data_folder = os.path.join(os.path.dirname(__file__), "data")
if not os.path.exists(data_folder):
    os.makedirs(data_folder, exist_ok=False)

for real_weighted in [True, False]:
# for real_weighted in [True]:

    for bias_eta in bias_eta_vec:

        if 'PROCESS_DATA' in os.environ:
            
            fixed_configurations = []
            configurations = []
            data_vec = []
            for max_half_weight in max_half_weight_vec:
                log_filepath = os.path.join(data_folder, f"runtime_statistics_UF{'' if real_weighted else '_integer'}_{bias_eta}_{max_half_weight}.txt")
                with open(log_filepath, "r", encoding="utf-8") as f:
                    lines = f.readlines()
                    for line in lines:
                        line = line.strip(" \r\n")
                        if line == "":  # ignore empty line
                            continue
                        if line[:3] == "#f ":
                            fixed_configurations.append(json.loads(line[3:]))
                        elif line[:2] == "# ":
                            configurations.append(json.loads(line[2:]))
                            data_vec.append([])
                        else:
                            data_vec[-1].append(json.loads(line))
            
            # sanity check
            assert len(max_half_weight_vec) == len(configurations)
            for i in range(len(max_half_weight_vec)):
                fixed_configuration = fixed_configurations[i]
                configuration = configurations[i]
                assert configuration["di"] == d and configuration["dj"] == d and configuration["noisy_measurements"] == T
                assert fixed_configuration["bias_eta"] == float(bias_eta) or (bias_eta == "inf" and fixed_configuration["bias_eta"] == None)
            
            def generate_print(d, T, max_half_weight, data, time_field_name):
                time_field_data = [time_field_name(e) for e in data]
                if not isinstance(time_field_data[0], list):
                    time_field_data = [[e] for e in time_field_data]
                data_vec_len = len(time_field_data[0])
                sample_cnt = len(time_field_data)
                result = f"{d} {T} {max_half_weight} {sample_cnt}"
                for i in range(data_vec_len):
                    # time_vec = np.sort([e[i] for e in time_field_data])
                    time_vec = [e[i] for e in time_field_data]
                    # print(time_vec)
                    # time regardless of error
                    avr_all = np.average(time_vec)
                    mid_all = np.median(time_vec)
                    std_all = np.std(time_vec)
                    max_all = np.amax(time_vec)
                    result += f" {avr_all} {std_all} {mid_all} {max_all}"
                return result
            
            USE_MEDIAN_INSTEAD = False
            USE_MAX_INSTEAD = False
            def fit(content, starting_max_half_weight, ending_max_half_weight):
                X = []
                Ys = []
                Yavrs = []
                groups = None
                lines = content.split("\n")
                for line in lines[1:]:
                    line = line.strip("\r\n ")
                    if line == "":
                        continue
                    spt = line.split(" ")
                    if groups == None:
                        assert (len(spt) - 4) % 4 == 0, "data must be groups of 4"
                        groups = (len(spt) - 4) // 4
                        for _ in range(groups):
                            Ys.append([])
                            Yavrs.append([])
                    max_half_weight = int(spt[2])
                    if max_half_weight < starting_max_half_weight:
                        continue
                    if max_half_weight >= ending_max_half_weight:
                        continue
                    X.append(max_half_weight)
                    for i in range(groups):
                        bias = 4 + 4 * i
                        if USE_MEDIAN_INSTEAD:
                            t = float(spt[bias+2])
                        elif USE_MAX_INSTEAD:
                            t = float(spt[bias+3])
                        else:
                            t = float(spt[bias])
                        tavr = float(spt[bias+1])
                        Ys[i].append(t)
                        Yavrs[i].append(tavr)
                print(X)
                print(Ys)
                results = []
                for i in range(groups):
                    slope, intercept, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Ys[i]])
                    slope_avr, _, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Yavrs[i]])
                    results.append((slope, slope_avr, intercept))
                return results
            
            # process each config
            content = "# " + "<d> <T> <max_half_weight> <sample_cnt> [<avr_all> <std_all> <mid_all> <max_all>]..." + "\n"
            for i in range(len(max_half_weight_vec)):
                max_half_weight = max_half_weight_vec[i]
                data = data_vec[i]
                content += generate_print(d, T, max_half_weight, data, time_field_name) + "\n"
            content += "\n"
            target_slope = 0 if real_weighted else 1
            results = fit(content, starting_max_half_weight, ending_max_half_weight)
            for slope, slope_avr, intercept in results:
                target_value = slope * math.log(starting_max_half_weight) + intercept
                intercept_refined = target_value - target_slope * math.log(starting_max_half_weight)
                content += f"# {slope} {intercept} {slope_avr} {target_slope} {intercept_refined}  # slope, intercept, slope_avr, target_slope, intercept_refined\n"
                # content += f"# slope = {slope}, slope_avr = {slope_avr}, intercept = {intercept}\n"
                # content += f"# fit(x) = exp({slope} * log(x) + ({intercept}))\n"

            print(content)
            with open(os.path.join(os.path.dirname(__file__), f"processed_UF{'' if real_weighted else '_integer'}_{bias_eta}.txt"), "w", encoding="utf8") as f:
                f.write(content)

            continue  # skip real simulation
        
        for max_half_weight in max_half_weight_vec:

            log_filepath = os.path.join(data_folder, f"runtime_statistics_UF{'' if real_weighted else '_integer'}_{bias_eta}_{max_half_weight}.txt")

            ENABLE_MULTITHREADING = False
            num_threads = os.cpu_count() // 2 if ENABLE_MULTITHREADING else 1
            print(num_threads)

            parallel_init = 10

            UF_parameters = f"-p{num_threads} --code_type StandardXZZXCode --noise_model generic-biased-with-biased-cx --bias_eta {bias_eta} --decoder union-find --decoder_config {{\"use_real_weighted\":{'true' if real_weighted else 'false'},\"max_half_weight\":{max_half_weight},\"benchmark_skip_building_correction\":true,\"use_combined_probability\":false}} --parallel_init {parallel_init} --use_brief_edge".split(" ")
            UF_command = qec_playground_benchmark_simulator_runner_vec_command([p], [d], [d], [T], UF_parameters + ["--log_runtime_statistics", log_filepath], max_N=max_N, min_error_cases=max_N)
            print(" ".join(UF_command))

            # UF
            print("UF running...")
            stdout, returncode = run_qec_playground_command_get_stdout(UF_command, no_stdout = True)
            # print("\n" + stdout)
            assert returncode == 0, "command fails..."
