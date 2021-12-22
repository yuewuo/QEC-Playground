import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout

di_vec = [di for di in range(3, 25 +1, 2)]

parameters = f"--debug_print_only --decoder UF --max_half_weight 10 --use_xzzx_code --error_model GenericBiasedWithBiasedCX".split(" ")

for di in di_vec:

    command = ["/usr/bin/time", "-v", "--output", os.path.join(os.path.dirname(__file__), f"d_{di}.txt")] + qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([0.01], [di], [di], [di], parameters)
    print(" ".join(command))

    # run experiment
    stdout, returncode = run_qec_playground_command_get_stdout(command, stderr_to_stdout=True)
    assert returncode == 0, "command fails..."
