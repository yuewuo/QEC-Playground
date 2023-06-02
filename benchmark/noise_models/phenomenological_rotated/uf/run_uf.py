import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO
from slurm_distribute import cpu_hours as CH
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "threshold_analyzer"))
from threshold_analyzer import qecp_benchmark_simulate_func_command_vec
from threshold_analyzer import run_qecp_command_get_stdout, compile_code_if_necessary
from threshold_analyzer import ThresholdAnalyzer

slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # for more machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_TIME = "1:10:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '24G'

di_vec = [3, 5, 7, 9, 11, 13]
p_vec = [0.05, 0.04, 0.03, 0.025, 0.02, 0.015, 0.01, 0.0075, 0.005, 0.0025, 0.001]
print(p_vec)
min_error_cases = 6000

max_repeats = 100000000

# origin: 60 cores, 1200 sec = 20CPU hours
UF_parameters = f"""-p{STO(0)} --code-type rotated-planar-code --ignore-logical-i --decoder union-find --time-budget {CH(10)} --noise-model stim-noise-model --noise-model-configuration {{"after_clifford_depolarization":0,"after_reset_flip_probability":0}}""".split(" ")  # a maximum 20min for each point

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qecp_command_get_stdout):
    results = []
    for di in di_vec:
        local_results = []
        filename = os.path.join(os.path.dirname(__file__), f"d_{di}_{di}.txt")
        print(p_vec)
        for p in p_vec:
            command = qecp_benchmark_simulate_func_command_vec(p, di, di, di + 1, UF_parameters, max_repeats=max_repeats, min_error_cases=min_error_cases)
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
            error_count = int(lst[4])
            error_rate = float(lst[5])
            confidence_interval = float(lst[7])

            # record result
            print_result = f"{p} " + full_result
            local_results.append(print_result)
            results.append(print_result)
            print(print_result)

            if error_count < 100:
                break  # next is not trust-worthy, ignore every p behind it

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
