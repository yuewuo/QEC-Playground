import os, sys
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(__file__), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
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
# original time: 60 cores for 10min, which is 60*10/60 = 10 CPU hours
common_parameters = f"-p{STO(0)} --time_budget {CH(10)} --error_model Arxiv200404693".split(" ")

run_specific_idx = None
if len(sys.argv) > 1:
    run_specific_idx = int(sys.argv[1])

def bothprint(*args, **kwargs):
    print(*args, **kwargs, flush=True)
    print(*args, **kwargs, flush=True, file=sys.stderr)

# customize simulator runner
def simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=1000000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [dj], [T], parameters, max_N, min_error_cases)
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
    return error_rate, confidence_interval, full_result

results = []
pauli_ratio_strs = ["0"] + [f"{0.01 * i:.3g}" for i in range(1, 5)] + [f"{0.05 * i:.3g}" for i in range(1, 20+1)]

parameters_vec = []
settings_vec = []

# generate all interested configs
for UF_decoder in [False, True]:
    for no_autotune in [False, True]:
        for autotune_minus_no_error in [False, True]:
            for use_combined_probability in [False, True]:
                for use_nature_initialization_error in [False]:  # no need to iterate for now
                    for use_nature_measurement_error in [False]:  # no need to iterate for now
                        error_model_configuration = f'{{"use_nature_initialization_error":{"true" if use_nature_initialization_error else "false"},"use_nature_measurement_error":{"true" if use_nature_measurement_error else "false"}}}'
                        parameters = common_parameters + ["--error_model_configuration", error_model_configuration]
                        if UF_decoder:
                            parameters += ["--decoder", "UF", "--max_half_weight", "20"]
                        if no_autotune:
                            parameters += ["--no_autotune"]
                        if not autotune_minus_no_error:
                            parameters += ["--disable_autotune_minus_no_error"]
                        if not use_combined_probability:
                            parameters += ["--disable_combined_probability"]
                        parameters_vec.append(parameters)
                        settings_vec.append((
                            UF_decoder,
                            no_autotune,
                            autotune_minus_no_error,
                            use_combined_probability,
                            use_nature_initialization_error,
                            use_nature_measurement_error,
                        ))

if run_specific_idx is not None:
    print("run_specific_idx:", run_specific_idx)

    evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters_vec[run_specific_idx], simulator_runner=simulator_runner)
    evaluator.searching_lower_bound = 0.001
    evaluator.searching_upper_bound = 0.03
    evaluator.target_threshold_accuracy = 0.02
    threshold, relative_confidence_interval = evaluator.evaluate_threshold()
    print(f"pair: {pair}")
    print(f"parameters: {parameters}")
    print(f"threshold = {threshold}")
    print(f"relative_confidence_interval = {relative_confidence_interval}")
    print(f"UF_decoder, no_autotune, autotune_minus_no_error, use_combined_probability, use_nature_initialization_error, use_nature_measurement_error = {settings_vec[run_specific_idx]}")
    print(f"{run_specific_idx} {threshold} {relative_confidence_interval}")

    exit(0)  # exit script immediately, this is the spawned script


compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qec_playground_command_get_stdout):
    for idx, parameters in enumerate(parameters_vec):
        command = ["python3", os.path.abspath(__file__), str(idx)]
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
        run_specific_idx = lst[0]
        assert run_specific_idx == str(idx), "sanity check"
        threshold = float(lst[1])
        relative_confidence_interval = float(lst[2])

        UF_decoder, no_autotune, autotune_minus_no_error, use_combined_probability, use_nature_initialization_error, use_nature_measurement_error = settings_vec[idx]
        converted_result = f"{UF_decoder} {no_autotune} {autotune_minus_no_error} {use_combined_probability} {use_nature_initialization_error} {use_nature_measurement_error} {threshold} {relative_confidence_interval}"

        results.append(converted_result)

    if slurm_commands_vec is not None:
        return

    print("\n".join(results))

    filename = os.path.join(os.path.dirname(__file__), f"thresholds.txt")
    with open(filename, "w", encoding="utf8") as f:
        f.write("# UF_decoder, no_autotune, autotune_minus_no_error, use_combined_probability, use_nature_initialization_error, use_nature_measurement_error, threshold, relative_confidence_interval" + "\n")
        f.write("\n".join(results) + "\n")
