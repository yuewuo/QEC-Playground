import os, sys, subprocess, shutil, shlex, time, stat
from datetime import datetime


DEBUG_USING_INTERACTIVE_PARTITION = False  # only enable while debugging

# to check which node causes problem, run `sacct -j <JOBID> | grep FAILED`
NODE_BLACK_LIST = ["p08r07n[01-08]", "p09r11n25"]  # these nodes fails

# utility tool
ONLY_PRINT_COMMANDS = False
if 'ONLY_PRINT_COMMANDS' in os.environ and os.environ["ONLY_PRINT_COMMANDS"] == "TRUE":
    ONLY_PRINT_COMMANDS = True

# check for slurm flags in environment
SLURM_DISTRIBUTE_ENABLED = False
SLURM_USE_EXISTING_DATA = False
if 'SLURM_USE_EXISTING_DATA' in os.environ and os.environ["SLURM_USE_EXISTING_DATA"] == "TRUE":
    SLURM_USE_EXISTING_DATA = True
    SLURM_DISTRIBUTE_ENABLED = True  # always use slurm workflow
if 'SLURM_DISTRIBUTE_ENABLED' in os.environ and os.environ["SLURM_DISTRIBUTE_ENABLED"] == "TRUE":
    SLURM_DISTRIBUTE_ENABLED = True

if SLURM_DISTRIBUTE_ENABLED:
    SLURM_DISTRIBUTE_ENABLED = True
    SLURM_DISTRIBUTE_CPUS_PER_TASK = 36
    SLURM_DISTRIBUTE_MEM_PER_TASK = '8G'  # do not use too much memory, otherwise the task will probably fail with exit code = 1
    SLURM_DISTRIBUTE_MAX_JOB = 25  # 1000 CPUs per person
    SLURM_DISTRIBUTE_TIME = "1-00:00:00"

def slurm_threads_or(default_threads):
    if SLURM_DISTRIBUTE_ENABLED:
        return SLURM_DISTRIBUTE_CPUS_PER_TASK
    return default_threads

def confirm_or_die(action=""):
    while True:
        answer = input(f"[{action}] Continue? [Y/YES/N/NO]")
        if answer.upper() in ["Y", "YES"]:
            return
        elif answer.upper() in ["N", "NO"]:
            raise "confirmation failed, exit"

