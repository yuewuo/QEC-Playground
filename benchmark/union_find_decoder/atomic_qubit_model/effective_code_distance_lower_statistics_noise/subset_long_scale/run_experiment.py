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
from slurm_distribute import cpu_hours as CH

origin_folder = os.path.join(os.path.dirname(__file__), "..", "..", "different_erasure_pauli_ratio_circuit_level")

# read in the threshold
thresholds = []
with open(os.path.join(origin_folder,  "thresholds.txt"), "r", encoding="utf8") as f:
    lines = f.readlines()
    for line in lines:
        line = line.strip(" \r\n")
        if line == "":
            continue
        pauli_ratio, threshold, dev = line.split(" ")
        if pauli_ratio not in ["0", "0.01", "0.02", "0.05", "0.1", "0.25", "0.5", "1"]:
            continue
        thresholds.append((pauli_ratio, float(threshold), float(dev)))

# print("subset:")
# print(thresholds)

di = 5
min_error_cases = 100000
# min_error_cases = 10  # debug

max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "05:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '4G'
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # use fewer cores for more available resources (use `SLURM_USE_SCAVENGE_PARTITION` option to speed up)
# 18000 sec for 12 cores, that is 60 CPU hours
parameters = f"-p{STO(0)} --decoder UF --max_half_weight 100 --time_budget {CH(60)} --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure --error_model_configuration {{\"use_correlated_pauli\":true}}".split(" ")  # a maximum 60min for each point

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    pth_L_results = []

    for pauli_ratio, threshold, _ in thresholds:

        # print(f"running pauli_ratio = {pauli_ratio}, threshold = {threshold}...")
        step = 0.8
        p_vec = []
        for i in range(-200, 200):
            pi = threshold * (step ** i)
            if pi < 0.2 and pi/threshold > 0.01:
                p_vec.append(pi)
        # print(p_vec)

        filename = os.path.join(os.path.dirname(__file__), f"pauli_ratio_{pauli_ratio}.txt")

        results = []
        for p in p_vec:
            p_pauli = p * float(pauli_ratio)
            p_erasure = p * (1 - float(pauli_ratio))
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [di], parameters + ["--pes", f"[{p_erasure:.8e}]"], max_N=max_N, min_error_cases=min_error_cases)
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
            error_count = int(lst[4])
            error_rate = float(lst[5])
            confidence_interval = float(lst[7])

            # record result
            print_result = f"{p / threshold} {p} " + full_result
            results.append(print_result)
            print(print_result)

            if p == threshold:
                pth_L_results.append(f"{pauli_ratio} {error_rate} {confidence_interval}")

            if error_count < min_error_cases * 0.001:
                break  # next is not trust-worthy, ignore every p behind it

        if slurm_commands_vec is not None:
            continue

        print("\n\n")
        print("\n".join(results))
        print("\n\n")

        with open(filename, "w", encoding="utf-8") as f:
            f.write("\n".join(results) + "\n")

    print("\n\n")
    print("\n".join(pth_L_results))
    print("\n\n")

    pth_L_filepath = os.path.join(os.path.dirname(__file__), f"pth_L.txt")
    with open(pth_L_filepath, "w", encoding="utf-8") as f:
        f.write("\n".join(pth_L_results) + "\n")
