import os, sys, math
import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner

# is_rough_test = True
is_rough_test = False
pair_ones = [((3, 9, 9), 400.), ((5, 15, 15), 100.), ((7, 21, 21), 40.)]

def get_result_with_bias_eta(bias_eta, pair_one, is_MWPM=True):
    pCX = 0.006  # fix pCX instead of fix p=pz
    p = pCX / (2 + 12 / float(bias_eta))
    parameters = "-p0 --use_xzzx_code --noise_model GenericBiasedWithBiasedCX".split(" ")
    parameters = parameters + f"--bias_eta {bias_eta}".split(" ")
    if not is_MWPM:
        parameters = parameters + "--decoder UF --max_half_weight 10".split(" ")
    verbose = True
    runner = qec_playground_fault_tolerant_MWPM_simulator_runner
    max_N = 100000 if is_rough_test else 100000000
    min_error_cases = 3000 if is_rough_test else 10000
    error_rate, confidence_interval, full_result = runner(p, pair_one, parameters, True, verbose, use_fake_runner=False, max_N=max_N, min_error_cases=min_error_cases)
    return error_rate, confidence_interval, full_result

bias_eta_vec = [1, 10, 100, 1000, "+inf"]
if not is_rough_test:
    detailed_count_10 = 5
    basic_vec = [10 ** (i / detailed_count_10) for i in range(detailed_count_10)]
    bias_eta_vec = basic_vec + [e * 10 for e in basic_vec] + [e * 100 for e in basic_vec] + [1000, "+inf"]

for pair_one, max_zeta in pair_ones:
    MWPM_results = []
    UF_results = []
    for bias_eta in bias_eta_vec:
        if bias_eta > max_zeta:
            break
        # MWPM result
        error_rate, confidence_interval, full_result = get_result_with_bias_eta(bias_eta, pair_one, is_MWPM=True)
        MWPM_results.append((bias_eta, error_rate, confidence_interval, full_result))
        print("MWPM:", full_result)
        # UF result
        error_rate, confidence_interval, full_result = get_result_with_bias_eta(bias_eta, pair_one, is_MWPM=False)
        UF_results.append((bias_eta, error_rate, confidence_interval, full_result))
        print("UF:", full_result)

    assert len(MWPM_results) == len(UF_results)
    print("\n\n\n\n")
    print(f"pair_one: {pair_one}")
    print("MWPM results:")
    for bias_eta, error_rate, confidence_interval, full_result in MWPM_results:
        print(bias_eta, full_result)
    print("UF results:")
    for bias_eta, error_rate, confidence_interval, full_result in UF_results:
        print(bias_eta, full_result)
    print("\n\n\n\n")