def slurm_distribute_wrap(program):
    def wrapper():
        if ONLY_PRINT_COMMANDS or SLURM_DISTRIBUTE_ENABLED:
            # first gether all commands
            slurm_commands_vec = []
            def error_run_command_get_stdout(command):
                print(f"should not call `run_command_get_stdout` here, command: ${command}")
                raise "should not call `run_command_get_stdout` here"
            program(slurm_commands_vec=slurm_commands_vec, run_command_get_stdout=error_run_command_get_stdout)
            stringify_commands = []
            def run_stringify_command(command):
                return f"{' '.join([shlex.quote(e) for e in command])}"
            for idx, command in enumerate(slurm_commands_vec):
                stringify_command = run_stringify_command(command)
                stringify_commands.append(stringify_command)
                if ONLY_PRINT_COMMANDS:
                    print(stringify_command)
            if ONLY_PRINT_COMMANDS:
                return None  # terminate the program

        if not SLURM_DISTRIBUTE_ENABLED:
            return program()
        else:
            slurm_jobs_folder = os.path.join(os.path.abspath(os.getcwd()), "slurm_jobs")
            if not SLURM_USE_EXISTING_DATA:
                # print out for confirmation
                print("commands:")
                for idx, command in enumerate(slurm_commands_vec):
                    print(f"{idx}. {command}")
                print("parameters:")
                job_name = datetime.now().strftime("QEC-Playground=%m-%d-%Y=%H:%M:%S")
                if DEBUG_USING_INTERACTIVE_PARTITION:
                    job_count = 1
                else:
                    job_count = len(slurm_commands_vec)
                confirm_or_die(f"clear content in {slurm_jobs_folder}")
                if os.path.exists(slurm_jobs_folder):
                    shutil.rmtree(slurm_jobs_folder)
                os.makedirs(slurm_jobs_folder)
                parameters = [f"--job-name={job_name}", f"--time={SLURM_DISTRIBUTE_TIME}", f"--mem={SLURM_DISTRIBUTE_MEM_PER_TASK}", "--mail-type=ALL", "--nodes=1", "--ntasks=1"
                    , f"--cpus-per-task={SLURM_DISTRIBUTE_CPUS_PER_TASK}", f"--array=0-{job_count-1}", f'--out="{os.path.join(slurm_jobs_folder, "slurm-%a.out")}"'
                    , f'--error="{os.path.join(slurm_jobs_folder, "slurm-%a.err")}"']
                if len(NODE_BLACK_LIST) > 0:
                    parameters.append(f"--exclude={','.join(NODE_BLACK_LIST)}")
                for parameter in parameters:
                    print(f"    {parameter}")
                job_script_sbatch_path = os.path.join(slurm_jobs_folder, f"job_script.sbatch")
                job_script_sbatch_content = ""
                job_script_sbatch_content += f"#!/bin/bash\n"
                for parameter in parameters:
                    job_script_sbatch_content += f"#SBATCH {parameter}\n"
                job_script_sbatch_content += "\n"
                ERRCODE = 91
                stringify_command_set = {}
                for idx, command in enumerate(slurm_commands_vec):
                    stringify_command = run_stringify_command(command)
                    # sanity check of the commands
                    if stringify_command in stringify_command_set:
                        confirm_or_die(f"duplicate command from {stringify_command_set[stringify_command]} and {idx}: {stringify_command}")
                    stringify_command_set[stringify_command] = idx
                if DEBUG_USING_INTERACTIVE_PARTITION:
                    job_script_sbatch_content += f'if [ "$SLURM_ARRAY_TASK_ID" == "0" ];\n'
                    job_script_sbatch_content += f'then\n'
                    for idx, command in enumerate(slurm_commands_vec):
                        job_script_sbatch_content += f'    {stringify_commands[idx]} > {os.path.join(slurm_jobs_folder, f"{idx}.jobout")} || exit {ERRCODE};\n'
                    job_script_sbatch_content += f'fi\n'
                else:
                    for idx, command in enumerate(slurm_commands_vec):
                        job_script_sbatch_content += f'if [ "$SLURM_ARRAY_TASK_ID" == "{idx}" ]; then {stringify_commands[idx]} > {os.path.join(slurm_jobs_folder, f"{idx}.jobout")} 2> {os.path.join(slurm_jobs_folder, f"{idx}.joberror")} || exit {ERRCODE}; fi\n'
                job_script_sbatch_content += "\n"
                with open(job_script_sbatch_path, "w", encoding="utf8") as f:
                    f.write(job_script_sbatch_content)
                print(f"{job_script_sbatch_path}:\n")
                print(job_script_sbatch_content, end='')
                confirm_or_die(f"Please review job batch file above, jobs will be sent to slurm")

                slurm_run_sbatch_wait(job_script_sbatch_path, [i for i in range(job_count)], slurm_commands_vec=slurm_commands_vec
                    , use_interactive_partition=DEBUG_USING_INTERACTIVE_PARTITION)

            # gather the data with feeding results
            results = {}
            for idx, command in enumerate(slurm_commands_vec):
                with open(os.path.join(slurm_jobs_folder, f"{idx}.jobout"), "r", encoding="utf8") as f:
                    results[stringify_commands[idx]] = f.read()
            
            # rerun the simulation feeding the results
            def feeding_output(command):
                stringify_command = run_stringify_command(command)
                if stringify_command not in results:
                    print(f"couldn't find results for command '{stringify_command}'")
                    raise "result not found"
                return results[stringify_command], 0
            return program(slurm_commands_vec=None, run_command_get_stdout=feeding_output)

    return wrapper

def slurm_distribute_run(program):
    wrapper = slurm_distribute_wrap(program)
    wrapper()
    return wrapper

