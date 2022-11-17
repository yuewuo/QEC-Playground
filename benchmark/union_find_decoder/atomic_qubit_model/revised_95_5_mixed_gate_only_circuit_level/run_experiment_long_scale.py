import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO
from slurm_distribute import cpu_hours as CH
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # use fewer CPUs for more available resources
slurm_distribute.SLURM_DISTRIBUTE_TIME = "2:00:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '24G'

di_vec = [3, 5, 7, 9, 11, 13]
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
print(p_vec)
min_error_cases = 1000

# debug configurations
# di_vec = [3, 5]
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(3)]
# min_error_cases = 100

max_N = 100000000

# original was 20min for 60 cores, if using 12 CPUs for each task then it should be 100min which is 6000sec
# this is 20 CPU hours
UF_parameters = f"-p{STO(0)} --decoder UF --max_half_weight 10 --time_budget {CH(20)} --use_xzzx_code --noise_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ")  # a maximum 20min for each point

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    results = []
    for di in di_vec:
        local_results = []
        filename = os.path.join(os.path.dirname(__file__), f"d_{di}_{di}.txt")
        for p in p_vec:
            p_pauli = p * 0.05
            p_erasure = p * 0.95
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
