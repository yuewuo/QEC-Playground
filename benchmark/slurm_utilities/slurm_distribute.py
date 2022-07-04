import os, sys, subprocess, shutil, shlex, time, stat, hjson
from datetime import datetime
import slurm_rerun_failed
import multiprocessing

if 'SLURM_HELP' in os.environ:
    OKBLUE = '\033[94m'
    WARNING = '\033[93m'
    ENDC = '\033[0m'
    print(f"Slurm Distributed")
    print(f"  by Yue Wu (yue.wu@yale.edu)")
    print(f"Options (only with slurm):")
    print(f"  {OKBLUE}DEBUG_USING_INTERACTIVE_PARTITION{ENDC} : use interactive session, which is usually faster to get the first (and only) session")
    print(f"  {OKBLUE}SLURM_USE_SCAVENGE_PARTITION{ENDC} : use scavenge partition can leverage more cores and use less fair-share points")
    print(f"                                   but depending on the availability of machines, it's unclear whether this will speed up")
    print(f"  {OKBLUE}SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE{ENDC} : use existing data if possible, and avoid running those slurm commands again")
    print(f"Options (any machine):")
    print(f"  {WARNING}ONLY_PRINT_COMMANDS{ENDC} : to print command only and then exit")
    print(f"  {WARNING}SLURM_USE_EXISTING_DATA{ENDC} : use existing data to run the script")
    print(f"")
    print(f"For more details, please read the script file: {__file__}")
    print(f"(note: to run normally, you must delete SLURM_HELP in your env variable)")
    exit(0)

DEBUG_USING_INTERACTIVE_PARTITION = False  # only enable while debugging

# to check which node causes problem, run `sacct -j <JOBID> | grep FAILED`
# NODE_BLACK_LIST = ["p08r07n[01-08]", "p09r11n25"]  # these nodes fails
NODE_BLACK_LIST = []

# utility tool
ONLY_PRINT_COMMANDS = False
if 'ONLY_PRINT_COMMANDS' in os.environ and os.environ["ONLY_PRINT_COMMANDS"] != "":
    ONLY_PRINT_COMMANDS = True

# check for slurm flags in environment
SLURM_DISTRIBUTE_ENABLED = False
SLURM_USE_EXISTING_DATA = False
SLURM_USE_SCAVENGE_PARTITION = False
if 'SLURM_USE_EXISTING_DATA' in os.environ and os.environ["SLURM_USE_EXISTING_DATA"] != "":
    SLURM_USE_EXISTING_DATA = True
    SLURM_DISTRIBUTE_ENABLED = True  # always use slurm workflow
if 'SLURM_DISTRIBUTE_ENABLED' in os.environ and os.environ["SLURM_DISTRIBUTE_ENABLED"] != "":
    SLURM_DISTRIBUTE_ENABLED = True
if 'SLURM_USE_SCAVENGE_PARTITION' in os.environ and os.environ["SLURM_USE_SCAVENGE_PARTITION"] != "":
    SLURM_USE_SCAVENGE_PARTITION = True
SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE = False
if 'SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE' in os.environ and os.environ["SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE"] != "":
    SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE = True

SLURM_DISTRIBUTE_FORBIDDEN = False  # never allow the script run in slurm environment
# this is used when a script doesn't want user to use slurm to distribute tasks, for example, time sensitive benchmarks
SLURM_DISTRIBUTE_DO_NOT_CHECK_JOBOUT = False  # used with the above to avoid checking for *.jobout files

if SLURM_DISTRIBUTE_ENABLED:
    SLURM_DISTRIBUTE_ENABLED = True
    SLURM_DISTRIBUTE_CPUS_PER_TASK = 36
    SLURM_DISTRIBUTE_MEM_PER_TASK = '8G'  # do not use too much memory, otherwise the task will probably fail with exit code = 1
    SLURM_DISTRIBUTE_TIME = "1-00:00:00"

