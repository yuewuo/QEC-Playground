"""
author: Yue Wu (yue.wu@yale.edu)  2022.3.12


This automatic threshold analyzer works as follows:

1. use two small code distances (e.g. 5 and 7) to roughly evaluate the threshold parameters
2. estimate a list of p to collect for higher code distances
3. repeat (2) one more time if the result is not ideal, e.g. the threshold point is extrapolating

"""

import math, os, tempfile, subprocess
from scipy.optimize import curve_fit
from scipy.stats import linregress
import numpy as np
import matplotlib.pyplot as plt

import subprocess, sys
qec_playground_root_dir = subprocess.run("git rev-parse --show-toplevel", cwd=os.path.dirname(os.path.abspath(__file__)), shell=True, check=True, capture_output=True).stdout.decode(sys.stdout.encoding).strip(" \r\n")
rust_dir = qec_playground_root_dir

example_default_rough_runtime_budget = (3000, 60)  # runtime_budget can be any format, here example is (min error case, max time)
example_default_runtime_budget = (18000, 3600)  # 1 hours or 18000 samples
# p_graph is used to build the decoding graph: to have a better accuracy when evaluating threshold, we need to fix the decoding graph
def example_simulate_func(p, d, runtime_budget, p_graph=None):
    min_error_case, time_budget = runtime_budget
    A = 0.3  # logical error rate at threshold is 30%
    B = 1  # does not scale
    C = -0.3  # add some non-linearty around the threshold
    x = (p - 0.10261) * (d ** (1 / 1.46))
    logical_error_rate = A + B * x + C * (x**2)
    if logical_error_rate > 0.75:
        logical_error_rate = 0.75
    if logical_error_rate < 0.01:
        logical_error_rate = 0.01
    relative_error = math.sqrt(1. / min_error_case)
    logical_error_rate = logical_error_rate * (1 + np.random.normal(scale=relative_error / 2))
    print(f"[fake] simulating d={d}, min_error_case={min_error_case}, time_budget={time_budget}, pl={logical_error_rate}, stddev={relative_error}")
    return (logical_error_rate, relative_error)

QEC_PLAYGROUND_COMPILATION_DONE = False
if 'MANUALLY_COMPILE_QEC' in os.environ and os.environ["MANUALLY_COMPILE_QEC"] == "TRUE":
    QEC_PLAYGROUND_COMPILATION_DONE = True
def compile_code_if_necessary(additional_build_parameters=None):
    global QEC_PLAYGROUND_COMPILATION_DONE
    if QEC_PLAYGROUND_COMPILATION_DONE is False:
        build_parameters = ["cargo", "build", "--release"]
        if additional_build_parameters is not None:
            build_parameters += additional_build_parameters
        # print(build_parameters)
        process = subprocess.Popen(build_parameters, universal_newlines=True, stdout=sys.stdout, stderr=sys.stderr, cwd=rust_dir)
        process.wait()
        assert process.returncode == 0, "compile has error"
        QEC_PLAYGROUND_COMPILATION_DONE = True

def run_qecp_command_get_stdout(command, no_stdout=False, use_tmp_out=False, stderr_to_stdout=False):
    print("\n[command]", command)
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
    process = subprocess.Popen(command, universal_newlines=True, env=env, stdout=stdout, stderr=(stdout if stderr_to_stdout else sys.stderr), bufsize=10000000)
    stdout, _ = process.communicate()
    if use_tmp_out:
        out_file.flush()
        out_file.close()
        with open(out_filename, "r", encoding="utf8") as f:
            stdout = f.read()
        os.remove(out_filename)
    return stdout, process.returncode

