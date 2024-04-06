import os
import sys
import subprocess
import sys
import json
import tempfile
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(
    __file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_tolerant_MWPM_dir = os.path.join(
    qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_tolerant_MWPM_dir)
fusion_blossom_root_dir = os.path.join(
    os.path.dirname(qec_playground_root_dir), "fusion-blossom")
assert os.path.exists(
    fusion_blossom_root_dir), "please clone fusion-blossom alongside QEC-Playground folder"
sys.path.insert(0, os.path.join(fusion_blossom_root_dir, "scripts"))

if True:
    from automated_threshold_evaluation import qec_playground_benchmark_simulator_runner_vec_command
    from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
    sys.path.insert(0, os.path.join(qec_playground_root_dir,
                    "benchmark", "slurm_utilities"))
    import slurm_distribute
    from slurm_distribute import slurm_threads_or as STO
    from slurm_distribute import cpu_hours as CH
    from graph_time_partition import read_syndrome_file, graph_time_partition


slurm_distribute.SLURM_DISTRIBUTE_TIME = "1:20:00"
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '8G'
# for more usuable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12

simulation_parameters = f"""-p{STO(0)}""".split(" ")

# rotated surface code only supports odd number code distances
di_vec = [3, 5, 7, 9, 11, 13]
T_vec = [6, 10, 14, 18, 22, 26]
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
p_vec[0] = 0.4
min_error_cases = 40000
max_N = 100000000


def vec_command_with_partition(p, d, T, parameters, decoder_config=None, max_N=100000, min_error_cases=3000, rust_dir=rust_dir, time_budget=None):
    # first get partition
    assert "--decoder" not in parameters, "please do not specify decoder, because this function assumes parallel-fusion decoder"
    assert "--decoder-config" not in parameters, "please do not specify decoder_config in parameters, use `decoder_config` instead"
    filename = tempfile.NamedTemporaryFile(
        suffix='.syndromes', delete=False).name
    tmp_parameters = ["--debug-print", "fusion-blossom-syndrome-file",
                      "--fusion-blossom-syndrome-export-filename", filename, "--decoder", "fusion"]
    tmp_command = qec_playground_benchmark_simulator_runner_vec_command(
        [p], [d], [d], [T], parameters + tmp_parameters, max_N=1, min_error_cases=1)
    stdout, returncode = run_qec_playground_command_get_stdout(tmp_command)
    assert returncode == 0, "command fails..."
    initializer, positions = read_syndrome_file(filename)
    partition = graph_time_partition(initializer, positions)
    if os.path.exists(filename):
        os.remove(filename)
    if decoder_config is None:
        decoder_config = {}
    decoder_config["partition_config"] = json.loads(partition.to_json())
    add_parameters = ["--decoder", "parallel-fusion",
                      "--decoder-config", json.dumps(decoder_config, separators=(',', ':'))]
    return qec_playground_benchmark_simulator_runner_vec_command([p], [d], [d], [T], parameters + add_parameters, max_N=max_N, min_error_cases=min_error_cases, time_budget=time_budget)


def common_evaluation(directory, parameters, di_vec=di_vec, T_vec=T_vec, p_vec=p_vec, use_partitioned_decoder_config=None):

    compile_code_if_necessary()

    @ slurm_distribute.slurm_distribute_run(directory)
    def experiment(slurm_commands_vec=None, run_command_get_stdout=run_qec_playground_command_get_stdout):

        for (di, T) in zip(di_vec, T_vec):
            filename = os.path.join(directory, f"d{di}_T{T}.txt")

            results = []
            for p in p_vec:
                if use_partitioned_decoder_config is None:
                    command = qec_playground_benchmark_simulator_runner_vec_command(
                        [p], [di], [di], [T], parameters, max_N=max_N, min_error_cases=min_error_cases)
                else:
                    command = vec_command_with_partition(
                        p, di, T, parameters, decoder_config=use_partitioned_decoder_config, max_N=max_N,
                        min_error_cases=min_error_cases)
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
                if len(lst) < 7:
                    print_result = f"# data missing"
                else:
                    total_rounds = int(lst[3])
                    error_count = int(lst[4])
                    error_rate = float(lst[5])
                    confidence_interval = float(lst[7])
                    print_result = f"{full_result}"

                # record result
                results.append(print_result)
                print(print_result)

            if slurm_commands_vec is not None:
                continue

            print("\n\n")
            print("\n".join(results))
            print("\n\n")

            with open(filename, "w", encoding="utf-8") as f:
                f.write("\n".join(results) + "\n")
