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

di_vec = [11, 13]
p = 0.07
divide = 10
bias_eta_vec = [str(0.5 * (10 ** (i / divide))) for i in range(4 * divide + 1)] + ["+inf"]
min_error_cases = 1000000
# min_error_cases = 10  # debug

max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "12:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '4G'
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
# this will consume 167*12*12=24048 CPU hours, taking 2004 CPUs working simultaneously
UF_parameters = f"-p{STO(0)} --decoder UF --max_half_weight 100 --time_budget {3600*3*4} --use_xzzx_code --shallow_error_on_bottom".split(" ")  # a maximum 20min for each point
MWPM_parameters = f"-p{STO(0)} --time_budget {3600*3*4} --use_xzzx_code --shallow_error_on_bottom".split(" ")

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):

    collected_results = {}

    for (filename_prefix, paramters) in [("UF", UF_parameters), ("MWPM", MWPM_parameters)]:
        collected_results[filename_prefix] = {}
        for di in di_vec:
            collected_results[filename_prefix][di] = {}
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
                collected_results[filename_prefix][di][bias_eta] = (error_rate, confidence_interval)

                # record result
                print_result = f"{bias_eta if bias_eta != '+inf' else '50000'} {p} {di} {total_rounds} {error_count} {error_rate} {confidence_interval}"
                if bias_eta == "+inf":
                    results.append("# the following is actually for bias_eta = +inf, just for ease of plotting")
                results.append(print_result)
                print(print_result)
            
            if slurm_commands_vec is not None:
                continue
            
            print("\n\n")
            print("\n".join(results))
            print("\n\n")

            with open(filename, "w", encoding="utf-8") as f:
                f.write("\n".join(results) + "\n")

    if slurm_commands_vec is not None:
        return
    
    for di in di_vec:
        filename = os.path.join(os.path.dirname(__file__), f"relative_d{di}_p{p}.txt")
        with open(filename, "w", encoding="utf8") as f:
            for bias_eta in bias_eta_vec:
                error_rate_UF, confidence_interval_UF = collected_results["UF"][di][bias_eta]
                error_rate_MWPM, confidence_interval_MWPM = collected_results["MWPM"][di][bias_eta]
                relative = (error_rate_UF - error_rate_MWPM) / (error_rate_UF + error_rate_MWPM)
                relative_confidence_interval = (
                    (2 * error_rate_MWPM / ((error_rate_UF + error_rate_MWPM) ** 2) * confidence_interval_UF * error_rate_UF) ** 2 + 
                    (2 * error_rate_UF / ((error_rate_UF + error_rate_MWPM) ** 2) * confidence_interval_MWPM * error_rate_MWPM) ** 2
                ) ** 0.5
                if bias_eta == "+inf":
                    f.write("# the following is actually for bias_eta = +inf, just for ease of plotting\n")
                f.write(f"{bias_eta if bias_eta != '+inf' else '50000'} {relative} {relative_confidence_interval}\n")
