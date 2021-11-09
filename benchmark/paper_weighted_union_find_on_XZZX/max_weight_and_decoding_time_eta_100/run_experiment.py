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

pairs = [ (4, 12, 12), (5, 15, 15), (6, 18, 18) ]  # (di, dj, T)
p = 0.008

max_half_weights = [ i for i in range(1, 11) ]

max_N = 100000000  # this is rarely achieved because p is large enough

min_error_cases = 100  # for debugging
# min_error_cases = 40000  # real experiment

slurm_distribute.SLURM_DISTRIBUTE_FORBIDDEN = True  # forbidden the use of slurm distribute

ENABLE_MULTITHREADING = True
num_threads = os.cpu_count() - 2 if ENABLE_MULTITHREADING else 1
print("num_threads:", num_threads)

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    for pair in pairs:
        di, dj, T = pair

        for max_half_weight in max_half_weights:

            log_filepath = os.path.join(os.path.dirname(__file__), f"runtime_statistics_{di}_{max_half_weight}.txt")

            parameters = f"-p{num_threads} --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta 100 --decoder UF --max_half_weight {max_half_weight}".split(" ")
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [dj], [T], parameters + ["--log_runtime_statistics", log_filepath], max_N=max_N, min_error_cases=min_error_cases)
            if slurm_commands_vec is not None:
                slurm_commands_vec.sanity_checked_append(command)
                continue
            print(" ".join(command))

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
            print(full_result)

        if slurm_commands_vec is not None:
            continue

        # TODO: process data
