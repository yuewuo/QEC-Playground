import os
import sys
import subprocess
import sys
import json
import tempfile
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
from colour import Color
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
# for more usable machines, use `SLURM_USE_SCAVENGE_PARTITION=1` flag
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 12

simulation_parameters = f"""-p{STO(0)}""".split(" ")

# rotated surface code only supports odd number code distances
di_vec = [3, 5, 7, 9, 11, 13]
T_vec = [6, 10, 14, 18, 22, 26]
p_vec = [0.5 * (10 ** (- i / 5)) for i in range(5 * 4 + 1)]
p_vec[0] = 0.4
min_error_cases = 40000
max_N = 100000000


def force_single_thread(parameters):
    def process(parameter):
        if parameter.startswith("-p"):
            return "-p1"
        return parameter
    return [process(parameter) for parameter in parameters]


def vec_command_with_partition(p, d, T, parameters, decoder_config=None, max_N=100000, min_error_cases=3000, rust_dir=rust_dir, time_budget=None):
    # first get partition
    assert "--decoder" not in parameters, "please do not specify decoder, because this function assumes parallel-fusion decoder"
    assert "--decoder-config" not in parameters, "please do not specify decoder_config in parameters, use `decoder_config` instead"
    filename = tempfile.NamedTemporaryFile(
        suffix='.syndromes', delete=False).name
    tmp_parameters = ["--debug-print", "fusion-blossom-syndrome-file",
                      "--fusion-blossom-syndrome-export-filename", filename, "--decoder", "fusion"]
    tmp_command = qec_playground_benchmark_simulator_runner_vec_command(
        [p], [d], [d], [T], force_single_thread(parameters) + tmp_parameters, max_N=1, min_error_cases=1)
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


def name_log10(e):
    if e == 0:
        return "1"
    if e == -1:
        return "0.1"
    return f"$10^{{{e}}}$"


def plot_setup():
    ticks_p_log10 = [-4, -3, -2, -1, 0]
    plt.xticks([np.log(10**e) for e in ticks_p_log10],
               [name_log10(e) for e in ticks_p_log10])
    plt.xlim(np.log(5e-5), np.log(0.3))
    ticks_pL_log10 = [-6, -5, -4, -3, -2, -1, 0]
    plt.yticks([np.log(10**e) for e in ticks_pL_log10],
               [name_log10(e) for e in ticks_pL_log10])
    plt.ylim(np.log(1e-6), np.log(1))


colors = {}
for di, color in zip(di_vec, list(mcolors.TABLEAU_COLORS)[:len(di_vec)]):
    rgb = mcolors.TABLEAU_COLORS[color]
    colors[di] = rgb


def mix_color(color1, color2, mix=0.5):
    c1 = Color(color1).rgb
    c2 = Color(color2).rgb
    cm = [c1[i] * mix + c2[i] * (1 - mix) for i in range(3)]
    return Color(rgb=tuple(cm)).hex


def read_file(filename) -> list[list[float], list[float], list[float]]:
    curve_p = []
    curve_pL = []
    curve_dev = []
    with open(filename, encoding='utf8') as f:
        lines = f.readlines()
        for line in lines:
            line = line.strip(" \r\n")
            lst = line.split(" ")
            if len(lst) < 7 or line.startswith("#"):
                print(f"[warning] data missing at di={di}, T={T}")
            else:
                p = float(lst[0])
                total_rounds = int(lst[3])
                error_count = int(lst[4])
                error_rate = float(lst[5])
                confidence_interval = float(lst[7])
            if not confidence_interval < 0.2:
                continue
            curve_p.append(p)
            curve_pL.append(error_rate)
            curve_dev.append(confidence_interval)
    return curve_p, curve_pL, curve_dev


