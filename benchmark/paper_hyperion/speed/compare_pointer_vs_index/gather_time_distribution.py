import os, sys, git, math
from tqdm import tqdm

qec_playground_root_dir = git.Repo(".", search_parent_directories=True).working_tree_dir
benchmark_dir = os.path.join(qec_playground_root_dir, "benchmark")
sys.path.insert(0, benchmark_dir)

from qecp_util import *
from common import *

this_dir = os.path.dirname(os.path.abspath(__file__))

for name in ["pointer", "index"]:

    distribution = TimeDistribution()

    for job_id in tqdm(range(split_job)):
        benchmark_profile_path = os.path.join(
            profile_folder, f"{name}_{job_id}.profile"
        )
        statistics = RuntimeStatistics(
            benchmark_profile_path,
            apply_entries=lambda entry: entry["elapsed"]["decode"],
        )

        for decoding_time in statistics.entries:
            distribution.record(decoding_time)

    average_latency = distribution.average_latency()
    print(f"average decoding time: {average_latency:.3e}s")

    with open(
        os.path.join(this_dir, f"{name}_{d}.distribution"), "w", encoding="utf8"
    ) as f:
        f.write(distribution.to_line())
