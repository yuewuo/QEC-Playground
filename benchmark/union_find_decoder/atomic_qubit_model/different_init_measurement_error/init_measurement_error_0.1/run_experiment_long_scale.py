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

di_vec = [3, 5, 7, 9, 11, 13]
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
print(p_vec)
min_error_cases = 1000

# debug configurations
# di_vec = [3, 5]
# p_vec = [0.5 * (10 ** (- i / 3)) for i in range(3)]
# min_error_cases = 100

max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "03:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '4G'
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # use `SLURM_USE_SCAVENGE_PARTITION=1` for more available resources

init_measurement_error_rate = 0.1
error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate},"use_correlated_pauli":true}}'
UF_parameters = f"-p{STO(0)} --decoder UF --max_half_weight 10 --time_budget {3600*3} --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ") + ["--error_model_configuration", error_model_configuration]  # a maximum 20min for each point

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    results = []
    for di in di_vec:
        local_results = []
        filename = os.path.join(os.path.dirname(__file__), f"d_{di}.txt")
        for p in p_vec:
            p_pauli = p * 0.02
            p_erasure = p * 0.98
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [di], UF_parameters + ["--pes", f"[{p_erasure}]"], max_N=max_N, min_error_cases=min_error_cases)
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
