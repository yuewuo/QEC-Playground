import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)

GENERATE_HPC_SBATCH_SCRIPTS = False
if len(sys.argv) > 1 and sys.argv[1] == "hpc":
    qec_playground_root_dir = "/home/yw729/project/QEC-Playground"
    GENERATE_HPC_SBATCH_SCRIPTS = True

rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [4,5,6,7,8]
p_vec = [0.0050,0.0055,0.0060,0.0065,0.0070,0.0075,0.0080,0.0085,0.0090,0.0095,0.0100,0.0105,0.0110]
min_error_cases = 100000
# min_error_cases = 10  # debug

max_N = 400000

parameters = f"-p0 --time_budget 1800 --use_xzzx_code --bias_eta 100".split(" ")  # a maximum 20min for each point


for (name, error_model) in [("biased", "GenericBiasedWithBiasedCX"), ("standard", "GenericBiasedWithStandardCX")]:
    for di in di_vec:
        filename = os.path.join(os.path.dirname(__file__), f"{name}_{di}.txt")
        
        results = []
        if GENERATE_HPC_SBATCH_SCRIPTS:
            filename = f"{name}_{di}.txt"
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, [di], [3 * di], [3 * di], parameters + ["--error_model", f"{error_model}"], max_N=max_N, min_error_cases=min_error_cases, rust_dir="$QECPlaygroundPath/backend/rust")
            print("$SRUN " + " ".join(command) + " > " + filename + " &")
        else:
            for p in p_vec:
                command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [3 * di], [3 * di], parameters + ["--error_model", f"{error_model}"], max_N=max_N, min_error_cases=min_error_cases)
                print(" ".join(command))

                # run experiment
                stdout, returncode = run_qec_playground_command_get_stdout(command)
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
                print_result = f"{full_result}"
                results.append(print_result)
                print(print_result)
        
            print("\n\n")
            print("\n".join(results))
            print("\n\n")

            with open(filename, "w", encoding="utf-8") as f:
                f.write("\n".join(results) + "\n")
