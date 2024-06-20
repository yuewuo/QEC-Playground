import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
src_dir = os.path.join(qec_playground_root_dir, "src")
sys.path.insert(0, src_dir)
from helper import run_command_get_stdout

## Define parameters
n = 0 # number of noisy measurement rounds
p = 0.05 # error rate
max_tree_size_vec = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
d_vec = [11, 15, 17]

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

