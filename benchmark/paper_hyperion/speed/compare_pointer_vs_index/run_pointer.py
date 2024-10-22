import os, sys, git, math

qec_playground_root_dir = git.Repo(".", search_parent_directories=True).working_tree_dir
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_tolerant_MWPM_dir = os.path.join(
    qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM"
)
sys.path.insert(0, fault_tolerant_MWPM_dir)

from automated_threshold_evaluation import (
    qec_playground_benchmark_simulator_runner_vec_command,
)
from automated_threshold_evaluation import (
    run_qec_playground_command_get_stdout,
    compile_code_if_necessary,
)

sys.path.insert(
    0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities")
)
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO
from common import *

slurm_distribute.SLURM_DISTRIBUTE_TIME = "12:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = "8G"
# for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12
parameters = f"-p{STO(0)} --time-budget {3600*10} --code-type rotated-planar-code --noise-model stim-noise-model".split(
    " "
)

compile_code_if_necessary("--features=hyperion")


@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(
    slurm_commands_vec=None,
    run_command_get_stdout=run_qec_playground_command_get_stdout,
):

    filename = os.path.join(os.path.dirname(__file__), f"result_pointer.txt")
    total_sample = 0
    total_error = 0
    results = []

    for job_id in range(split_job):
        benchmark_profile_path = os.path.join(
            profile_folder, f"pointer_{job_id}.profile"
        )
        command = qec_playground_benchmark_simulator_runner_vec_command(
            [p],
            [d],
            [d],
            [d],
            parameters
            + decoder_parameter.split(" ")
            + ["--label", f"{job_id}"]
            + ["--log-runtime-statistics", benchmark_profile_path],
            max_N=max_N // split_job,
            min_error_cases=min_error_cases // split_job,
        )
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
        total_rounds = int(lst[3])
        error_count = int(lst[4])

        # record result
        print_result = f"{full_result}"
        results.append(print_result)
        print(print_result)
        total_sample += total_rounds
        total_error += error_count

    with open(filename + ".original", "w", encoding="utf-8") as f:
        f.write("\n".join(results) + "\n")

    error_rate = total_error / total_sample
    confidence = (
        math.sqrt(1.96 * (error_rate * (1.0 - error_rate) / total_sample)) / error_rate
    )

    format = "# <total samples> <total errors> <logical error rate> <confidence>"
    formatted = f"{total_sample} {total_error} {error_rate:.6e} {confidence:.3e}"
    print(format)
    print(formatted)
    with open(filename, "w", encoding="utf-8") as f:
        f.write(format + "\n")
        f.write(formatted + "\n")
