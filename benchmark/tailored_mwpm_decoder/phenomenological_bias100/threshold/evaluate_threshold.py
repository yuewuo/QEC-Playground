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

rough_code_distances = [13, 15]
rough_runtime_budgets = [(30000, 30000, 3600), (30000, 30000, 3600)]  # (max_repeats, min_error_case, time_budget)
rough_init_search_start_p = 0.06
code_distances = [13, 15, 17, 19, 21]
runtime_budgets = [(30000, 30000, 3600 * 10)] * len(code_distances)  # each given 10 hours
target_relative_diff = 0.5
fit_samples = 7
bias_eta = 100

slurm_distribute.SLURM_DISTRIBUTE_TIME = "10:20:00"
# slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '10G'
slurm_distribute.SLURM_DISTRIBUTE_MEM_PER_TASK = '70G'  # necessary only for large code distances like 19 and 21
slurm_distribute.SLURM_DISTRIBUTE_CPUS_PER_TASK = 20
def generate_parameters(bias_eta):
    return f"-p{STO(0)} --code_type RotatedTailoredCode --bias_eta {bias_eta} --decoder tailored-mwpm --decoder_config {{\"pcmg\":true}} --noise_model phenomenological".split(" ")

PRECISE_RESULT_FILE = os.path.join(os.path.dirname(__file__), f"precise_result.hjson")
RESULT_FILE = os.path.join(os.path.dirname(__file__), f"result.hjson")
HISTORY_FILE = os.path.join(os.path.dirname(__file__), f"history.tmp")  # to avoid deleting important result
def time_str():
    now = datetime.datetime.now()
    return now.strftime("%Y-%m-%d %H:%M:%S.%f")\

for result_file in [RESULT_FILE, PRECISE_RESULT_FILE]:
    if not os.path.exists(result_file):
        with open(result_file, "w", encoding="utf8") as f:
            f.write(hjson.dumps({}, sort_keys=True))
            f.flush()

def truncate_write_file(f, content):
    f.seek(0)
    f.truncate()
    f.write(content)
    f.flush()

# once a rough estimate is missing, this will be set to True, and no precise estimation will be run afterwards
ROUGH_ESTIMATE_ONLY = False

compile_code_if_necessary()
@slurm_distribute.slurm_distribute_run(os.path.dirname(__file__))
def experiment(slurm_commands_vec = None, run_command_get_stdout=run_qecp_command_get_stdout):

    with open(RESULT_FILE, "r", encoding="utf8") as f:
        log_obj = hjson.loads(f.read())
        need_rough_estimate = f"bias_eta_{bias_eta}" not in log_obj
    if need_rough_estimate:
        global ROUGH_ESTIMATE_ONLY
        ROUGH_ESTIMATE_ONLY = True

    print(f"bias_eta: {bias_eta}")

    parameters = generate_parameters(bias_eta)
    def generate_command(p, d, runtime_budget, p_center=None):
        max_repeats, min_error_case, time_budget = runtime_budget
        command = qecp_benchmark_simulate_func_command_vec(p, d, d, d, parameters, max_repeats=max_repeats, min_error_cases=min_error_case, time_budget=time_budget, p_graph=p_center)
        return command

    def simulate_func(p, d, runtime_budget, p_graph=None):
        command = generate_command(p, d, runtime_budget, p_graph)
        stdout, returncode = run_command_get_stdout(command)
        assert returncode == 0, "command fails..."
        full_result = stdout.strip(" \r\n").split("\n")[-1]
        lst = full_result.split(" ")
        error_rate = float(lst[5])
        confidence_interval = float(lst[7])
        return (error_rate, confidence_interval)
    threshold_analyzer = ThresholdAnalyzer(code_distances, simulate_func)
    threshold_analyzer.rough_code_distances = rough_code_distances
    threshold_analyzer.verbose = True
    threshold_analyzer.rough_runtime_budgets = rough_runtime_budgets
    threshold_analyzer.rough_init_search_start_p = rough_init_search_start_p
    threshold_analyzer.code_distances = code_distances
    threshold_analyzer.runtime_budgets = runtime_budgets
    threshold_analyzer.target_relative_diff = target_relative_diff
    threshold_analyzer.fit_samples = fit_samples

    with open(RESULT_FILE, "r", encoding="utf8") as f:
        log_obj = hjson.loads(f.read())
        need_rough_estimate = f"bias_eta_{bias_eta}" not in log_obj
    if need_rough_estimate:
        if slurm_commands_vec is not None:
            print("[warning] you're running rough estimation in slurm cluster")
            print("          the best practice is to run it locally and synchronize result.hjson to slurm cluster")
            exit(1)
        # run rough estimate and save to file
        rough_popt, rough_perr = threshold_analyzer.rough_estimate()
        with open(RESULT_FILE, "r+", encoding="utf8") as f:
            log_obj = hjson.loads(f.read())
            log_obj[f"bias_eta_{bias_eta}"] = [rough_popt, rough_perr]
            truncate_write_file(f, hjson.dumps(log_obj, sort_keys=True))
        with open(HISTORY_FILE, "a", encoding="utf8") as f:
            f.write(f"{time_str()}\n")
            f.write(hjson.dumps(["rough estimate", bias_eta, rough_popt, rough_perr]))
            f.write("\n\n")
            f.flush()
    
    if ROUGH_ESTIMATE_ONLY:
        return

    # read existing estimate (not necessarily rough estimate, but can also be a previous precise estimate)
    with open(RESULT_FILE, "r", encoding="utf8") as f:
        log_obj = hjson.loads(f.read())
        rough_popt, rough_perr = log_obj[f"bias_eta_{bias_eta}"]
    if slurm_commands_vec is not None:
        collected_parameters, p_list = threshold_analyzer.precise_estimate_parameters(rough_popt)
        for collected_parameters_row in collected_parameters:
            for (p, d, runtime_budget, p_center) in collected_parameters_row:
                command = generate_command(p, d, runtime_budget, p_center)
                slurm_commands_vec.sanity_checked_append(command)
        return  # skip handling results

    # at this point, if in slurm, all results have been cached
    popt, perr = threshold_analyzer.precise_estimate(rough_popt)
    with open(PRECISE_RESULT_FILE, "r+", encoding="utf8") as f:
        log_obj = hjson.loads(f.read())
        log_obj[f"bias_eta_{bias_eta}"] = [popt, perr]
        truncate_write_file(f, hjson.dumps(log_obj, sort_keys=True))
    with open(HISTORY_FILE, "a", encoding="utf8") as f:
        f.write(f"{time_str()}\n")
        f.write(hjson.dumps(["precise estimate", bias_eta, popt, perr]))
        f.write("\n\n")
        f.flush()
    image_path = os.path.join(os.path.dirname(__file__), f"threshold_bias_eta_{bias_eta}.pdf")
    threshold_analyzer.save_image_collected_data(image_path, popt, perr, *threshold_analyzer.collected_data_list[-1])
