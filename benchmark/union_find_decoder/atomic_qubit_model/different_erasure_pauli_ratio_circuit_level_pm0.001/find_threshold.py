import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import AutomatedThresholdEvaluator, qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command, run_qec_playground_command_get_stdout, compile_code_if_necessary
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO
from slurm_distribute import cpu_hours as CH
slurm_distribute.SLURM_DISTRIBUTE_TIME = "24:00:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '16G'
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 36

pair = [ (11, 11, 11), (15, 15, 15) ]  # (di, dj, T)
# original time: 60 cores for 5min, which is 60*5/60 = 5 CPU hours
init_measurement_error_rate = 0.001
error_model_configuration = f'{{"initialization_error_rate":{init_measurement_error_rate},"measurement_error_rate":{init_measurement_error_rate},"use_correlated_pauli":true}}'
parameters = f"-p{STO(0)} --decoder UF --max_half_weight 10 --time_budget {CH(5)} --use_xzzx_code --error_model OnlyGateErrorCircuitLevelCorrelatedErasure".split(" ") + ["--error_model_configuration", error_model_configuration]

run_specific_pauli_ratio = None
if len(sys.argv) > 1:
    run_specific_pauli_ratio = sys.argv[1]

def bothprint(*args, **kwargs):
    print(*args, **kwargs, flush=True)
    print(*args, **kwargs, flush=True, file=sys.stderr)

# customize simulator runner
def make_simulator_runner(pauli_ratio_str):
    def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
        di, dj, T = pair_one
        min_error_cases = min_error_cases if is_rough_test else max_N
        pauli_ratio = float(pauli_ratio_str)
        p_pauli = p * pauli_ratio
        p_erasure = p * (1 - pauli_ratio)
        command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p_pauli], [di], [dj], [T], parameters + ["--pes", f"[{p_erasure}]"], max_N, min_error_cases)
        if verbose:
            bothprint(" ".join(command))
        stdout, returncode = run_qec_playground_command_get_stdout(command)
        if verbose:
            bothprint("")
            bothprint(stdout)
        assert returncode == 0, "command fails..."
        full_result = stdout.strip(" \r\n").split("\n")[-1]
        lst = full_result.split(" ")
        error_rate = float(lst[5])
        confidence_interval = float(lst[7])
        return error_rate, confidence_interval, full_result + f" {p} {pauli_ratio_str}"
    return simulator_runner

results = []
pauli_ratio_strs = ["0"] + [f"{0.01 * i:.3g}" for i in range(1, 5)] + [f"{0.05 * i:.3g}" for i in range(1, 20+1)]

if run_specific_pauli_ratio is not None:
    print("run_specific_pauli_ratio:", run_specific_pauli_ratio)

    simulator_runner = make_simulator_runner(run_specific_pauli_ratio)
    evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters, simulator_runner=simulator_runner)
    evaluator.searching_lower_bound = 0.001
    evaluator.searching_upper_bound = 0.05
    evaluator.target_threshold_accuracy = 0.02
    threshold, relative_confidence_interval = evaluator.evaluate_threshold()
    print(f"pair: {pair}")
    print(f"parameters: {parameters}")
    print(f"threshold = {threshold}")
    print(f"relative_confidence_interval = {relative_confidence_interval}")
    print(f"{run_specific_pauli_ratio} {threshold} {relative_confidence_interval}")

    exit(0)  # exit script immediately, this is the spawned script


print("pauli_ratio_strs:", pauli_ratio_strs)

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    for pauli_ratio_str in pauli_ratio_strs:
        command = ["python3", os.path.abspath(__file__), pauli_ratio_str]
        if slurm_commands_vec is not None:
            slurm_commands_vec.sanity_checked_append(command)
            continue

        # run experiment
        stdout, returncode = run_command_get_stdout(command)
        print("\n" + stdout)
        assert returncode == 0, "command fails..."

        # full result
        full_result = stdout.strip(" \r\n").split("\n")[-1]
        lst = full_result.split(" ")
        run_specific_pauli_ratio = lst[0]
        assert run_specific_pauli_ratio == pauli_ratio_str, "sanity check"
        threshold = float(lst[1])
        relative_confidence_interval = float(lst[2])

        results.append(full_result)

    if slurm_commands_vec is not None:
        return

    print("\n".join(results))

    filename = os.path.join(os.path.dirname(__file__), f"thresholds.txt")
    with open(filename, "w", encoding="utf8") as f:
        f.write("\n".join(results) + "\n")
