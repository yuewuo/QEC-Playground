import os, sys, git
from dataclasses import dataclass

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

p_per_10 = 5
p_vec = [1e-4 * (10 ** (i / p_per_10)) for i in range(-2, p_per_10 * 2 + 1)]
d_vec = [3, 5, 7, 9, 11, 13, 15]
min_error_cases = 40000
max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "12:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = "8G"
# for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12
parameters = f"-p{STO(0)} --time-budget {3600*10} --code-type rotated-planar-code --noise-model stim-noise-model".split(
    " "
)

compile_code_if_necessary()


@dataclass
class Configuration:
    name: str
    decoder_parameter: str


configurations = [
    Configuration(
        name="mwpm",
        decoder_parameter='--decoder fusion --decoder-config {"max_half_weight":7}',
    ),
    Configuration(
        name="weighted_uf",
        decoder_parameter='--decoder fusion --decoder-config {"max_half_weight":7,"max_tree_size":0}',
    ),
    Configuration(
        name="unweighted_uf",
        decoder_parameter='--decoder fusion --decoder-config {"max_half_weight":1,"max_tree_size":0}',
    ),
]


@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(
    slurm_commands_vec=None,
    run_command_get_stdout=run_qec_playground_command_get_stdout,
):

    for config in configurations:
        for d in d_vec:
            filename = os.path.join(os.path.dirname(__file__), f"{config.name}_{d}.txt")

            results = []
            for p in p_vec:
                command = qec_playground_benchmark_simulator_runner_vec_command(
                    [p],
                    [d],
                    [d],
                    [d - 1],
                    parameters + config.decoder_parameter.split(" "),
                    max_N=max_N,
                    min_error_cases=min_error_cases,
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
                error_rate = float(lst[5])
                confidence_interval = float(lst[7])

                # record result
                print_result = f"{full_result}"
                results.append(print_result)
                print(print_result)

            if slurm_commands_vec is not None:
                continue

            print("\n\n")
            print("\n".join(results))
            print("\n\n")

            with open(filename, "w", encoding="utf-8") as f:
                f.write("\n".join(results) + "\n")
