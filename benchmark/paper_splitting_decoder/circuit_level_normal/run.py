import os
import sys
import subprocess
import sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(
    __file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(
    qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)

if True:
    from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
    from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
    sys.path.insert(0, os.path.join(qec_playground_root_dir,
                    "benchmark", "slurm_utilities"))
    import slurm_distribute
    from slurm_distribute import slurm_threads_or as STO
    from slurm_distribute import cpu_hours as CH

# rotated surface code only supports odd number code distances
di_vec = [3, 5, 7]
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
p_vec[0] = 0.4
min_error_cases = 40000
max_N = 100000000

print(p_vec)

slurm_distribute.SLURM_DISTRIBUTE_TIME = "12:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '8G'
# for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12
parameters = f"""-p{STO(0)} --time-budget {CH(10)} --code-type rotated-planar-code --decoder fusion --noise-model stim-noise-model --ignore-logical-i""".split(" ")

compile_code_if_necessary()


@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec=None, run_command_get_stdout=run_qec_playground_command_get_stdout):

    for di in di_vec:
        filename = os.path.join(os.path.dirname(__file__), f"d_{di}.txt")

        results = []
        for p in p_vec:
            command = qec_playground_benchmark_simulator_runner_vec_command(
                [p], [di], [di], [di], parameters, max_N=max_N, min_error_cases=min_error_cases)
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
