import os, sys, subprocess, time
import slurm_distribute


def try_collect_missing_jobs(sbatch_file_path):
    # generate rerun sbatch file
    job_count = slurm_distribute.get_job_count_from_sbatch(sbatch_file_path)
    assert job_count is not None, "sbatch file must contains #SBATCH --array=0-<job_count-1>"
    
    # iterate job outputs
    sbatch_file_folder = os.path.dirname(sbatch_file_path)
    missing_indices = []
    missing_or_empty_indices = []
    for idx in range(job_count):
        job_out_filepath = os.path.join(sbatch_file_folder, f"{idx}.jobout")
        if os.path.exists(job_out_filepath):
            with open(job_out_filepath, "r", encoding="utf8") as f:
                content = f.read()
                if content == "":
                    missing_or_empty_indices.append(idx)
                else:
                    lines = content.split("\n")
                    if lines[0].startswith("format:") and (len(lines) < 1 or lines[1] == ""):
                        missing_or_empty_indices.append(idx)
        else:
            missing_indices.append(idx)
            missing_or_empty_indices.append(idx)
    return missing_indices, missing_or_empty_indices

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: <sbatch_file_path>")
        print("output: comma separated missing outputs")
        exit(-1)
    sbatch_file_path = os.path.abspath(sys.argv[1])
    missing_indices, missing_or_empty_indices = try_collect_missing_jobs(sbatch_file_path)
    print(f"{len(missing_indices)} missing indices: {','.join([str(e) for e in missing_indices])}")
    print(f"{len(missing_or_empty_indices)} missing or empty indices: {','.join([str(e) for e in missing_or_empty_indices])}")
    print(f"tip: rerun them: ./slurm_rerun_failed.py <sbatch_file_path> <failed_cases: comma separated>")