def plot_folder(folder_path, no_label=False, mix=1, linestyle="solid"):
    for (di, T) in zip(di_vec, T_vec):
        filename = os.path.join(folder_path, f"d{di}_T{T}.txt")
        curve_p, curve_pL, curve_dev = read_file(filename)
        color = mix_color(colors[di], "white", mix)
        plt.plot([np.log(p) for p in curve_p], [np.log(pL)
                 for pL in curve_pL], label=None if no_label else f"$d={di}$",
                 color=color, linestyle=linestyle)
        upper = []
        lower = []
        for pL, dev in zip(curve_pL, curve_dev):
            upper.append(np.log((1 + dev)))
            lower.append(-np.log((1 - dev)))
        plt.errorbar([np.log(p) for p in curve_p], [np.log(pL) for pL in curve_pL],
                     yerr=(lower, upper), fmt='o', capsize=3, markersize=3, ecolor=color, color=color)


def relative_plot_setup():
    ticks_p_log10 = [-4, -3, -2, -1, 0]
    plt.xticks([np.log(10**e) for e in ticks_p_log10],
               [name_log10(e) for e in ticks_p_log10])
    plt.xlim(np.log(5e-5), np.log(0.3))
    plt.ylim(0.7, 1.5)


def relative_plot_folder(folder_path1, folder_path2):
    for (di, T) in zip(di_vec, T_vec):
        filename1 = os.path.join(folder_path1, f"d{di}_T{T}.txt")
        curve_p1, curve_pL1, curve_dev1 = read_file(filename1)
        filename2 = os.path.join(folder_path2, f"d{di}_T{T}.txt")
        curve_p2, curve_pL2, curve_dev2 = read_file(filename2)
        end = max([i+1 for i in range(min(len(curve_p1), len(curve_p2)))
                  if curve_p1[i] == curve_p2[i]])
        curve_p = curve_p1[:end]
        curve_pL = [curve_pL1[i] / curve_pL2[i] for i in range(end)]
        curve_dev = [np.sqrt(curve_dev1[i] ** 2 + curve_dev2[i] ** 2)
                     for i in range(end)]
        color = colors[di]
        yerr = [curve_dev[i] * curve_pL[i] for i in range(end)]
        plt.errorbar([np.log(p) for p in curve_p], curve_pL,
                     yerr=yerr, fmt='-o', capsize=3, markersize=3, ecolor=color, color=color, label=f"$d={di}$")


def plot_large(script_folder):

    outputs = [
        ("Fusion MWPM (solid) vs MWPM (dotted)",
         "fusion_mwpm", ["fusion_mwpm", "mwpm"]),
        ("Fusion UF (solid) vs UF (dotted)", "fusion_uf", ["fusion_uf", "uf"]),
        ("Compare All (Fusion, No) $\\times$ (MWPM, UF)",
         "compare_all", ["fusion_mwpm", "mwpm", "fusion_uf", "uf"])
    ]

    mix = [1.0, 0.5, 1, 0.5]
    linestyle = ["solid", "dashed", "solid", "dashed"]

    for title, filename, names in outputs:
        plt.cla()
        plot_setup()
        for idx, name in reversed(list(enumerate(names))):
            plot_folder(os.path.join(script_folder, name),
                        no_label=idx > 0, mix=mix[idx], linestyle=linestyle[idx])
        plt.legend()
        plt.title(title)
        # plt.show()
        plt.savefig(f"{filename}_large.pdf")


def plot_relative(script_folder):

    relative_outputs = [
        ("Fusion UF Relative to UF", "fusion_uf", ["fusion_uf", "uf"]),
        ("Fusion MWPM Relative to MWPM",
         "fusion_mwpm", ["fusion_mwpm", "mwpm"]),
    ]

    for title, filename, names in relative_outputs:
        plt.cla()
        relative_plot_setup()
        relative_plot_folder(os.path.join(
            script_folder, names[0]), os.path.join(script_folder, names[1]))
        plt.legend()
        plt.title(title)
        # plt.show()
        plt.savefig(f"{filename}_relative.pdf")
