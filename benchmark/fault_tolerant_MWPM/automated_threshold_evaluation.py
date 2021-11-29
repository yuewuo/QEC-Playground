"""

Author: Yue Wu (yue.wu@yale.edu)

I found it's super annoying to test the threshold of multiple parameters (for example to plot the relationship between biase ζ and threshold).
I would like to write a general tool to automate the evaluation of any QEC threshold, by
1. roughly search the threshold point
2. more accurate logical error rate around the threshold point
3. output the threshold and the confidence interval

This script supports simulator in https://github.com/yuewuo/QEC-Playground by default, and can add support to other simulators by adding functions

"""

import os, sys, math, subprocess, random
import scipy.stats
import numpy as np
import tempfile

qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")

def main():
    # # test basic command runner
    # random_error_rate, confidence_interval, full_result = qec_playground_fault_tolerant_MWPM_simulator_runner(0.005, (5, 5, 5), "-b10 -p0 --use_xzzx_code --bias_eta 100 --error_model GenericBiasedWithBiasedCX".split(" "), True, True)
    # print(random_error_rate, confidence_interval, full_result)
    # exit(0)

    # UnionFind Decoder (max_half_weight = 10), XZZX code, 
    pair = [ (4, 12, 12), (5, 15, 15) ]  # (di, dj, T)
    parameters = "-p0 --use_xzzx_code --bias_eta 10 --error_model GenericBiasedWithBiasedCX".split(" ")
    evaluator = AutomatedThresholdEvaluator(pair, parameters=parameters)
    threshold, relative_confidence_interval = evaluator.evaluate_threshold()
    print("\n\nresult:")
    print(f"pair: {pair}")
    print(f"parameters: {parameters}")
    print(f"threshold = {threshold}")
    print(f"relative_confidence_interval = {relative_confidence_interval}")

"""
use_fake_runner: help to debug the script, by using a simple error rate model 0.2 * (p/pth)^d

return:
    (error_rate, confidence_interval, full_result)
    error_rate: float
    confidence_interval: float
    full_result: str
"""
QEC_PLAYGROUND_COMPILATION_DONE = False
if 'MANUALLY_COMPILE_QEC' in os.environ and os.environ["MANUALLY_COMPILE_QEC"] == "TRUE":
    QEC_PLAYGROUND_COMPILATION_DONE = True
def compile_code_if_necessary():
    global QEC_PLAYGROUND_COMPILATION_DONE
    if QEC_PLAYGROUND_COMPILATION_DONE is False:
        process = subprocess.Popen(["cargo", "build", "--release"], universal_newlines=True, stdout=sys.stdout, stderr=sys.stderr, cwd=rust_dir)
        process.wait()
        assert process.returncode == 0, "compile has error"
        QEC_PLAYGROUND_COMPILATION_DONE = True
def run_qec_playground_command_get_stdout(command, no_stdout=False, use_tmp_out=False):
    compile_code_if_necessary()
    env = os.environ.copy()
    env["RUST_BACKTRACE"] = "full"
    stdout = subprocess.PIPE
    if use_tmp_out:
        out_file = tempfile.NamedTemporaryFile(delete=False)
        out_filename = out_file.name
        stdout = out_file
    if no_stdout:
        stdout = sys.stdout
    process = subprocess.Popen(command, universal_newlines=True, env=env, stdout=stdout, stderr=sys.stderr, bufsize=100000000)
    process.wait()
    stdout, _ = process.communicate()
    if use_tmp_out:
        out_file.flush()
        out_file.close()
        with open(out_filename, "r", encoding="utf8") as f:
            stdout = f.read()
        os.remove(out_filename)
    return stdout, process.returncode

def qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command(p_vec, di_vec, dj_vec, T_vec, parameters, max_N=100000, min_error_cases=3000, rust_dir=rust_dir):
    p_str = "[" + ",".join([f"{e:.8e}" for e in p_vec]) + "]"
    di_str = "[" + ",".join([str(e) for e in di_vec]) + "]"
    dj_str = "[" + ",".join([str(e) for e in dj_vec]) + "]"
    T_str = "[" + ",".join([str(e) for e in T_vec]) + "]"
    qecp_path = os.path.join(rust_dir, "target", "release", "rust_qecp")
    command = [qecp_path, "tool", "fault_tolerant_benchmark", di_str, "--djs", dj_str, T_str, f"-m{max_N}", f"-e{min_error_cases}", p_str] + parameters
    return command

