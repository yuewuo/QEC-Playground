import os, sys, git
import matplotlib.pyplot as plt

qec_playground_root_dir = git.Repo(".", search_parent_directories=True).working_tree_dir
benchmark_dir = os.path.join(qec_playground_root_dir, "benchmark")
sys.path.insert(0, benchmark_dir)

from qecp_util import *
from common import *

this_dir = os.path.dirname(os.path.abspath(__file__))

for config in configurations:

    dist_filename = os.path.join(this_dir, f"{config.name}_{d}.distribution")
    with open(dist_filename, "r", encoding="utf8") as f:
        distribution = TimeDistribution.from_line(f.read())

    pL_filename = os.path.join(this_dir, f"{config.name}_{d}.txt")
    # also print logical error rate
    pL = None
    confidence = None
    with open(pL_filename, "r", encoding="utf8") as f:
        for line in f.readlines():
            if line.startswith("#"):
                continue
            line = line.strip("\r\n ")
            spt = line.split(" ")
            assert len(spt) == 4
            pL = float(spt[2])
            confidence = float(spt[3])

    print(
        f"{config.name}: average decoding time: {distribution.average_latency():.3e}s, pL = {pL:.3e} (confidence = {confidence:.2e})"
    )

    x_vec, y_vec = distribution.flatten()

    plt.cla()
    plt.loglog(x_vec, y_vec, ".-")
    plt.xlim(1e-7, 10)
    plt.ylim(0.5, 1e9)
    plt.ylabel("Sample Count")
    plt.xlabel("Decoding Latency (s)")
    plt.savefig(f"{config.name}_{d}.pdf")