def qecp_benchmark_simulate_func_command_vec(p, di, dj, T, parameters, max_repeats=1000000000, min_error_cases=3000, rust_dir=rust_dir, time_budget=None, p_graph=None):
    p_str = f"[{p:.8e}]"
    di_str = f"[{str(di)}]"
    dj_str = f"[{str(dj)}]"
    T_str = f"[{str(T)}]"
    qecp_path = os.path.join(rust_dir, "target", "release", "qecp-cli")
    command = [qecp_path, "tool", "benchmark", di_str, "--djs", dj_str, T_str, f"-m{max_repeats}", f"-e{min_error_cases}", p_str] + parameters
    if time_budget is not None:
        command += ["--time-budget", f"{time_budget}"]
    if p_graph is not None:
        command += ["--ps-graph", f"[{p_graph:.8e}]"]
    return command

# example of how to wrap qecp_benchmark_simulate_func_basic: CSS surface code with single round of perfect measurement
def example_qecp_benchmark_simulate_func(p, d, runtime_budget, p_graph=None):
    min_error_case, time_budget = runtime_budget
    parameters = f"-p0 --decoder mwpm --decoder_config {{\"pcmg\":true}}".split(" ")
    command = qecp_benchmark_simulate_func_command_vec(p, d, d, 0, parameters, min_error_cases=min_error_case, time_budget=time_budget, p_graph=p_graph)
    stdout, returncode = run_qecp_command_get_stdout(command)
    assert returncode == 0, "command fails..."
    full_result = stdout.strip(" \r\n").split("\n")[-1]
    lst = full_result.split(" ")
    error_rate = float(lst[5])
    confidence_interval = float(lst[7])
    return (error_rate, confidence_interval)


def example_main():
    USE_REAL_SIMULATION = False
    simulate_func = example_qecp_benchmark_simulate_func if USE_REAL_SIMULATION else example_simulate_func
    code_distances = [13, 15, 17] if USE_REAL_SIMULATION else [13, 15, 17, 19, 21]
    threshold_analyzer = ThresholdAnalyzer(code_distances, simulate_func
        , default_rough_runtime_budget=example_default_rough_runtime_budget, default_runtime_budget=example_default_runtime_budget)
    threshold_analyzer.verbose = True
    threshold_analyzer.estimate(save_image=os.path.join(os.path.dirname(__file__), f"example.pdf"))