def qec_playground_fault_tolerant_MWPM_simulator_runner(p, pair_one, parameters, is_rough_test, verbose, use_fake_runner=False, max_N=100000, min_error_cases=3000):
    di, dj, T = pair_one
    min_error_cases = min_error_cases if is_rough_test else max_N
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [dj], [T], parameters, max_N, min_error_cases)
    if verbose:
        print(" ".join(command))
    if use_fake_runner:
        origin_error_rate = 0.2 * math.pow(p / 0.07, di)
        confidence_interval = 0.025  # 3000 error cases
        error_rate = random_non_negative(origin_error_rate, confidence_interval)
        full_result = f"full result not available for fake runner {pair_one}, {p}, {error_rate}({confidence_interval})"
    else:
        stdout, returncode = run_qec_playground_command_get_stdout(command)
        if verbose:
            print("")
            print(stdout)
        assert returncode == 0, "command fails..."
        full_result = stdout.strip(" \r\n").split("\n")[-1]
        lst = full_result.split(" ")
        error_rate = float(lst[5])
        confidence_interval = float(lst[7])
    return error_rate, confidence_interval, full_result

def random_non_negative(error_rate, confidence_interval_95):
    stddev = error_rate * confidence_interval_95 / 1.96
    random_error_rate = 0
    while random_error_rate <= 0:
        random_error_rate = random.gauss(error_rate, stddev)
    return random_error_rate

