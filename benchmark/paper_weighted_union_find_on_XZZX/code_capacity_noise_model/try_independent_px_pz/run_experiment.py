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

# di_vec = [11, 13]
di_vec = [11]
p = 0.1
# bias_eta_vec = [0.5, 1, 3, 10, 30, 100, 300, 1000, 3000, 10000, "+inf"]
# bias_eta_vec = [0.5] + [1 * (10 ** (i / 4)) for i in range(4 * 4 + 1)] + ["+inf"]
divide = 20
bias_eta_vec = [0.5 * (10 ** (i / divide)) for i in range(4 * divide + 1)] + ["+inf"]
min_error_cases = 100000
# min_error_cases = 10  # debug

max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "02:00:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '4G'
UF_parameters = f"-p{STO(0)} --decoder UF --max_half_weight 100 --time_budget 3600 --use_xzzx_code --shallow_error_on_bottom --independent_px_pz".split(" ")  # a maximum 20min for each point
MWPM_parameters = f"-p{STO(0)} --time_budget 3600 --use_xzzx_code --shallow_error_on_bottom --independent_px_pz".split(" ")

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    # for (filename_prefix, paramters) in [("UF", UF_parameters), ("MWPM", MWPM_parameters)]:
    for (filename_prefix, paramters) in [("MWPM", MWPM_parameters)]:
        for di in di_vec:
            filename = os.path.join(os.path.dirname(__file__), f"{filename_prefix}_d{di}_p{p}.txt")
            
            results = []
            for bias_eta in bias_eta_vec:
                command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [di], [0], paramters + ["--bias_eta", f"{bias_eta}"], max_N=max_N, min_error_cases=min_error_cases)
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
                print_result = f"{bias_eta} {p} {di} {total_rounds} {error_count} {error_rate} {confidence_interval}"
                results.append(print_result)
                print(print_result)
            
            if slurm_commands_vec is not None:
                continue
            
            print("\n\n")
            print("\n".join(results))
            print("\n\n")

            with open(filename, "w", encoding="utf-8") as f:
                f.write("\n".join(results) + "\n")
