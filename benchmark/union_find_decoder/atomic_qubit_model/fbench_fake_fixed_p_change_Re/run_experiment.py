import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(__file__), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 8  # it doesn't rely on too much CPUs
slurm_distribute.SLURM_DISTRIBUTE_TIME = "5:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '6G'
import math, random, scipy.stats
import numpy as np

di_vec = [3, 5, 7, 9, 11, 13]
p = 1e-5  # low p case, so that when Re=95%, it's still in the de=(d+1)/2 regime
split = 20
Re_vec = [i / split for i in range(split)] + [0.96, 0.97, 0.98, 0.99, 0.994, 0.997, 1]
print(Re_vec)
min_error_cases = 0  # +inf
max_N = 0  # +inf

time_budget = 5 * 3600  # 5 hour
# time_budget = 10  # debug
UF_parameters = f"-p{STO(0)} --decoder UF --max_half_weight 10 --time_budget {time_budget} --use_xzzx_code --error_model OnlyGateErrorCircuitLevel --use_fast_benchmark --fbench_use_fake_decoder --fbench_disable_additional_error --fbench_target_dev 1e-2".split(" ")

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    results = []
    for di in di_vec:
        local_results = []
        filename = os.path.join(os.path.dirname(__file__), f"d_{di}_{di}.txt")
        for Re in Re_vec:
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

            # record result
            print_result = f"{Re} " + full_result
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
