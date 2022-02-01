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

d = 5
p = 0.005
max_N = 100000000
min_error_cases = 400000
bias_eta_vec = ["10", "100", "1000", "inf"]

slurm_distribute.SLURM_DISTRIBUTE_TIME = "06:30:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '2G'
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
time_budget = 6 * 3600  # 6 hour
parameters = f"-p{STO(0)} --time_budget {time_budget} --use_xzzx_code --error_model GenericBiasedWithBiasedCX".split(" ")

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    for bias_eta in bias_eta_vec:

        filename = os.path.join(os.path.dirname(__file__), f"bias_eta_{bias_eta}.txt")
        results = []
        
        for max_half_weight in ["MWPM"] + [i for i in range(1, 11)] + [i for i in range(12, 51, 2)]: # MWPM is baseline

            local_parameters = parameters + ["--bias_eta", f"{bias_eta}"]
            if max_half_weight != "MWPM":
                local_parameters += ["--max_half_weight", f"{max_half_weight}", "--decoder", "UF"]

            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [d], [d], [d], local_parameters, max_N=max_N, min_error_cases=min_error_cases)
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
            print_result = f"{max_half_weight} {p} {d} {total_rounds} {error_count} {error_rate} {confidence_interval}"
            results.append(print_result)
            print(print_result)

        if slurm_commands_vec is not None:
            continue

        print("\n\n")
        print("\n".join(results))
        print("\n\n")

        with open(filename, "w", encoding="utf-8") as f:
            f.write("\n".join(results) + "\n")