def slurm_threads_or(default_threads):
    if SLURM_DISTRIBUTE_ENABLED:
        return SLURM_DISTRIBUTE_CPUS_PER_TASK
    if 'STO_THREAD_LIMIT' in os.environ and os.environ["STO_THREAD_LIMIT"] != "":
        return int(os.environ["STO_THREAD_LIMIT"])
    return default_threads

def cpu_hours(target_cpu_hours):
    target_cpu_seconds = target_cpu_hours * 3600
    if SLURM_DISTRIBUTE_ENABLED:
        return target_cpu_seconds / SLURM_DISTRIBUTE_CPUS_PER_TASK
    return target_cpu_seconds / multiprocessing.cpu_count()

def confirm_or_die(action=""):
    while True:
        answer = input(f"[{action}] Continue? [Y/YES/N/NO]")
        if answer.upper() in ["Y", "YES"]:
            return
        elif answer.upper() in ["N", "NO"]:
            raise "confirmation failed, exit"

class SlurmDistributeVec(list):
    def sanity_checked_append(self, command):
        if SLURM_DISTRIBUTE_ENABLED:
            for e in command:
                if e == "-p0":
                    confirm_or_die(f"You're using `-p0` option in slurm enabled environment, which may create more threads than allocated to you; please use `slurm_threads_or` to access the allocated number of threads in this case")
        self.append(command)

QECP_IDENT = "backend/rust/target/release/qecp"
def command_parse_qecp(command):  # used when match command
    command_lst = command.split(" ")
    if command_lst[0][-len(QECP_IDENT):] == QECP_IDENT:
        command_lst[0] = QECP_IDENT
    return "".join(command_lst).strip("\r\n ")

