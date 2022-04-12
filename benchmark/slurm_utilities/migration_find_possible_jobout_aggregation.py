import os, sys, subprocess


qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
process = subprocess.run(["git", "ls-files"], universal_newlines=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, cwd=qec_playground_root_dir)
output = process.stdout
files = output.split("\n")
print(f"number of files in the git repo: {len(files)}")

possible_aggregation_folders = dict()
possible_aggregation_number = 0
for filename in files:
    if filename[-7:] == ".jobout":
        folder = "/".join(filename.split("/")[:-1])
        if folder in possible_aggregation_folders:
            possible_aggregation_folders[folder] += 1
        else:
            possible_aggregation_folders[folder] = 1
        possible_aggregation_number += 1

print(f"number of files that can be reduced by this aggregation: {possible_aggregation_number}")

"""
2022/4/12
number of files in the git repo: 6694
number of files that can be reduced by this aggregation: 5264

we can reduce the number of files in this git repo by more than 80%!
"""

for folder in possible_aggregation_folders:
    print(os.path.join(qec_playground_root_dir, folder), possible_aggregation_folders[folder])


# SLURM_USE_EXISTING_DATA=1 python3 ...

for folder in possible_aggregation_folders:
    if os.path.exists(os.path.join(qec_playground_root_dir, folder, "_aggregated"))


