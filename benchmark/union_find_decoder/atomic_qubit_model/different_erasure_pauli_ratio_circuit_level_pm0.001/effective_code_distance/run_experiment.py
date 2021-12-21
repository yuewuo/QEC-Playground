import os, sys, math, random, scipy.stats
import numpy as np
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

origin_folder = os.path.join(os.path.dirname(__file__), "..")
legacy_folder = os.path.join(os.path.dirname(__file__), "..", "..", "different_erasure_pauli_ratio_circuit_level", "legacy")

def read_origin_configurations():

    # read in the threshold
    thresholds = []
    with open(os.path.join(origin_folder,  "thresholds.txt"), "r", encoding="utf8") as f:
        lines = f.readlines()
        for line in lines:
            line = line.strip(" \r\n")
            if line == "":
                continue
            pauli_ratio, threshold, dev = line.split(" ")
            thresholds.append((pauli_ratio, float(threshold), float(dev)))

    configurations = []
    for (pauli_ratio, threshold, dev) in thresholds:
        ratio_configurations = []
        filepath = os.path.join(legacy_folder,  f"pauli_ratio_{pauli_ratio}.txt")
        with open(filepath, "r", encoding="utf8") as f:
            lines = f.readlines()
            for line in lines:
                line = line.strip(" \r\n")
                if line == "":
                    continue
                spt = line.split(" ")
                p_pth = float(spt[0])
                # p = float(spt[1])
                # pL = float(spt[7])
                # pL_dev = float(spt[9])
                ratio_configurations.append((p_pth, p_pth * threshold))
        configurations.append((pauli_ratio, ratio_configurations[-8:]))
    return configurations

configurations = read_origin_configurations()
# print(configurations)

di = 5
min_error_cases = 100000
# min_error_cases = 10  # debug

max_N = 100000000

slurm_distribute.SLURM_DISTRIBUTE_TIME = "05:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '4G'
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12  # use fewer cores for more available resources (use `SLURM_USE_SCAVENGE_PARTITION` option to speed up)
# 18000 sec for 12 cores, that is 60 CPU hours
init_measurement_error_rate = 0.001
error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate},"use_correlated_pauli":true}}'
parameters = f"-p{STO(0)} --decoder UF --max_half_weight 100 --time_budget {CH(60)} --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ") + ["--error_model_configuration", error_model_configuration]  # a maximum 60min for each point

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    lines = []
    for pauli_ratio, ratio_configurations in configurations:
        data = []
        for p_pth, p in ratio_configurations:
            p_pauli = p * float(pauli_ratio)
            p_erasure = p * (1 - float(pauli_ratio))
            command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [di], [di], parameters + ["--pes", f"[{p_erasure}]"], max_N=max_N, min_error_cases=min_error_cases)
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
            pL = float(lst[5])
            pL_dev = float(lst[7])

            data.append((p_pth, pL, pL_dev))

        if slurm_commands_vec is not None:
            continue

        X = [math.log(p_pth) for p_pth, pL, pL_dev in data]
        slope_vec = []
        for random_round in range(100):
            Y = [math.log(pL) for p_pth, pL, pL_dev in data]
            for i in range(len(data)):
                Y[i] += random.gauss(0, data[i][2] / 1.96)
            slope, intercept, _, _, _ = scipy.stats.linregress(X, Y)
            slope_vec.append(slope)
            # print(line, slope)
        slope = np.mean(slope_vec)
        slope_confidence_interval = 1.96 * np.std(slope_vec)

        line = f"{pauli_ratio} {slope} {slope_confidence_interval}"
        print(line)
        lines.append(line)

    if slurm_commands_vec is not None:
        return

    content = "\n".join(lines)
    print(content)
    with open(os.path.join(os.path.dirname(__file__), "effective_code_distance.txt"), "w", encoding="utf8") as f:
        f.write(content + "\n")
