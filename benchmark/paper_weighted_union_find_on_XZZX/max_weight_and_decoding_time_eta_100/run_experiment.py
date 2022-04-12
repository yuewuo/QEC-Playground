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
from slurm_distribute import confirm_or_die
import json

# import process data library
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "code_distance_decoding_time_eta_100"))
from process_data import generate_print, print_title

pairs = [ (4, 12, 12), (5, 15, 15), (6, 18, 18) ]  # (di, dj, T)
p = 0.008

max_half_weights = [i for i in range(1, 11)] + [i for i in range(12, 51, 2)]

max_N = 100000000  # this is rarely achieved because p is large enough

# min_error_cases = 100  # for debugging
min_error_cases = 40000  # real experiment

slurm_distribute.SLURM_DISTRIBUTE_FORBIDDEN = True  # forbidden the use of slurm distribute
slurm_distribute.SLURM_DISTRIBUTE_DO_NOT_CHECK_JOBOUT = True

ENABLE_MULTITHREADING = True
num_threads = os.cpu_count() - 2 if ENABLE_MULTITHREADING else 1
print("num_threads:", num_threads)

if (not slurm_distribute.ONLY_PRINT_COMMANDS) and (not slurm_distribute.SLURM_USE_EXISTING_DATA):
    confirm_or_die("sure to start simulation? this will truncate the existing runtime-statistics files")

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    for pair in pairs:
        di, dj, T = pair
        results = []

        for max_half_weight in max_half_weights:

            log_filepath = os.path.join(os.path.dirname(__file__), f"runtime_statistics_{di}_{max_half_weight}.txt")

            parameters = f"-p{num_threads} --use_xzzx_code --error_model GenericBiasedWithBiasedCX --bias_eta 100 --decoder UF --max_half_weight {max_half_weight}".split(" ")
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [dj], [T], parameters + ["--log_runtime_statistics", log_filepath], max_N=max_N, min_error_cases=min_error_cases)
            if slurm_commands_vec is not None:
                slurm_commands_vec.sanity_checked_append(command)
                continue
            print(" ".join(command))

            stdout, returncode = run_command_get_stdout(command)
            print("\n" + stdout)  # SLURM_DISTRIBUTE_DO_NOT_CHECK_JOBOUT
            assert returncode == 0, "command fails..."

            # process data
            data = []
            with open(log_filepath, "r", encoding="utf-8") as f:
                lines = f.readlines()
                for line in lines:
                    line = line.strip(" \r\n")
                    if line == "":  # ignore empty line
                        continue
                    if line[:3] == "#f ":
                        pass
                    elif line[:2] == "# ":
                        pass
                    else:
                        data.append(json.loads(line))
            time_field_name = "time_run_to_stable"
            result = f"{max_half_weight} " + generate_print(di, dj, T, data, time_field_name)
            print(result)
            results.append(result)

        if slurm_commands_vec is not None:
            continue

        print("\n\n")
        print(f"<max_half_weight> " + print_title)
        print("\n".join(results))
        print("\n\n")

        filename = os.path.join(os.path.dirname(__file__), f"data_{di}.txt")
        with open(filename, "w", encoding="utf-8") as f:
            f.write(f"<max_half_weight> " + print_title + "\n")
            f.write("\n".join(results) + "\n")
