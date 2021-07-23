import sys, os, random, math
from scipy.optimize import leastsq, fsolve
import numpy as np

class DataPoint:
    def __init__(self, p, di, T, total_rounds, qec_failed, error_rate, dj, confidence_interval_95_percent):
        self.p = float(p)
        self.di = int(di)
        self.dj = int(dj)
        self.T = int(T)
        self.total_rounds = int(total_rounds)
        self.qec_failed = int(qec_failed)
        self.error_rate = float(error_rate)
        self.confidence_interval_95_percent = float(confidence_interval_95_percent)
    @classmethod
    def from_str(cls, string):
        string = string.strip(" \r\n")
        if string == "":
            return None
        lst = string.split(" ")
        if len(lst) < 8:
            print("unrecognized data line: '%s'" % string)
            print("format should be <p> <di> <T> <total_rounds> <qec_failed> <error_rate> <dj> <confidence_interval_95_percent> ...")
            return None
        return cls(lst[0], lst[1], lst[2], lst[3], lst[4], lst[5], lst[6], lst[7])
    def __repr__(self):
        return f"DataPoint({self.di}, {self.dj}, {self.T}, {self.p}, {self.error_rate}, {self.confidence_interval_95_percent})"
    def generate_random_logical_error_rate(self):
        stddev = self.error_rate * self.confidence_interval_95_percent / 1.96
        rst = 0
        while rst <= 0 or rst >= 1:
            rst = random.gauss(self.error_rate, stddev)
        return rst  # make sure to return a reasonable logical error rate

def read_data_from_file(filepath):
    data = []
    with open(filepath, "r", encoding="utf-8") as f:
        lines = f.readlines()
        for line in lines:
            data_point = DataPoint.from_str(line)
            if data_point is not None:
                data.append(data_point)
    return data

def filter_data_with_pth_low_high(data, pth_low, pth_high):
    return [data_point for data_point in data if data_point.p >= pth_low and data_point.p <= pth_high]

def func_sq(params, x):
    A, B, C = params
    return A * x * x + B * x + C

def error_sq(params, x, y):
    return func_sq(params, x) - y

def solve_fit_function_sq(random_data):
    X = np.array([math.log(e[0]) for e in random_data])
    Y = np.array([math.log(e[1]) for e in random_data])
    p0 = [0, 0, 0]
    para = leastsq(error_sq, p0, args=(X, Y))
    A, B, C = para[0]
    return A, B, C

def measure_intersection_point(data1, data2, pth_estimate, random_count = 1000):
    ln_pth_vec = []
    for i in range(random_count):
        random_data1 = [(data_point.p, data_point.generate_random_logical_error_rate()) for data_point in data1]
        random_data2 = [(data_point.p, data_point.generate_random_logical_error_rate()) for data_point in data1]
        A1, B1, C1 = solve_fit_function_sq(random_data1)
        A2, B2, C2 = solve_fit_function_sq(random_data2)
        A = A1 - A2
        B = B1 - B2
        C = C1 - C2
        B2_4AC = B * B - 4 * A * C
        if B2_4AC < 0:
            continue  # just try next round
        lnp1 = (- B + math.sqrt(B2_4AC)) / (2 * A)
        lnp2 = (- B - math.sqrt(B2_4AC)) / (2 * A)
        lnp1_satisfied = lnp1 < 0
        lnp2_satisfied = lnp2 < 0
        if not lnp1_satisfied and not lnp2_satisfied:
            continue
        lnp = lnp1 if lnp1_satisfied else lnp2
        if lnp1_satisfied and lnp2_satisfied:  # both satisfy the requirement
            lnp = lnp1 if abs(lnp1 - math.log(pth_estimate)) < abs(lnp2 - math.log(pth_estimate)) else lnp2
        ln_pth_vec.append(lnp)
    ln_pth = np.mean(ln_pth_vec)
    pth = math.exp(ln_pth)
    std_over_mean = np.std(ln_pth_vec)  # e^ε = 1 + ε when ε << 1
    return pth, std_over_mean

if __name__ == "__main__":
    if len(sys.argv) != 5:
        print("usage: python3 ./estimate_threshold.py <datapath1> <datapath2> <pth_estimate> <pth_half_interval>")
        print("    it will only use data points in the range of [pth_estimate - pth_half_interval, pth_estimate + pth_half_interval]")
        print("    the data file should be generated by `fault_tolerant_benchmark` tool or have the same format")
        print("         format: <p> <di> <T> <total_rounds> <qec_failed> <error_rate> <dj> <confidence_interval_95_percent>")
        print("         an example of this is `0.0035 4 12 100500 17037 0.1695223880597015 12 1.4e-2`")
        exit(0)
    datapath1 = sys.argv[1]
    datapath2 = sys.argv[2]
    pth_estimate = float(sys.argv[3])
    pth_half_interval = float(sys.argv[4])
    pth_low = pth_estimate - pth_half_interval
    pth_high = pth_estimate + pth_half_interval
    data1 = filter_data_with_pth_low_high(read_data_from_file(datapath1), pth_low, pth_high)
    data2 = filter_data_with_pth_low_high(read_data_from_file(datapath2), pth_low, pth_high)
    pth, std_over_mean = measure_intersection_point(data1, data2, pth_estimate)
    print("pth, std_over_mean")
    print(pth, std_over_mean)