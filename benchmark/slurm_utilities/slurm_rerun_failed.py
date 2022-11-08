import os, sys, subprocess, time
import slurm_distribute


def rerun_failed(sbatch_file_path, failed_cases, slurm_commands_vec=None, use_interactive_partition=False):
    # generate rerun sbatch file
    sbatch_file_folder = os.path.dirname(sbatch_file_path)
    rerun_file_path = os.path.join(sbatch_file_folder, "rerun-" + os.path.basename(sbatch_file_path))
    with open(sbatch_file_path, "r", encoding="utf8") as f:
        lines = f.readlines()
    with open(rerun_file_path, "w", encoding="utf8") as f:
        for line in lines:
            if line.startswith("#SBATCH --array="):
                f.write(f"#SBATCH --array={','.join([str(e) for e in failed_cases])}\n")
            else:
                f.write(line)
    print("rerun_file_path", rerun_file_path)
    slurm_distribute.slurm_run_sbatch_wait(rerun_file_path, failed_cases, original_sbatch_file_path=sbatch_file_path, slurm_commands_vec=slurm_commands_vec, use_interactive_partition=use_interactive_partition)

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("usage: <sbatch_file_path> <failed_cases: comma separated>")
        exit(-1)
    sbatch_file_path = os.path.abspath(sys.argv[1])
    failed_cases = [int(e) for e in sys.argv[2].split(",")]
    rerun_failed(sbatch_file_path, failed_cases)
