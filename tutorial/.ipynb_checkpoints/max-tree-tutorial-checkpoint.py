import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
# rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
rust_dir = qec_playground_root_dir
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO

di_vec = [3,5,7,9,11,13,15]
p_vec = [0.05]
max_tree_size_vec = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15]
min_error_cases = 100000
max_half_weight_vec = [1, 2, 3, 4, 5, 6, 7] + [8, 10, 12, 14] + [16, 20, 24, 28] + [2 ** i for i in range(5, 11)]
n = 0 

max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "12:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '8G'  # it took 8G memory at 8x24x24 on my laptop, set higher RAM in HPC
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
parameters = f"-p{STO(0)} --time-budget {3600*3*4} --decoder fusion".split(" ")


compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    qecp_path = os.path.join(rust_dir, "target", "release", "qecp-cli")
    for d_i in di_vec:
        for p in p_vec:
            for max_half_weight in max_half_weight_vec: 
                filename = os.path.join(os.path.dirname(__file__), f"max-tree-max-half-weight_{d_i}_{p}_{max_half_weight}.txt")
                results = []
                for max_tree_size in max_tree_size_vec:
                    command = [qecp_path, "tool", "benchmark", f"[{d_i}]", f"[{n}]", f"-m{max_N}", f"-e{min_error_cases}", f"[{p}]"]
                    command += parameters
                    command += ["--decoder-config", f'{{"max_tree_size":{max_tree_size},"max_half_weight":{max_half_weight}}}']
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
                    # lst = full_result.split(" ")
                    # total_rounds = int(lst[3])
                    # error_count = int(lst[4])
                    # error_rate = float(lst[5])
                    # confidence_interval = float(lst[7])

                    # record result
                    print_result = f"{full_result}"
                    results.append(print_result)
                    print(print_result)
                
                if slurm_commands_vec is not None:
                    continue
                
                print("\n\n")
                print("\n".join(results))
                print("\n\n")

                with open(filename, "w", encoding="utf-8") as f:
                    f.write("\n".join(results) + "\n")




# # YL's script
# # import os, sys
# # import subprocess, sys
# # qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
# # src_dir = os.path.join(qec_playground_root_dir, "src")
# # sys.path.insert(0, src_dir)
# # from helper import run_command_get_stdout
# # import slurm_distribute
# # from slurm_distribute import slurm_threads_or as STO

# # ## Define parameters
# # n = 0 # number of noisy measurement rounds
# # p = 0.05 # error rate
# # max_tree_size_vec = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
# # d_vec = [11, 13]

# # rust_dir = qec_playground_root_dir  # updated project structure
# # decoder = "fusion"
# # output_dir = os.path.join(qec_playground_root_dir, "tutorial")

# # def qec_playground_benchmark_max_tree_size(d_vec, n, p, max_tree_size_vec, decoder, output_dir ,rust_dir=rust_dir):
# #     qecp_path = os.path.join(rust_dir, "target", "release", "qecp-cli")
    
# #     for d_i in d_vec: 
# #         filename = os.path.join(output_dir, f"d_{d_i}_{p}.txt")
# #         results = []
# #         for max_tree_size in max_tree_size_vec:
# #             command = [qecp_path, "tool", "benchmark", f"[{d_i}]", f"[{n}]", f"[{p}]"]
# #             command += ["--decoder", decoder]
# #             command += ["--decoder-config", f'{{"max_tree_size":{max_tree_size}}}']
# #             print("command: {command}")
# #             command += ["-p10"]
# #             print(" ".join(command))
# #             stdout, returncode = run_command_get_stdout(command, no_stdout=False)
# #             print("\n")
# #             print(stdout)
# #             assert returncode == 0, "command fails..."

# #             full_result = stdout.strip(" \r\n").split("\n")[-1]
# #             lst = full_result.split(" ")
# #             if len(lst) < 7:
# #                 print_result = f"# data missing"
# #             else:
# #                 error_rate = float(lst[5])
# #                 confidence_interval = float(lst[7])
# #                 print_result = f"{full_result}"
            
# #             results.append(print_result)
# #             # print(print_result)

# #         # print("\n\n")
# #         # print("\n".join(results))
# #         # print("\n\n")

# #         with open(filename, "w", encoding="utf-8") as f:
# #             f.write("\n".join(results) + "\n")

# # qec_playground_benchmark_max_tree_size(d_vec, n, p, max_tree_size_vec, decoder, output_dir)

