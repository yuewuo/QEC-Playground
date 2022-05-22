import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO

d = 7
p = 0.002
bias_eta_vec = ["10", "15", "30", "100", "1000", "inf"]
# bias_eta_vec = ["inf"]
max_half_weight_vec = [1, 2, 3, 4, 5, 6, 7] + [2 ** i for i in range(3, 11)]
# max_half_weight_vec = [1000]
max_N = 1000000000
min_error_cases = 40000
# min_error_cases = 4000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "06:30:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '4G'
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # for more usable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
time_budget = 6 * 3600  # 6 hour
parameters = f"-p{STO(0)} --time_budget {time_budget} --code_type StandardXZZXCode --error_model generic-biased-with-biased-cx --decoder union-find".split(" ")

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):

    for real_weighted in [True, False]:
    # for real_weighted in [True]:

        for bias_eta in bias_eta_vec:

            filename = os.path.join(os.path.dirname(__file__), f"bias_eta_{bias_eta}_{'real' if real_weighted else 'integer'}.txt")
            results = []

            for max_half_weight in max_half_weight_vec:

                local_parameters = parameters + ["--decoder_config", f"{{\"use_real_weighted\":{'true' if real_weighted else 'false'},\"max_half_weight\":{max_half_weight},\"use_combined_probability\":false}}"]
                local_parameters += ["--bias_eta", f"{bias_eta}"]
                command = qec_playground_benchmark_simulator_runner_vec_command([p], [d], [d], [d], local_parameters, max_N=max_N, min_error_cases=min_error_cases)
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
