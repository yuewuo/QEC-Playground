"""
Prerequisite:
run `cargo build --release` under /backend/rust/ directory

Usage:
see `main`
"""

import numpy as np
import os, sys, math

"""
weights: see `output_weights_to_file` and `generate_default_weights_to_file`
p: physical qubit error rate, default to 0.01
min_error_cases: if reaches the minimum error cases, the error rate is returned immediately. speed up computation
max_N: the maximum cases to try before return. This is useful when logical error rate is too small to be found in reasonable time
parallel: use how many processes for parallel computing. <= 1 will automatically use os.cpu_count()-1 instead
"""
def compute_error_rate(weights, p=0.01, min_error_cases=1000, max_N=100000000, rust_qecp_path=None, rust_qecp_name="rust_qecp", parallel=1):
    if parallel < 1:
        parallel = os.cpu_count() - 1
    if rust_qecp_path is None:  # automatically find the rust_qecp binary
        rust_qecp_path = os.path.join(os.path.dirname(os.path.dirname(os.path.realpath(__file__))), "rust", "target", "release")
        rust_qecp_binary = os.path.join(rust_qecp_path, rust_qecp_name)
    d = sanity_check_weights_return_d(weights)
    weights_path = os.path.join(rust_qecp_path, "weights.txt")
    output_weights_to_file(weights, weights_path)
    max_N_each = math.ceil(max_N / parallel)
    min_error_cases_each = math.ceil(min_error_cases / parallel)
    cmd = rust_qecp_binary + " tool error_rate_MWPM_with_weight [%d] [%f] -m %d -e %d -w " % (d, p, max_N_each, min_error_cases_each) + weights_path
    # print("cmd:", cmd)
    runnings = []
    for i in range(parallel):
        r = os.popen(cmd)
        runnings.append(r)
    total_rounds = 0
    qec_failed = 0
    for r in runnings:
        text = r.read()
        elements = text.split(" ")
        assert int(elements[1]) == d, "strange output from rust_qecp"
        total_rounds += int(elements[2].strip())
        qec_failed += int(elements[3].strip())
    # print(total_rounds, qec_failed)
    return qec_failed / total_rounds

"""
weights should be 4 dimensional numpy array.
See `generate_default_weights_to_file` about how to generate weights
"""
def output_weights_to_file(weights, filename):
    d = sanity_check_weights_return_d(weights)
    with open(filename, "w", encoding="ascii") as f:
        f.write("%d\n" % d)
        for i1 in range(d+1):
            for j1 in range(d+1):
                for i2 in range(d+1):
                    for j2 in range(d+1):
                        f.write("%d %d %d %d %.16f\n" % (i1, j1, i2, j2, weights[i1][j1][i2][j2]))

def sanity_check_weights_return_d(weights):
    shape = weights.shape
    assert len(shape) == 4
    d1, d2, d3, d4 = shape
    assert d1 == d2 and d1 == d3 and d1 == d4
    d = d1 - 1
    assert d > 0
    return d

def generate_weights_from_function(d, func):
    weights = np.zeros((d + 1, d + 1, d + 1, d + 1), dtype=np.double)
    for i1 in range(d + 1):
        for j1 in range(d + 1):
            for i2 in range(d + 1):
                for j2 in range(d + 1):
                    weights[i1, j1, i2, j2] = func(i1, j1, i2, j2)
    return weights

def default_weights(i1, j1, i2, j2):
    def distance_delta(i, j):
        return (abs(i + j) + abs(i - j)) / 2.
    def distance(i1, j1, i2, j2):
        return distance_delta(i2 - i1, j2 - j1)
    return -distance(i1, j1, i2, j2)

def generate_default_weights_to_file(d, filename):
    weights = generate_weights_from_function(d, default_weights)
    output_weights_to_file(weights, filename)



if __name__ == "__main__":
    # generate correct default weights
    generate_default_weights_to_file(5, "../rust/default_weights.txt")
    # also generate a wrong one
    output_weights_to_file(generate_weights_from_function(5, lambda *x: -default_weights(*x)), "../rust/wrong_default_weights.txt")

    # test calling rust functions
    default_weights = generate_weights_from_function(5, default_weights)

    error_rate = compute_error_rate(default_weights, min_error_cases=1000, parallel=0)  # parallel=0 to use number of CPUs - 1 processes
    print("error_rate:", error_rate)
