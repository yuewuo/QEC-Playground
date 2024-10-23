import os, sys, git
import matplotlib.pyplot as plt
import numpy as np

qec_playground_root_dir = git.Repo(".", search_parent_directories=True).working_tree_dir
benchmark_dir = os.path.join(qec_playground_root_dir, "benchmark")
sys.path.insert(0, benchmark_dir)

from qecp_util import *
from common import *

this_dir = os.path.dirname(os.path.abspath(__file__))

suffixes = ["", "_time_decode", "_time_decode_mwpf", "_time_decode_bp"]

for config in configurations:

    for suffix in suffixes:

        dist_filename = os.path.join(this_dir, f"{config.name}_{d}{suffix}.distribution")
        with open(dist_filename, "r", encoding="utf8") as f:
            distribution = TimeDistribution.from_line(f.read())

        x_vec, y_vec = distribution.flatten()
        # Calculate the cumulative sum of y_vec to get the CDF
        y_cdf = np.cumsum(y_vec) / np.sum(y_vec)

        plt.cla()  # Clear the plot
        plt.loglog(x_vec, y_cdf, ".-")
        plt.xlim(1e-7, 10)
        plt.ylim(0.01, 1)
        plt.ylabel("CDF")
        plt.xlabel("Decoding Latency (s)")
        plt.savefig(f"{config.name}_{d}{suffix}_cdf.pdf")