def slurm_distribute_wrap(program, filefolder):
    def wrapper():
        if ONLY_PRINT_COMMANDS or SLURM_DISTRIBUTE_ENABLED:
            # first gether all commands
            slurm_commands_vec = SlurmDistributeVec()
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
                    print(f"{idx}.", stringify_command)
            if ONLY_PRINT_COMMANDS:
                return None  # terminate the program

        slurm_jobs_folder = os.path.join(os.path.abspath(filefolder), "slurm_jobs")
        job_script_sbatch_path = os.path.join(slurm_jobs_folder, f"job_script.sbatch")
        # sbatch: error: Batch job submission failed: Pathname of a file, directory or other parameter too long
        # Yue 2022.5.13: in order to fix the above issue, I need to put the commands outside of sbatch file...
        job_script_sh_path = os.path.join(slurm_jobs_folder, f"job_script.sh")
        if SLURM_DISTRIBUTE_ENABLED and SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE:
            previous_job_commands = read_job_commands_from_sbatch(job_script_sbatch_path, job_script_sh_path)
            existing_jobouts = [None for e in stringify_commands]
            def command_equal(cmd1, cmd2):
                # don't care the parameter (usually binary path) before the first space character
                return command_parse_qecp(cmd1) == command_parse_qecp(cmd2)
            reusable_count = 0
            for new_i, stringify_command in enumerate(stringify_commands):
                for prev_i, previous_job_command in enumerate(previous_job_commands):
                    if command_equal(stringify_command, previous_job_command):
                        job_out_filepath = os.path.join(slurm_jobs_folder, f"{prev_i}.jobout")
                        if os.path.exists(job_out_filepath):
                            with open(job_out_filepath, "r", encoding="utf8") as f:
                                prev_jobout = f.read()
                                if prev_jobout == "":
                                    continue  # data not exists
                        print("found reusable data new_idx =", new_i, "prev_idx =", prev_i)
                        reusable_count += 1
                        existing_jobouts[new_i] = prev_jobout
            confirm_or_die(f"found {reusable_count} reusable data, apart from that only {len(stringify_commands) - reusable_count} remains to run")

        if not SLURM_DISTRIBUTE_ENABLED:
            return program()
        else:
            if not SLURM_USE_EXISTING_DATA:
                assert not SLURM_DISTRIBUTE_FORBIDDEN, "using slurm to distribute tasks are forbidden"
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
                # delete the whole folder
                if os.path.exists(slurm_jobs_folder):
                    shutil.rmtree(slurm_jobs_folder)
                # build folder and copy existing data
                os.makedirs(slurm_jobs_folder)
                if SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE:
                    for new_i, stringify_command in enumerate(stringify_commands):
                        if existing_jobouts[new_i] is not None:
                            job_out_filepath = os.path.join(slurm_jobs_folder, f"{new_i}.jobout")
                            with open(job_out_filepath, "w", encoding="utf-8") as f:
                                f.write(existing_jobouts[new_i])
                # build scripts
                parameters = [f"--job-name={job_name}", f"--time={SLURM_DISTRIBUTE_TIME}", f"--mem={SLURM_DISTRIBUTE_MEM_PER_TASK}", "--mail-type=ALL", "--nodes=1", "--ntasks=1"
                    , f"--cpus-per-task={SLURM_DISTRIBUTE_CPUS_PER_TASK}", f"--array=0-{job_count-1}"]
                parameters.append(f'--out="{os.path.join(slurm_jobs_folder, "%a.jobout")}"')
                parameters.append(f'--error="{os.path.join(slurm_jobs_folder, "%a.joberror")}"')
                if SLURM_USE_SCAVENGE_PARTITION:
                    parameters.append(f"--requeue")
                    parameters.append(f"--partition=scavenge")
                if len(NODE_BLACK_LIST) > 0:
                    parameters.append(f"--exclude={','.join(NODE_BLACK_LIST)}")
                for parameter in parameters:
                    print(f"    {parameter}")
                job_script_sbatch_content = ""
                job_script_sbatch_content += f"#!/bin/bash\n"
                for parameter in parameters:
                    job_script_sbatch_content += f"#SBATCH {parameter}\n"
                job_script_sbatch_content += f"\nsource {job_script_sh_path}\n\n"
                job_script_sh_content = "#!/bin/bash\n\n"
                ERRCODE = 91
                stringify_command_set = {}
                for idx, command in enumerate(slurm_commands_vec):
                    stringify_command = run_stringify_command(command)
                    # sanity check of the commands
                    if stringify_command in stringify_command_set:
                        confirm_or_die(f"duplicate command from {stringify_command_set[stringify_command]} and {idx}: {stringify_command}")
                    stringify_command_set[stringify_command] = idx
                if DEBUG_USING_INTERACTIVE_PARTITION:
                    job_script_sh_content += f'if [ "$SLURM_ARRAY_TASK_ID" == "0" ];\n'
                    job_script_sh_content += f'then\n'
                    for idx, command in enumerate(slurm_commands_vec):
                        job_script_sh_content += f'    {stringify_commands[idx]} || exit {ERRCODE};\n'
                    job_script_sh_content += f'fi\n'
                else:
                    for idx, command in enumerate(slurm_commands_vec):
                        job_script_sh_content += f'if [ "$SLURM_ARRAY_TASK_ID" == "{idx}" ]; then {stringify_commands[idx]} || exit {ERRCODE}; fi\n'
                job_script_sh_content += "\n"
                with open(job_script_sh_path, "w", encoding="utf8") as f:
                    f.write(job_script_sh_content)
                print(f"{job_script_sh_path}:\n")
                print(job_script_sh_content, end='')
                with open(job_script_sbatch_path, "w", encoding="utf8") as f:
                    f.write(job_script_sbatch_content)
                print(f"{job_script_sbatch_path}:\n")
                print(job_script_sbatch_content, end='')
                confirm_or_die(f"Please review job batch file above, jobs will be sent to slurm")

                if SLURM_USE_PREVIOUS_DATA_IF_POSSIBLE:
                    run_indices = []
                    for new_i, stringify_command in enumerate(stringify_commands):
                        if existing_jobouts[new_i] is None:
                            run_indices.append(new_i)
                    print(run_indices)
                    slurm_rerun_failed.rerun_failed(job_script_sbatch_path, run_indices, slurm_commands_vec=slurm_commands_vec
                        , use_interactive_partition=DEBUG_USING_INTERACTIVE_PARTITION)
                else:
                    slurm_run_sbatch_wait(job_script_sbatch_path, [i for i in range(job_count)], slurm_commands_vec=slurm_commands_vec
                        , use_interactive_partition=DEBUG_USING_INTERACTIVE_PARTITION)

            # gather the data with feeding results
            results = {}
            aggregated_results = [None] * len(slurm_commands_vec)
            if not SLURM_DISTRIBUTE_DO_NOT_CHECK_JOBOUT and os.path.exists(os.path.join(slurm_jobs_folder, f"_aggregated.hjson")):
                with open(os.path.join(slurm_jobs_folder, f"_aggregated.hjson"), "r", encoding="utf8") as f:
                    aggregated_results = hjson.loads(f.read())
                    if len(aggregated_results) != len(slurm_commands_vec):
                        confirm_or_die(f"reading from aggregated result, but it only has {len(aggregated_results)} entries instead of {len(slurm_commands_vec)} as requests, continue?")
                    for idx, (command, result) in enumerate(aggregated_results):
                        results[command] = result
            for idx, command in enumerate(slurm_commands_vec):
                if SLURM_DISTRIBUTE_DO_NOT_CHECK_JOBOUT:
                    results[stringify_commands[idx]] = "SLURM_DISTRIBUTE_DO_NOT_CHECK_JOBOUT"
                else:
                    # cover the result if jobout file exist
                    if os.path.exists(os.path.join(slurm_jobs_folder, f"{idx}.jobout")):
                        with open(os.path.join(slurm_jobs_folder, f"{idx}.jobout"), "r", encoding="utf8") as f:
                            results[stringify_commands[idx]] = f.read()
                            aggregated_results[idx] = (stringify_commands[idx], results[stringify_commands[idx]])
            
            # record the data into a single file, to reduce the number of files committed to git
            if not SLURM_DISTRIBUTE_DO_NOT_CHECK_JOBOUT:
                with open(os.path.join(slurm_jobs_folder, f"_aggregated.hjson"), "w", encoding="utf8") as f:
                    f.write(hjson.dumps(aggregated_results))

            # rerun the simulation feeding the results
            parsed_results = {}
            for key in results:
                parsed_results[command_parse_qecp(key)] = results[key]
            print(parsed_results)
            def feeding_output(command):
                stringify_command = run_stringify_command(command)
                parsed_command = command_parse_qecp(stringify_command)
                if parsed_command not in parsed_results:
                    print(f"couldn't find results for command '{stringify_command}'")
                    raise "result not found"
                return parsed_results[parsed_command], 0
            return program(slurm_commands_vec=None, run_command_get_stdout=feeding_output)

    return wrapper

def slurm_distribute_run_with_folder(program, filefolder):
    wrapper = slurm_distribute_wrap(program, filefolder)
    wrapper()
    return wrapper

def slurm_distribute_run(filefolder):
    assert isinstance(filefolder, str), "please specify folder, e.g. os.path.dirname(__file__)"
    def wrapper(program):
        slurm_distribute_run_with_folder(program, filefolder)
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
        time.sleep(60)
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

def read_job_commands_from_sbatch(sbatch_file_path, job_script_sh_path):
    job_commands = []
    for filepath in [sbatch_file_path, job_script_sh_path]:
        with open(filepath, "r", encoding="utf8") as f:
            for line in f.readlines():
                if line.startswith('if [ "$SLURM_ARRAY_TASK_ID" == '):
                    line = "".join(line.split("]; then ")[1:])
                    line = "".join(line.split(" || exit")[:-1])
                    # print(line)
                    job_commands.append(line)
    return job_commands

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
                slurm_commands_vec.sanity_checked_append(command)
                continue
            stdout, returncode = run_command_get_stdout(command)
            print("\n" + stdout)
            assert returncode == 0, "command fails..."
