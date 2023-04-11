import os, sys, subprocess, hjson, datetime
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "threshold_analyzer"))
from threshold_analyzer import qecp_benchmark_simulate_func_command_vec
from threshold_analyzer import run_qecp_command_get_stdout
from threshold_analyzer import ThresholdAnalyzer

rough_code_distances = [3, 5]
# code_distances = [5, 7, 9]
code_distances = [7, 9, 11, 13, 15]

# example of how to wrap qecp_benchmark_simulate_func_basic: CSS surface code with single round of perfect measurement
def example_qecp_benchmark_simulate_func(p, d, runtime_budget, p_graph=None):
    min_error_case, time_budget = runtime_budget
    parameters = f"-p0 --code_type StandardPlanarCode --noise_model depolarizing-noise --decoder fusion".split(" ")
    command = qecp_benchmark_simulate_func_command_vec(p, d, d, d, parameters, min_error_cases=min_error_case, time_budget=time_budget, p_graph=p_graph)
    stdout, returncode = run_qecp_command_get_stdout(command)
    assert returncode == 0, "command fails..."
    full_result = stdout.strip(" \r\n").split("\n")[-1]
    lst = full_result.split(" ")
    error_rate = float(lst[5])
    confidence_interval = float(lst[7])
    return (error_rate, confidence_interval)

simulate_func = example_qecp_benchmark_simulate_func
threshold_analyzer = ThresholdAnalyzer(code_distances, simulate_func, default_rough_runtime_budget=(6000, 2400), default_runtime_budget=(18000, 3600))
threshold_analyzer.rough_code_distances = rough_code_distances
threshold_analyzer.rough_init_search_start_p = 0.015  # threshold is below 1%
threshold_analyzer.verbose = True


# threshold_analyzer.estimate(save_image=os.path.join(os.path.dirname(__file__), f"threshold.pdf"))


rough_popt = [0.11201048955093067, 11.00874349616047, 99.99999999999999, 0.007693538428726712, 1.3198353584625055]
threshold_analyzer.target_relative_diff = 0.01
threshold_analyzer.fit_samples = 9
popt, perr = threshold_analyzer.precise_estimate(rough_popt)
threshold_analyzer.save_image_collected_data(os.path.join(os.path.dirname(__file__), f"threshold.pdf")
                                             , popt, perr, *threshold_analyzer.collected_data_list[-1])
