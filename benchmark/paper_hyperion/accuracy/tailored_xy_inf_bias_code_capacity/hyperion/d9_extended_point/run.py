import os
import sys
import subprocess
import sys
import json
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(
    __file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_tolerant_MWPM_dir = os.path.join(
    qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_tolerant_MWPM_dir)

if True:
    from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
    from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
    sys.path.insert(0, os.path.join(qec_playground_root_dir,
                    "benchmark", "slurm_utilities"))
    import slurm_distribute
    from slurm_distribute import slurm_threads_or as STO
    from slurm_distribute import cpu_hours as CH

slurm_distribute.SLURM_DISTRIBUTE_TIME = "5:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '96G'
# for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12

# tool benchmark '[9]' --djs '[9]' '[0]' -m100000000 -e40000 '[3.15478672e-01]'
min_error_cases = 40000
max_N = 100000000
repeat = 100
d = 9
p = 3.15478672e-01
parameters = (
    f"-p{STO(0)}" + """ --bias-eta 1e200 --code-type rotated-tailored-code --time-budget 15000.0 --decoder hyperion""").split(" ")


compile_code_if_necessary()


@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec=None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    all_rounds = 0
    all_error_count = 0

    for i in range(repeat):
        local_parameters = parameters + ["--label", f"{i}"]
        command = qec_playground_benchmark_simulator_runner_vec_command(
            [p], [d], [d], [0], local_parameters, max_N=max_N, min_error_cases=min_error_cases)
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
        if len(lst) < 7:
            print(f"{i} data is missing")
        else:
            total_rounds = int(lst[3])
            error_count = int(lst[4])
            error_rate = float(lst[5])
            confidence_interval = float(lst[7])
            print_result = f"{full_result}"
            all_rounds += total_rounds
            all_error_count += error_count

    if slurm_commands_vec is None:
        print(f"all_rounds: {all_rounds}")
        print(f"all_error_count: {all_error_count}")
        error_rate = all_error_count / all_rounds
        print(f"{p} {d} {0} {all_rounds} {all_error_count} {error_rate}")