def slurm_run_sbatch_wait(job_script_sbatch_path, job_indices, use_interactive_partition=False, slurm_commands_vec=None, original_sbatch_file_path=None):
    slurm_jobs_folder = os.path.dirname(job_script_sbatch_path)
    if slurm_commands_vec is None:
        slurm_commands_vec = {}
        for idx in job_indices:
            slurm_commands_vec[idx] = "<unknown>"
    if original_sbatch_file_path is None:
        original_sbatch_file_path = job_script_sbatch_path

    # run the batch file
    slurm_command = ["sbatch"] + (["-p", "interactive"] if use_interactive_partition else []) + [job_script_sbatch_path]
    process = subprocess.Popen(slurm_command, universal_newlines=True, stdout=subprocess.PIPE, stderr=sys.stderr)
    process.wait()
    stdout, _ = process.communicate()
    print(stdout, end="")
    assert process.returncode == 0, "sbatch command fails..."
    JOB_ID = stdout.split(" ")[-1].strip(" \r\n")
    # print(f"JOB_ID: {JOB_ID}")
    # time.sleep(1)  # sleep first because it takes some time to be observed in squeue command output

    # wait for the job to finish
    while True:
        process = subprocess.Popen(["squeue", "--me", "--array"], universal_newlines=True, stdout=subprocess.PIPE, stderr=sys.stderr)
        process.wait()
        stdout, _ = process.communicate()
        assert process.returncode == 0, "squeue command fails..."
        active_jobs = {}
        stdout_split = stdout.strip(" \r\n").split("\n")
        status_report_content = stdout_split[0] + "\n"
        for line in stdout_split:
            fields = line.strip(" \r\n").split(" ")
            sub_id = fields[0]
            for idx in job_indices:
                if sub_id == f"{JOB_ID}_{idx}":
                    active_jobs[idx] = line
                    status_report_content += line + "\n"
        with open(os.path.join(slurm_jobs_folder, f"unfinished.tasks"), "w", encoding="utf8") as f:
            f.write(status_report_content)
            f.flush()
        print(f"\rjobs remaining: [{len(active_jobs)}/{len(job_indices)}]", end="")
        if len(active_jobs) == 0:
            break
        time.sleep(0.3)  # sleep first because it takes some time to be observed in squeue command output
    print()

    # check all states
    check_cnt = 0
    while True:
        process = subprocess.Popen(["sacct", "-j", f"{JOB_ID}"], universal_newlines=True, stdout=subprocess.PIPE, stderr=sys.stderr)
        try:
            process.wait(timeout=60)
            stdout, _ = process.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            print("sacct command timeout, strange... but try again")
            continue
        assert process.returncode == 0, "sacct command fails..."
        stdout_split = stdout.strip(" \r\n").split("\n")
        status_report_content = stdout_split[0] + "\n"
        error_count = 0
        found_entries = {}
        for idx in job_indices:
            found_entries[idx] = False
        failed_indices = set()
        for line in stdout_split:
            fields = line.strip(" \r\n").split(" ")
            sub_id = fields[0]
            for idx in job_indices:
                if sub_id == f"{JOB_ID}_{idx}":
                    found_entries[idx] = True
                    status_report_content += line + "\n"
                    if line.count(f" COMPLETED ") < 1:
                        error_count += 1
                        failed_indices.add(idx)
        print(status_report_content, end="")
        with open(os.path.join(slurm_jobs_folder, f"all.tasks"), "w", encoding="utf8") as f:
            f.write(status_report_content)
            f.flush()
        has_unfounded_entry = False
        for idx in job_indices:
            if found_entries[idx] == False:
                has_unfounded_entry = True
                break
        if has_unfounded_entry:
            print("sacct seems to require the wrong list, run again after 3 sec")
            check_cnt += 1
            if check_cnt >= 3:
                answer = input(f"[sacct doesn't seem to give the right list, perhaps because of interrupted tasks] Break? [B/BREAK, default to continue]")
                if answer.upper() in ["B", "BREAK"]:
                    for idx in job_indices:
                        if found_entries[idx] == False:
                            error_count += 1
                            failed_indices.add(idx)
                    break
            time.sleep(3)
            continue
        break
    failed_indices = [e for e in failed_indices]
    failed_indices.sort()
    if error_count != 0:
        print(f"failed cases: {failed_indices}, corresponding commands are:")
        for idx in failed_indices:
            print(f"{idx}. {slurm_commands_vec[idx]}")
        rerun_sh_filename = datetime.now().strftime("rerun-%m-%d-%Y-%H-%M-%S.sh")
        rerun_sh_filepath = os.path.join(slurm_jobs_folder, rerun_sh_filename)
        rerun_py_path = os.path.join(os.path.dirname(__file__), "slurm_rerun_failed.py")
        print(f"to re-run the failed cases, run `slurm_jobs/{rerun_sh_filename}`(full path: {rerun_sh_filepath})\n  or command `python3 {rerun_py_path} {original_sbatch_file_path} {','.join([str(e) for e in failed_indices])}`")
        with open(rerun_sh_filepath, "w", encoding="utf8") as f:
            f.write(f"#!/bin/sh\n\n")
            f.write(f"# failed indices: {','.join([str(e) for e in failed_indices])}\n")
            f.write(f"python3 {rerun_py_path} {original_sbatch_file_path} {','.join([str(e) for e in failed_indices])}\n")
        os.chmod(rerun_sh_filepath, os.stat(rerun_sh_filepath).st_mode | stat.S_IRWXU)
        confirm_or_die(f"Seems like there are {error_count} failed tasks, if continue, we'll assume they're all good")

def get_job_count_from_sbatch(sbatch_file_path):
    job_count = None
    with open(sbatch_file_path, "r", encoding="utf8") as f:
        for line in f.readlines():
            if line.startswith("#SBATCH --array=0-"):
                job_count = int(line[len("#SBATCH --array=0-"):]) + 1
    return job_count

if __name__ == "__main__":
    # test 3 simple jobs

    def default_run_command_get_stdout(command, no_stdout=False):
        process = subprocess.Popen(command, universal_newlines=True, stdout=sys.stdout if no_stdout else subprocess.PIPE, stderr=sys.stderr)
        process.wait()
        stdout, _ = process.communicate()
        return stdout, process.returncode

    SLURM_DISTRIBUTE_CPUS_PER_TASK = 1
    SLURM_DISTRIBUTE_MEM_PER_TASK = '1G'
    SLURM_DISTRIBUTE_TIME = "00:05:00"

    @slurm_distribute_run
    def simple_test(slurm_commands_vec = None, run_command_get_stdout=default_run_command_get_stdout):
        for i in range(3):
            if i % 2 == 0:
                command = ["echo", f"{i}", f"job finished"]
            else:
                command = ["sleep", "1"]
            if slurm_commands_vec is not None:
                slurm_commands_vec.append(command)
                continue
            stdout, returncode = run_command_get_stdout(command)
            print("\n" + stdout)
            assert returncode == 0, "command fails..."