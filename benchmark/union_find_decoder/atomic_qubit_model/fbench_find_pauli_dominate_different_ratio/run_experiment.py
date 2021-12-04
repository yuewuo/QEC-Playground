import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO
from slurm_distribute import cpu_hours as CH
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # it doesn't rely on too much CPUs
slurm_distribute.SLURM_DISTRIBUTE_TIME = "02:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '2G'
import math, random, scipy.stats
import numpy as np

di = 5
dominate_threshold = 3.25  # the effective code for pure pauli is 3, pure erasure is 5. find a point where it's close to 3 to say it's pauli dominated

split = 20
Re_vec = [i / split for i in range(split)] + [0.96, 0.97, 0.98, 0.99, 0.994, 0.997, 1]

p_split = 10
p_vec = [0.5 * (10 ** (- i / p_split)) for i in range(p_split * 4 + 1)]
# print(p_vec)
min_error_cases = 0  # +inf
max_N = 0  # +inf

# 2 hour for each task on 12 CPUs, which is 24 CPU hours
UF_parameters = f"-p{STO(0)} --time_budget {CH(24)} --use_xzzx_code --error_model OnlyGateErrorCircuitLevel --use_fast_benchmark --fbench_use_fake_decoder --fbench_disable_additional_error --fbench_target_dev 1e-3".split(" ")

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    results = []
    for Re in Re_vec:
        local_results = []
        effective_distances = []
        filename = os.path.join(os.path.dirname(__file__), f"Re_{Re:.4g}.txt")
        for p in p_vec:
            p_pauli = p * (1 - Re)
            p_erasure = p * Re
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [di], UF_parameters + ["--pes", f"[{p_erasure:.8e}]"], max_N=max_N, min_error_cases=min_error_cases)
            if slurm_commands_vec is not None:
                slurm_commands_vec.sanity_checked_append(command)
                continue
            print(" ".join(command))
            

            # run experiment
            stdout, returncode = run_command_get_stdout(command)
            print("\n" + stdout)
            assert returncode == 0, "command fails..."

            # full result
            full_result = stdout.strip(" \r\n").split("\n")[-1]
            lst = full_result.split(" ")
            if lst[0] == "format:":
                print("[warning] missing data")
                continue
            error_rate = float(lst[7])
            confidence_interval = float(lst[8])
            
            # compute effective code distance
            if 'last_data' in locals() and last_data is not None:
                p_last, error_rate_last, confidence_interval_last = last_data
                X = [math.log(p_last), math.log(p)]
                baseline_slope, _, _, _, _ = scipy.stats.linregress(X, [math.log(error_rate_last), math.log(error_rate)])
                slope_vec = []
                for random_round in range(20):
                    Y = [math.log(error_rate_last) + random.gauss(0, confidence_interval_last / 1.96), math.log(error_rate) + random.gauss(0, confidence_interval / 1.96)]
                    slope, intercept, _, _, _ = scipy.stats.linregress(X, Y)
                    slope_vec.append(slope)
                slope_confidence_interval = 1.96 * np.std(slope_vec)
                full_result += f" {baseline_slope} {slope_confidence_interval} {math.sqrt(p * p_last)}"
                effective_distances.append([math.sqrt(p * p_last), baseline_slope, slope_confidence_interval])
            if p == p_vec[-1]:
                last_data = None
            else:
                last_data = (p, error_rate, confidence_interval)

            # record result
            print_result = f"{p} " + full_result
            local_results.append(print_result)
            results.append(print_result)
            print(print_result)

        if slurm_commands_vec is not None:
            continue

        print("\n\n")
        print("\n".join(local_results))
        print("\n\n")

        with open(filename, "w", encoding="utf-8") as f:
            f.write("\n".join(local_results) + "\n")

        results.append("")

    if slurm_commands_vec is not None:
        return

    print("\n\n")
    print("\n".join(results))

    # analyze slopes
    # 2021.12.3 we found that fbench is somehow inconsistent with MC simulation, so I'll just put it here to wait to a fix to fbench
