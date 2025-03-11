import os, sys
import subprocess, sys
# from src import helper
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
src_dir = os.path.join(qec_playground_root_dir, "src")
sys.path.insert(0, src_dir)



from helper import run_command_get_stdout

## Define parameters
# d_vec = [3, 5, 7, 9, 11, 13] # code distance
n = 0 # number of noisy measurement rounds
p = 0.05 # error rate
max_tree_size_vec = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
# max_tree_size_vec = [0, 100]
d_vec = [11]
# d_vec = [7, 9, 11, 13]
# max_tree_size_vec = [0]

# git_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(
#     __file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
# rust_dir = git_root_dir
# fusion_path = os.path.join(rust_dir, "target", "release", "QEC-Playground")
# command = [fusion_path]


# def qec_playground_benchmark_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, parameters, max_N=100000, min_error_cases=3000, rust_dir=rust_dir, time_budget=None):
#     p_str = "[" + ",".join([f"{e:.8e}" for e in p_vec]) + "]"
#     di_str = "[" + ",".join([str(e) for e in di_vec]) + "]"
#     dj_str = "[" + ",".join([str(e) for e in dj_vec]) + "]"
#     T_str = "[" + ",".join([str(e) for e in T_vec]) + "]"
#     qecp_path = os.path.join(rust_dir, "target", "release", "qecp-cli")
#     command = [qecp_path, "tool", "benchmark", di_str, "--djs", dj_str,
#                T_str, f"-m{max_N}", f"-e{min_error_cases}", p_str] + parameters
#     if time_budget is not None:
#         command += ["--time_budget", f"{time_budget}"]
#     return command

rust_dir = qec_playground_root_dir  # updated project structure
decoder = "fusion"
output_dir = os.path.join(qec_playground_root_dir, "tutorial")

def qec_playground_benchmark_max_tree_size(d_vec, n, p, max_tree_size_vec, decoder, output_dir ,rust_dir=rust_dir):
    qecp_path = os.path.join(rust_dir, "target", "release", "qecp-cli")
    
    for d_i in d_vec: 
        filename = os.path.join(output_dir, f"d_{d_i}_{p}.txt")
        results = []
        for max_tree_size in max_tree_size_vec:
            command = [qecp_path, "tool", "benchmark", f"[{d_i}]", f"[{n}]", f"[{p}]"]
            command += ["--decoder", decoder]
            command += ["--decoder-config", f'{{"max_tree_size":{max_tree_size}}}']
            print("command: {command}")
            command += ["-p10"]
            print(" ".join(command))
            stdout, returncode = run_command_get_stdout(command, no_stdout=False)
            print("\n")
            print(stdout)
            assert returncode == 0, "command fails..."

            full_result = stdout.strip(" \r\n").split("\n")[-1]
            lst = full_result.split(" ")
            if len(lst) < 7:
                print_result = f"# data missing"
            else:
                error_rate = float(lst[5])
                confidence_interval = float(lst[7])
                print_result = f"{full_result}"
            
            results.append(print_result)
            # print(print_result)

        # print("\n\n")
        # print("\n".join(results))
        # print("\n\n")

        with open(filename, "w", encoding="utf-8") as f:
            f.write("\n".join(results) + "\n")

qec_playground_benchmark_max_tree_size(d_vec, n, p, max_tree_size_vec, decoder, output_dir)



# print("error rate: {error_rate}")

# if __name__ == "__main__":
#     # d = 11 # code distance
#     # n = 0 # number of noisy measurement rounds
#     # p = 0.05 # error rate
#     # max_tree_size = 0 
#     # decoder = "fusion"
#     qec_playground_benchmark_max_tree_size(d_vec, n, p, max_tree_size_vec, output_dir, decoder)

