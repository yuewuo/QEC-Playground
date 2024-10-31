import os, sys, git, math
from tqdm import tqdm
from concurrent.futures import ThreadPoolExecutor, as_completed

qec_playground_root_dir = git.Repo(".", search_parent_directories=True).working_tree_dir
benchmark_dir = os.path.join(qec_playground_root_dir, "benchmark")
sys.path.insert(0, benchmark_dir)

from qecp_util import *
from common import *

this_dir = os.path.dirname(os.path.abspath(__file__))

max_workers = 4

for config in configurations:
    print(config)

    # could be modularized, but for now, just copy and paste
    distribution = TimeDistribution()
    distribution_time_decode = TimeDistribution()
    distribution_mwpf = TimeDistribution()
    distribution_bp = TimeDistribution()

    def job(job_id: int, pbar):
        local_distribution = TimeDistribution()
        local_distribution_time_decode = TimeDistribution()
        local_distribution_mwpf = TimeDistribution()
        local_distribution_bp = TimeDistribution()

        benchmark_profile_path = os.path.join(
            profile_folder, f"{config.name}_{d}_{job_id}.profile"
        )
        statistics = RuntimeStatistics(
            benchmark_profile_path,
            apply_entries=lambda entry: (
                entry["elapsed"]["decode"],
                entry["time_decode"],
                entry["time_decode_mwpf"],
                entry["time_decode_bp"],
            ),
        )

        for entry in statistics.entries:
            local_distribution.record(entry[0])
            local_distribution_time_decode.record(entry[1])
            local_distribution_mwpf.record(entry[2])
            local_distribution_bp.record(entry[3])

        pbar.update(1)
        return (
            local_distribution,
            local_distribution_time_decode,
            local_distribution_mwpf,
            local_distribution_bp,
        )

    with tqdm(total=split_job) as pbar:
        with ThreadPoolExecutor(max_workers=max_workers) as ex:
            futures = [ex.submit(job, job_id, pbar) for job_id in range(split_job)]
            for future in as_completed(futures):
                (
                    local_distribution,
                    local_distribution_time_decode,
                    local_distribution_mwpf,
                    local_distribution_bp,
                ) = future.result()
                distribution += local_distribution
                distribution_time_decode += local_distribution_time_decode
                distribution_mwpf += local_distribution_mwpf
                distribution_bp += local_distribution_bp

    """ old single thread implementation, was too slow"""
    # for job_id in tqdm(range(split_job)):
    #     benchmark_profile_path = os.path.join(
    #         profile_folder, f"{config.name}_{d}_{job_id}.profile"
    #     )
    #     statistics = RuntimeStatistics(
    #         benchmark_profile_path,
    #         apply_entries=lambda entry: (
    #             entry["elapsed"]["decode"],
    #             entry["time_decode"],
    #             entry["time_decode_mwpf"],
    #             entry["time_decode_bp"],
    #         ),
    #     )

    #     for entry in statistics.entries:
    #         distribution.record(entry[0])
    #         distribution_time_decode.record(entry[1])
    #         distribution_mwpf.record(entry[2])
    #         distribution_bp.record(entry[3])

    average_latency = distribution.average_latency()
    print(f"average decoding time: {average_latency:.3e}s")
    average_latency_time_decode = distribution_time_decode.average_latency()
    print(f"average time_decode: {average_latency_time_decode:.3e}s")
    average_latency_mwpf = distribution_mwpf.average_latency()
    print(f"average time_decode_mwpf: {average_latency_mwpf:.3e}s")
    average_latency_bp = distribution_bp.average_latency()
    print(f"average time_decode_bp: {average_latency_bp:.3e}s")

    with open(
        os.path.join(this_dir, f"{config.name}_{d}.distribution"), "w", encoding="utf8"
    ) as f:
        f.write(distribution.to_line())

    with open(
        os.path.join(this_dir, f"{config.name}_{d}_time_decode.distribution"),
        "w",
        encoding="utf8",
    ) as f:
        f.write(distribution_time_decode.to_line())

    with open(
        os.path.join(this_dir, f"{config.name}_{d}_time_decode_mwpf.distribution"),
        "w",
        encoding="utf8",
    ) as f:
        f.write(distribution_mwpf.to_line())

    with open(
        os.path.join(this_dir, f"{config.name}_{d}_time_decode_bp.distribution"),
        "w",
        encoding="utf8",
    ) as f:
        f.write(distribution_bp.to_line())
