"""
common utilities for profiling
"""

import os, sys, subprocess, tempfile, time, toml, shutil

qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
data_dir = os.path.join(os.path.dirname(__file__), f"data")

def timed_run_command(command, no_stdout=False, use_tmp_out=False, stderr_to_stdout=False):
    env = os.environ.copy()
    env["RUST_BACKTRACE"] = "full"
    stdout = subprocess.PIPE
    if use_tmp_out:
        out_file = tempfile.NamedTemporaryFile(delete=False)
        out_filename = out_file.name
        stdout = out_file
    if no_stdout:
        stdout = sys.stdout
    start_time = time.time()
    process = subprocess.Popen(command, universal_newlines=True, env=env, stdout=stdout, stderr=(stdout if stderr_to_stdout else sys.stderr), bufsize=100000000)
    stdout, _ = process.communicate()
    end_time = time.time()
    if use_tmp_out:
        out_file.flush()
        out_file.close()
        with open(out_filename, "r", encoding="utf8") as f:
            stdout = f.read()
        os.remove(out_filename)
    return stdout, process.returncode, end_time - start_time

def run_profile_command(name, data_folder, command, verbose=True):
    if verbose:
        print(f"[profile:{name}] running \"{' '.join(command)}\"")
    if not os.path.exists(data_folder):
        os.makedirs(data_folder, exist_ok=False)
    data_path = os.path.join(data_folder, f"{name}.toml")
    data = dict()
    data["create_time"] = time.strftime('%Y-%m-%d %H:%M:%S')
    stdout, returncode, time_consumption = timed_run_command(command)
    assert returncode == 0, "command failed..."
    splitted_stdout = [line.strip("\r\n") for line in stdout.split("\n")]
    data["time_consumption"] = time_consumption
    data["stdout"] = splitted_stdout
    with open(data_path, "w", encoding="utf8") as f:
        f.write(toml.dumps(data))

AUTO_PROFILER_DELETE_FOLDER = False
if 'AUTO_PROFILER_DELETE_FOLDER' in os.environ and os.environ["AUTO_PROFILER_DELETE_FOLDER"] != "":
    AUTO_PROFILER_DELETE_FOLDER = True
AUTO_PROFILER_KEEP_FOLDER = False
if 'AUTO_PROFILER_KEEP_FOLDER' in os.environ and os.environ["AUTO_PROFILER_KEEP_FOLDER"] != "":
    AUTO_PROFILER_KEEP_FOLDER = True
def confirm_action_to_data_folder(data_folder):
    assert not (AUTO_PROFILER_KEEP_FOLDER and AUTO_PROFILER_DELETE_FOLDER), "these two options conflict"
    if not AUTO_PROFILER_KEEP_FOLDER:
        if os.path.exists(data_folder):
            if AUTO_PROFILER_DELETE_FOLDER:
                answer = "D"
            else:
                print(f"[info] set \"AUTO_PROFILER_DELETE_FOLDER=1\" to automatically delete folder if exists")
                print(f"[info] set \"AUTO_PROFILER_KEEP_FOLDER=1\" to automatically keep folder as is if exists")
                answer = input(f"[warning] {data_folder} already exists, action? [D/DELETE/K/KEEP/E/EXIT]")
            if answer.upper() in ["D", "DELETE"]:
                shutil.rmtree(data_folder)
            elif answer.upper() in ["K", "KEEP"]:
                pass
            else:
                print(f"profile canceled due to existing data folder: {data_folder}")
                exit(0)