class ThresholdAnalyzer:
    LOWEST_THRESHOLD = 0.0001  # any valid threshold should not be less than this

    # `simulate_func``: a function that takes (p, d, runtime_budget) as parameter and return (logical_error_rate, relative_error)
    def __init__(self, code_distances, simulate_func, default_rough_runtime_budget=None, default_runtime_budget=None):
        # default parameters that can be changed later
        self.rough_code_distances = [5, 7]
        self.rough_runtime_budgets = None  # must be set if called rough estimate
        if default_rough_runtime_budget is not None:
            self.rough_runtime_budgets = [default_rough_runtime_budget] * len(self.rough_code_distances)
        self.rough_init_search_start_p = 0.45  # rough searching is unable to handle threshold >= 45%; disable rough searching in these cases
        self.rough_init_search_step = 0.8  # to find the initial range, starting from p=0.5, [p, p*s, p*s*s, ...]
        self.rough_init_search_target_relative_diff = 0.1  # the logical error rate should differ by 10% for two code distances
        self.rough_fit_samples = 5  # use 5 different p for each code distance
        self.verbose = False
        self.runtime_budgets = None
        self.target_relative_diff = 0.1  # the logical error rate should differ by 10% for the smallest and largest code distances
        self.fit_samples = 9  # use 9 different p for precise estimation
        if default_runtime_budget is not None:
            self.runtime_budgets = [default_runtime_budget] * len(code_distances)
        # required parameters
        assert len(code_distances) >= 2
        self.code_distances = code_distances
        self.simulate_func = simulate_func
        # record results for later usage
        self.collected_data_list = []
        # bounds
        def bounds(y_data, p_list):
            p_range = np.max(p_list) - np.min(p_list)
            lower_bounds = [np.min(y_data), -np.inf, -100, np.min(p_list) - p_range, 0.5]
            upper_bounds = [np.max(y_data), np.inf, 100, np.max(p_list) + p_range, 3]
            return lower_bounds, upper_bounds
        self.bounds = bounds

    def prepare_parameters(self):
        self.rough_code_distances = list(dict.fromkeys(self.rough_code_distances))
        self.rough_code_distances.sort()
        self.code_distances = list(dict.fromkeys(self.code_distances))
        self.code_distances.sort()

    # return (init_center, init_radius) where the init searching center is the middle point, and two end points are bounded by rough_init_search_target_relative_diff
    def rough_estimate_init_searching(self):
        if self.verbose:
            print("[info] rough estimate init searching")
        d_low, d_high = self.rough_code_distances[0], self.rough_code_distances[-1]
        tb_low, tb_high = self.rough_runtime_budgets[0], self.rough_runtime_budgets[-1]
        p = self.rough_init_search_start_p
        # first find a valid p where pl_low is higher than pl_high more than their relative difference
        while p > ThresholdAnalyzer.LOWEST_THRESHOLD:  # threshold should be 
            pl_low, dev_low = self.simulate_func(p, d_low, tb_low)
            pl_high, dev_high = self.simulate_func(p, d_high, tb_high)
            if pl_low * (1 + dev_low) < pl_high * (1 - dev_high):  # smaller with confidence
                break
            if pl_low * (1 - dev_low) > pl_high * (1 + dev_high):  # greater with confidence
                print("unable to find a position where small code distance has lower logical error rate (should happen above threshold)")
                # exit(1)
            p = p * self.rough_init_search_step
        assert p > ThresholdAnalyzer.LOWEST_THRESHOLD, "p too small yet still cannot find a point above the threshold"
        last_p = p
        last_pl_low = pl_low
        last_pl_high = pl_high
        p = p * self.rough_init_search_step
        while p > ThresholdAnalyzer.LOWEST_THRESHOLD:  # threshold should be 
            pl_low, dev_low = self.simulate_func(p, d_low, tb_low)
            pl_high, dev_high = self.simulate_func(p, d_high, tb_high)
            if pl_low * (1 + dev_low) < pl_high * (1 - dev_high):  # smaller with confidence
                last_p = p
                last_pl_low = pl_low
                last_pl_high = pl_high
                p = p * self.rough_init_search_step
                continue
            if pl_low * (1 - dev_low) > pl_high * (1 + dev_high):  # greater with confidence
                break
            p = p * self.rough_init_search_step
        assert p > ThresholdAnalyzer.LOWEST_THRESHOLD, "p too small yet still cannot find a possible threshold"
        if self.verbose:
            print(f"[info] pl_low={pl_low}, last_pl_low={last_pl_low}, pl_high={pl_high}, last_pl_high={last_pl_high}, last_p={last_p}, p={p}")
        # do a simple linear curve fit
        x_data = [last_p, p]
        y_data_low = [last_pl_low, pl_low]
        y_data_high = [last_pl_high, pl_high]
        slope_low, intercept_low, _, _, _ = linregress(x_data, y_data_low)
        slope_high, intercept_high, _, _, _ = linregress(x_data, y_data_high)
        if self.verbose:
            print(f"[info] slope_low={slope_low}, intercept_low={intercept_low}, slope_high={slope_high}, intercept_high={intercept_high}")
        # calculate intersection point of these two lines
        init_center = (intercept_high - intercept_low) / (slope_low - slope_high)
        y_center = init_center * slope_low + intercept_low
        # calculate the radius that is necessary to reach rough_init_search_target_relative_diff
        init_radius = y_center * self.rough_init_search_target_relative_diff / abs(slope_low - slope_high)
        if self.verbose:
            print(f"[info] init_center={init_center}, init_radius={init_radius} (p = [{init_center - init_radius}, {init_center + init_radius}])")
            # print(intercept_low + slope_low * (init_center - init_radius), intercept_high + slope_high * (init_center - init_radius))
            # print(intercept_low + slope_low * (init_center + init_radius), intercept_high + slope_high * (init_center + init_radius))
        return init_center, init_radius
    
    @staticmethod
    def quadratic_approx_curve(p_d_pair, A, B, C, pc0, v0):
        y_list = []
        for p, d in p_d_pair:
            x = (p - pc0) * (d ** (1. / v0))
            y = A + B * x + C * (x ** 2)
            y_list.append(y)
        return y_list

    def fit_results(self, collected_data, p_list, code_distances):
        if self.verbose:
            print(f"[info] fitting collected data:")
            for j, p in enumerate(p_list):
                print(f"    p={p}")
                for i, d in enumerate(code_distances):
                    pl, dev = collected_data[i][j]
                    print(f"        d={d}, pl={pl}, dev={dev}")
            print(f"[dump] collected_data = {collected_data}")
        x_data = []
        y_data = []
        sigma = []
        for i, d in enumerate(code_distances):
            for j, p in enumerate(p_list):
                x_data.append((p, d))
                pl, dev = collected_data[i][j]
                y_data.append(pl)
                sigma.append(dev)  # relative sigma
        guess_A = np.average(y_data)
        guess_pc0 = np.average(p_list)
        popt, pcov = curve_fit(ThresholdAnalyzer.quadratic_approx_curve, x_data, y_data, sigma=sigma, absolute_sigma=False, p0=[guess_A, 1, 0.1, guess_pc0, 1]
            , bounds=self.bounds(y_data, p_list))
        perr = np.sqrt(np.diag(pcov))
        if self.verbose:
            print(f"[info] fit result: A = {popt[0]} \u00B1 {perr[0]}, B = {popt[1]} \u00B1 {perr[1]} C = {popt[2]} \u00B1 {perr[2]}")
            print(f"                   pc0 = {popt[3]} \u00B1 {perr[3]}, v0 = {popt[4]} \u00B1 {perr[4]}")
            print(f"    popt: {list(popt)}")
            print(f"    perr: {list(perr)}")
        return popt, perr  # [A, B, C, pc0, v0]

    def rough_estimate(self):
        self.prepare_parameters()
        assert self.rough_runtime_budgets is not None, "self.rough_runtime_budget not provided"
        assert len(self.rough_runtime_budgets) == len(self.rough_code_distances), "self.rough_runtime_budget must corresponds to each code distance"
        assert len(self.rough_code_distances) >= 2
        # do initial searching, to fix a small region as the center
        init_center, init_radius = self.rough_estimate_init_searching()
        # generate the list of p to be simulated
        p_list = [init_center - init_radius + init_radius * 2 * i / (self.rough_fit_samples - 1) for i in range(self.rough_fit_samples)]
        if self.verbose:
            print(f"[info] p_list={p_list}")
        # collect all data from simulation
        collected_data = []
        for i, d in enumerate(self.rough_code_distances):
            collected_data_row = []
            for p in p_list:
                pl, dev = self.simulate_func(p, d, self.rough_runtime_budgets[i], p_graph=init_center)  # use this center point to build decoding graph
                collected_data_row.append((pl, dev))
            collected_data.append(collected_data_row)
        self.collected_data_list.append((self.rough_code_distances, p_list, collected_data))
        # fit into the curve
        popt, perr = self.fit_results(collected_data, p_list, self.rough_code_distances)
        return popt.tolist(), perr.tolist()
    
    def precise_estimate_parameters(self, rough_popt):
        self.prepare_parameters()
        assert self.runtime_budgets is not None, "self.runtime_budget not provided"
        assert len(self.runtime_budgets) == len(self.code_distances), "self.runtime_budget must corresponds to each code distance"
        assert len(self.code_distances) >= 2
        # calculate a proper radius
        d_low, d_high = self.code_distances[0], self.code_distances[-1]
        p_center = rough_popt[3]
        pl_center = rough_popt[0]
        rough_v0 = rough_popt[4]
        radius = pl_center * self.target_relative_diff / abs((d_low ** (1. / rough_v0)) - (d_high ** (1. / rough_v0)))
        p_list = [p_center - radius + radius * 2 * i / (self.fit_samples - 1) for i in range(self.fit_samples)]
        if self.verbose:
            print(f"[info] p_list={p_list}")
        # collect all data from simulation
        collected_parameters = []
        for i, d in enumerate(self.code_distances):
            collected_parameters_row = []
            for p in p_list:
                collected_parameters_row.append((p, d, self.runtime_budgets[i], p_center))  # use this center point to build decoding graph
            collected_parameters.append(collected_parameters_row)
        return collected_parameters, p_list

    def precise_estimate(self, rough_popt):
        if self.verbose:
            print(f"[info] precise estimate given rough starting point: {rough_popt[3]}")
        # collect all data from simulation
        collected_parameters, p_list = self.precise_estimate_parameters(rough_popt)
        collected_data = []
        for i, collected_parameters_row in enumerate(collected_parameters):
            collected_data_row = []
            for j, (p, d, runtime_budget, p_center) in enumerate(collected_parameters_row):
                pl, dev = self.simulate_func(p, d, runtime_budget, p_center)
                collected_data_row.append((pl, dev))
            collected_data.append(collected_data_row)
        self.collected_data_list.append((self.code_distances, p_list, collected_data))
        # fit into the curve
        popt, perr = self.fit_results(collected_data, p_list, self.code_distances)
        return popt.tolist(), perr.tolist()
    
    def save_image_collected_data(self, image_path, popt, perr, code_distances, p_list, collected_data):
        fig = plt.gcf()
        fig.set_size_inches(8, 6)
        # draw the simulated data
        for i, d in enumerate(code_distances):
            yerr_lower = [pl * (1 - 1 / (1 + dev)) for pl, dev in collected_data[i]]
            yerr_upper = [pl * dev for pl, dev in collected_data[i]]
            plt.errorbar(p_list, [pl for pl, dev in collected_data[i]], fmt='.', label = f"simulated d = {d}"
                , yerr=(yerr_lower, yerr_upper), color=f"C{i}")
        # draw the fitted data
        p_lower, p_higher = np.min(p_list), np.max(p_list)
        N = 100
        p_denser = [p_lower + (p_higher - p_lower) * i / (N-1) for i in range(N)]
        for i, d in enumerate(code_distances):
            fitted_list = ThresholdAnalyzer.quadratic_approx_curve([(p, d) for p in p_denser], *popt)
            plt.plot(p_denser, fitted_list, '-', label = f"fitted d = {d}", color=f"C{i}")
        plt.legend()
        plt.xlabel("physical error rate (p)")
        plt.ylabel("logical error rate (pL)")
        plt.title(f"Threshold = {popt[3]:.5g} \u00B1 {perr[3]:.2g}")
        # plt.show()
        plt.savefig(image_path)
        fig.clear()

    # first call rough estimate then precise estimate
    def estimate(self, target_relative_accuracy=0.01, save_image=None, retry=True):
        rough_popt, rough_perr = self.rough_estimate()
        popt, perr = self.precise_estimate(rough_popt)
        pl, pl_err = popt[3], perr[3]
        if retry and pl_err / pl > target_relative_accuracy:
            # recalculate using the new popt
            print("[warning] single attempt to precisely estimate the threshold fails, retry")
            popt, perr = self.precise_estimate(popt)
        # plot the fitted result as an image
        if save_image is not None:
            self.save_image_collected_data(save_image, popt, perr, *self.collected_data_list[-1])
    
    # given existing data, do estimate
    def estimate_exiting(self, p_list, collected_data, save_image=None):
        # fit into the curve
        popt, perr = self.fit_results(collected_data, p_list, self.code_distances)
        popt, perr = popt.tolist(), perr.tolist()
        pl, pl_err = popt[3], perr[3]
        # plot the fitted result as an image
        if save_image is not None:
            self.save_image_collected_data(save_image, popt, perr, self.code_distances, p_list, collected_data)

if __name__ == "__main__":
    example_main()
