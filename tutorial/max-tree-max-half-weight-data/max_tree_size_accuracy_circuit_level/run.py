import os, sys, git


qec_playground_root_dir = git.Repo(".", search_parent_directories=True).working_tree_dir
rust_dir = qec_playground_root_dir
fault_tolerant_MWPM_dir = os.path.join(
    qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM"
)
sys.path.insert(0, fault_tolerant_MWPM_dir)
from automated_threshold_evaluation import (
    run_qec_playground_command_get_stdout,
    compile_code_if_necessary,
)

sys.path.insert(
    0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities")
)
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO

di_vec = [3, 5, 7, 9, 11, 13, 15]
p_vec = [0.0025]
max_tree_size_vec = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
min_error_cases = 100000
max_half_weight_vec = (
    [1, 2, 3, 4, 5, 6, 7]
    + [8, 10, 12, 14]
    + [16, 20, 24, 28]
    + [2**i for i in range(5, 11)]
)

max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "13:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = "16G"
# for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12
parameters = f"-p{STO(0)} --time-budget {3600*3*4} --decoder fusion".split(" ")
parameters += ["--code-type", "rotated-planar-code"]
parameters += ["--noise-model", "stim-noise-model"]


compile_code_if_necessary()


@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(
    slurm_commands_vec=None,
    run_command_get_stdout=run_qec_playground_command_get_stdout,
):
    qecp_path = os.path.join(rust_dir, "target", "release", "qecp-cli")
    for d_i in di_vec:
        for p in p_vec:
            filename = os.path.join(
                os.path.dirname(__file__),
                f"{d_i}_{p}.txt",
            )
            results = ["# <max_half_weight> <max_tree_size> ..."]
            for max_half_weight in max_half_weight_vec:
                for max_tree_size in max_tree_size_vec:
                    command = [
                        qecp_path,
                        "tool",
                        "benchmark",
                        f"[{d_i}]",
                        f"[{d_i}]",
                        f"-m{max_N}",
                        f"-e{min_error_cases}",
                        f"[{p}]",
                    ]
                    command += parameters
                    command += [
                        "--decoder-config",
                        f'{{"max_tree_size":{max_tree_size},"max_half_weight":{max_half_weight}}}',
                    ]
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
                    # lst = full_result.split(" ")
                    # total_rounds = int(lst[3])
                    # error_count = int(lst[4])
                    # error_rate = float(lst[5])
                    # confidence_interval = float(lst[7])

                    # record result
                    print_result = f"{max_half_weight} {max_tree_size} {full_result}"
                    results.append(print_result)
                    print(print_result)

            if slurm_commands_vec is not None:
                continue

            print("\n\n")
            print("\n".join(results))
            print("\n\n")

            with open(filename, "w", encoding="utf-8") as f:
                f.write("\n".join(results) + "\n")
