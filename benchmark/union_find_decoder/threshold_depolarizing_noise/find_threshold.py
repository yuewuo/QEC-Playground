import os, sys, subprocess, hjson, datetime
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
sys.path.insert(0, os.path.join(qec_playground_root_dir, "benchmark", "threshold_analyzer"))
from threshold_analyzer import qecp_benchmark_simulate_func_command_vec
from threshold_analyzer import run_qecp_command_get_stdout
from threshold_analyzer import ThresholdAnalyzer

rough_code_distances = [3, 5]
code_distances = [5, 7, 9]
code_distances = [7, 9, 11, 13, 15]

# example of how to wrap qecp_benchmark_simulate_func_basic: CSS surface code with single round of perfect measurement
def example_qecp_benchmark_simulate_func(p, d, runtime_budget, p_graph=None):
    min_error_case, time_budget = runtime_budget
    parameters = f"-p0 --code_type StandardPlanarCode --noise_model depolarizing-noise --decoder union-find --decoder_config {{\"pcmg\":true,\"max_half_weight\":20,\"use_real_weighted\":true}}".split(" ")
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



rough_popt = [0.06190289499205579, 4.5358611979731664, 95.5169816090522, 0.0070900081943654025, 0.8812780953871281]
threshold_analyzer.target_relative_diff = 0.06
threshold_analyzer.fit_samples = 9
popt, perr = threshold_analyzer.precise_estimate(rough_popt)
threshold_analyzer.save_image_collected_data(os.path.join(os.path.dirname(__file__), f"threshold.pdf")
                                             , popt, perr, *threshold_analyzer.collected_data_list[-1])



"""
[info] fitting collected data:
    p=0.0067930108632623925
        d=7, pl=0.09447876589736456, dev=0.014
        d=9, pl=0.09264338831279109, dev=0.014
        d=11, pl=0.08868873702311326, dev=0.014
        d=13, pl=0.08388825461916634, dev=0.014
        d=15, pl=0.0818528640090215, dev=0.014
    p=0.006867260196038145
        d=7, pl=0.09906221106855105, dev=0.014
        d=9, pl=0.09683735992985061, dev=0.014
        d=11, pl=0.0929944203347799, dev=0.014
        d=13, pl=0.09195725246993676, dev=0.014
        d=15, pl=0.08841211835435576, dev=0.014
    p=0.006941509528813897
        d=7, pl=0.10310105048514773, dev=0.014
        d=9, pl=0.10005557716889901, dev=0.014
        d=11, pl=0.10033834439780828, dev=0.014
        d=13, pl=0.09767186982871343, dev=0.014
        d=15, pl=0.09590384041600955, dev=0.014
    p=0.00701575886158965
        d=7, pl=0.10829884006353178, dev=0.014
        d=9, pl=0.10685305553082243, dev=0.014
        d=11, pl=0.10587737047668346, dev=0.014
        d=13, pl=0.10377227973898406, dev=0.014
        d=15, pl=0.10235530939735468, dev=0.014
    p=0.007090008194365402
        d=7, pl=0.1108613309518677, dev=0.014
        d=9, pl=0.11211658829757419, dev=0.014
        d=11, pl=0.11112139898643852, dev=0.014
        d=13, pl=0.11124430740331082, dev=0.014
        d=15, pl=0.11049723756906077, dev=0.014
    p=0.007164257527141155
        d=7, pl=0.11621345935337259, dev=0.014
        d=9, pl=0.11556808186901728, dev=0.014
        d=11, pl=0.11715587373901724, dev=0.014
        d=13, pl=0.11730474732006126, dev=0.014
        d=15, pl=0.12001066702223408, dev=0.014
    p=0.007238506859916907
        d=7, pl=0.11835642379490147, dev=0.014
        d=9, pl=0.12295933004549242, dev=0.014
        d=11, pl=0.12191157347204161, dev=0.014
        d=13, pl=0.12524961557462028, dev=0.014
        d=15, pl=0.12849708383006975, dev=0.014
    p=0.00731275619269266
        d=7, pl=0.12350007203770659, dev=0.014
        d=9, pl=0.12651377242983758, dev=0.014
        d=11, pl=0.13111661446572948, dev=0.014
        d=13, pl=0.13389071542272274, dev=0.014
        d=15, pl=0.13853020392458637, dev=0.014
    p=0.007387005525468412
        d=7, pl=0.12814562229040455, dev=0.014
        d=9, pl=0.13235737446968082, dev=0.014
        d=11, pl=0.13801564177273423, dev=0.014
        d=13, pl=0.14221495386171154, dev=0.014
        d=15, pl=0.14830162875573608, dev=0.013
[dump] collected_data = [[(0.09447876589736456, 0.014), (0.09906221106855105, 0.014), (0.10310105048514773, 0.014), (0.10829884006353178, 0.014), (0.1108613309518677, 0.014), (0.11621345935337259, 0.014), (0.11835642379490147, 0.014), (0.12350007203770659, 0.014), (0.12814562229040455, 0.014)], [(0.09264338831279109, 0.014), (0.09683735992985061, 0.014), (0.10005557716889901, 0.014), (0.10685305553082243, 0.014), (0.11211658829757419, 0.014), (0.11556808186901728, 0.014), (0.12295933004549242, 0.014), (0.12651377242983758, 0.014), (0.13235737446968082, 0.014)], [(0.08868873702311326, 0.014), (0.0929944203347799, 0.014), (0.10033834439780828, 0.014), (0.10587737047668346, 0.014), (0.11112139898643852, 0.014), (0.11715587373901724, 0.014), (0.12191157347204161, 0.014), (0.13111661446572948, 0.014), (0.13801564177273423, 0.014)], [(0.08388825461916634, 0.014), (0.09195725246993676, 0.014), (0.09767186982871343, 0.014), (0.10377227973898406, 0.014), (0.11124430740331082, 0.014), (0.11730474732006126, 0.014), (0.12524961557462028, 0.014), (0.13389071542272274, 0.014), (0.14221495386171154, 0.014)], [(0.0818528640090215, 0.014), (0.08841211835435576, 0.014), (0.09590384041600955, 0.014), (0.10235530939735468, 0.014), (0.11049723756906077, 0.014), (0.12001066702223408, 0.014), (0.12849708383006975, 0.014), (0.13853020392458637, 0.014), (0.14830162875573608, 0.013)]]
[info] fit result: A = 0.1108578932739892 ± 0.0005709127850247237, B = 7.15242961079611 ± 0.6672497685692098 C = 99.99999999999999 ± 33.509080097733175
                   pc0 = 0.007083264160274917 ± 7.325442101326065e-06, v0 = 0.9826627873739233 ± 0.03607068842184267
    popt: [0.1108578932739892, 7.15242961079611, 99.99999999999999, 0.007083264160274917, 0.9826627873739233]
    perr: [0.0005709127850247237, 0.6672497685692098, 33.509080097733175, 7.325442101326065e-06, 0.03607068842184267]
"""
