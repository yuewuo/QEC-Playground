import os, sys, subprocess, hjson, datetime
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "slurm_utilities"))
import slurm_distribute
from slurm_distribute import slurm_threads_or as STO
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "threshold_analyzer"))
from threshold_analyzer import qecp_benchmark_simulate_func_command_vec
from threshold_analyzer import run_qecp_command_get_stdout, compile_code_if_necessary
from threshold_analyzer import ThresholdAnalyzer

di_vec = [4,5,6,7,8]
p_vec = [0.0025,0.0030,0.0035,0.0040,0.0045,0.0050,0.0055,0.0060,0.0065,0.0070,0.0075,0.0080,0.0085,0.0090,0.0095,0.0100,0.0105,0.0110]

for name in ["biased", "standard"]:
    if name == "biased":
        p_min = 0.0040
        p_max = 0.0055
    else:
        p_min = 0.0040
        p_max = 0.0055
    local_p_vec = [p for p in p_vec if p >= p_min and p <= p_max]
    collected_data = []
    for di in di_vec:
        filename = os.path.join(os.path.dirname(__file__), f"{name}_{di}.txt")
        collected_data_row = []
        with open(filename, "r", encoding="utf8") as f:
            for line in f.readlines():
                lst = line.strip(" \r\n").split("\n")[-1].split(" ")
                p = float(lst[0])
                error_rate = float(lst[5])
                confidence_interval = float(lst[7])
                if p >= p_min and p <= p_max:
                    collected_data_row.append((error_rate, confidence_interval))
            collected_data.append(collected_data_row)

    def disabled_simulate_func(p, d, runtime_budget, p_graph=None):
        exit(1)
    threshold_analyzer = ThresholdAnalyzer(di_vec, disabled_simulate_func)
    threshold_analyzer.verbose = True
    threshold_analyzer.estimate_exiting(local_p_vec, collected_data, os.path.join(os.path.dirname(__file__), f"threshold_{name}.pdf"))