class AutomatedThresholdEvaluator:
    def __init__(self, pair, parameters=[], simulator_runner=qec_playground_fault_tolerant_MWPM_simulator_runner):
        assert (isinstance(pair, list) or isinstance(pair, tuple)) and len(pair) == 2, "pair should be a list of 2"
        self.pair = pair
        self.parameters = parameters
        self.simulator_runner = simulator_runner
        # initialize searching parameters that can be later adjusted
        self.searching_upper_bound = 0.5  # most threshold is between 1e-4 and 0.5
        self.searching_lower_bound = 0.0001
        self.target_threshold_accuracy = 0.05  # target a 5% accuracy of threshold would be reasonable
        self.reasonable_threshold_range = 1.0  # drop if intersection point is not in [p_estimate / (1 + self.reasonable_threshold_range), p_estimate * (1 + self.reasonable_threshold_range)]
        self.do_not_believe_logical_error_rate_above = 0.45  # don't believe the data if the logical error rate is more than 45%
        self.verbose = True
        self.accurate_sample_count = 5  # sample equally-spaced in [rough / (1 + self.target_threshold_accuracy), rough * (1 + self.target_threshold_accuracy)]
        self.random_intersection_count = 1000

    def get_rough_estimation(self):
        upper_bound = self.searching_upper_bound
        lower_bound = self.searching_lower_bound
        if self.verbose:
            print(f"rough threshold estimation:")
        while upper_bound / lower_bound - 1 > self.target_threshold_accuracy:
            searching_p = math.sqrt(upper_bound * lower_bound)
            error_rate_1, confidence_interval_1, _ = self.simulator_runner(searching_p, self.pair[0], self.parameters, True, self.verbose)
            # do not believe the data if logical error rate is too high
            if error_rate_1 >= self.do_not_believe_logical_error_rate_above:
                upper_bound = searching_p
                continue
            error_rate_2, confidence_interval_2, _ = self.simulator_runner(searching_p, self.pair[1], self.parameters, True, self.verbose)
            # do not believe the data if logical error rate is too high
            if error_rate_2 >= self.do_not_believe_logical_error_rate_above:
                upper_bound = searching_p
                continue
            if self.verbose:
                print(f"[{lower_bound}, {upper_bound}] searching_p = {searching_p} [1] {error_rate_1}({confidence_interval_1}) [2] {error_rate_2}({confidence_interval_2})")
            # # early break if the error rate is already indistinguishable
            # if abs(error_rate_1 - error_rate_2) <= error_rate_1 * confidence_interval_1 + error_rate_2 * confidence_interval_2:
            #     break
            if error_rate_1 > error_rate_2 or error_rate_1 == 0:
                lower_bound = searching_p
            else:
                upper_bound = searching_p
            
        return math.sqrt(upper_bound * lower_bound)

    def get_accurate_threshold(self, rough_estimation):
        sampling_p_lower_bound = rough_estimation / (1 + self.target_threshold_accuracy)
        sampling_step = math.pow(1 + self.target_threshold_accuracy, 2 / (self.accurate_sample_count - 1))
        sampling_p_higher_bound = sampling_p_lower_bound * math.pow(sampling_step, self.accurate_sample_count)
        sampling_p_vec = []
        for i in range(self.accurate_sample_count):
            sampling_p_vec.append(sampling_p_lower_bound * math.pow(sampling_step, i))
        if self.verbose:
            print(f"accurate threshold estimation: {sampling_p_vec}")
        result_1_vec = []
        result_2_vec = []
        for i in range(self.accurate_sample_count):
            searching_p = sampling_p_vec[i]
            error_rate_1, confidence_interval_1, fr1 = self.simulator_runner(searching_p, self.pair[0], self.parameters, False, self.verbose)
            error_rate_2, confidence_interval_2, fr2 = self.simulator_runner(searching_p, self.pair[1], self.parameters, False, self.verbose)
            result_1_vec.append((error_rate_1, confidence_interval_1, fr1))
            result_2_vec.append((error_rate_2, confidence_interval_2, fr2))
        if self.verbose:
            print("configuration 1:")
            for i in range(self.accurate_sample_count):
                print(result_1_vec[i][2])
            print("configuration 2:")
            for i in range(self.accurate_sample_count):
                print(result_2_vec[i][2])
        ln_pth_vec = []
        X = [math.log(p) for p in sampling_p_vec]
        ln_pth_lower_bound = math.log(rough_estimation / (1 + self.reasonable_threshold_range))
        ln_pth_upper_bound = math.log(rough_estimation * (1 + self.reasonable_threshold_range))
        for j in range(self.random_intersection_count):
            Y1 = [math.log(random_non_negative(e[0], e[1])) for e in result_1_vec]
            slope1, intercept1, _, _, _ = scipy.stats.linregress(X, Y1)
            Y2 = [math.log(random_non_negative(e[0], e[1])) for e in result_2_vec]
            slope2, intercept2, _, _, _ = scipy.stats.linregress(X, Y2)
            lnp = (intercept2 - intercept1) / (slope1 - slope2)
            if lnp < ln_pth_lower_bound or lnp > ln_pth_upper_bound:
                print(f"[warning] found unreasonable intersection point (threshold) value p = {math.exp(lnp)}, algorithm may fail")
                continue
            ln_pth_vec.append(lnp)
        threshold = math.exp(np.mean(ln_pth_vec))
        confidence_interval = 1.96 * np.std(ln_pth_vec)  # e^ε = 1 + ε when ε << 1
        is_extrapolated = threshold > sampling_p_higher_bound or threshold < sampling_p_lower_bound
        return threshold, confidence_interval, is_extrapolated

    def evaluate_threshold(self):
        # first roughly search the threshold point
        rough_estimation = self.get_rough_estimation()
        if self.verbose:
            print("rough_estimation:", rough_estimation)
        # more accurate logical error rate around the threshold point
        threshold, confidence_interval, is_extrapolated = self.get_accurate_threshold(rough_estimation)
        # if error exceeds the target or threshold is extrapolated, then re-run the experiment
        if is_extrapolated or confidence_interval > self.target_threshold_accuracy:
            threshold, confidence_interval, is_extrapolated = self.get_accurate_threshold(threshold)
            if is_extrapolated:
                print(f"[warning] extrapolated threshold value even after retry: {threshold} {confidence_interval}")
        return threshold, confidence_interval

if __name__ == "__main__":
    main()
