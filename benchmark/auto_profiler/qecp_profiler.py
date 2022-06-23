"""
common utilities for profiling
"""

import os, sys, subprocess, re
import auto_profiler as ap
import toml

"""
compare mode will skip profiling, but instead work on compare functions
"""
COMPARE_WITH_VERSION = None
if 'COMPARE_WITH_VERSION' in os.environ and os.environ["COMPARE_WITH_VERSION"] != "":
    COMPARE_WITH_VERSION = os.environ["COMPARE_WITH_VERSION"]
print(f"[info] COMPARE_WITH_VERSION = {COMPARE_WITH_VERSION}")

qecp_path = os.path.join(ap.rust_dir, "target", "release", "qecp")

QEC_PLAYGROUND_COMPILATION_DONE = False
if 'MANUALLY_COMPILE_QEC' in os.environ and os.environ["MANUALLY_COMPILE_QEC"] == "TRUE":
    QEC_PLAYGROUND_COMPILATION_DONE = True
def compile_code_if_necessary(additional_build_parameters=None):
    global QEC_PLAYGROUND_COMPILATION_DONE
    if QEC_PLAYGROUND_COMPILATION_DONE is False:
        build_parameters = ["cargo", "build", "--release"]
        if additional_build_parameters is not None:
            build_parameters += additional_build_parameters
        # print(build_parameters)
        process = subprocess.Popen(build_parameters, universal_newlines=True, stdout=sys.stdout, stderr=sys.stderr, cwd=ap.rust_dir)
        process.wait()
        assert process.returncode == 0, "compile has error"
        QEC_PLAYGROUND_COMPILATION_DONE = True

# get version
def get_version():
    # compile_code_if_necessary()
    stdout, returncode, _ = ap.timed_run_command([qecp_path, "--help"])
    assert returncode == 0, "get version command fails..."
    title_line = stdout.split("\n")[0]
    reg = re.compile("^QECPlayground (?P<version>\d+\.\d+\.\d+)$")
    reg_match = reg.match(title_line)
    assert reg_match is not None, "cannot extract version information, please check the output format of qecp"
    return reg_match.groupdict()['version']
qecp_version = get_version()
print(f"[info] rust qecp version: {qecp_version}")

def get_qecp_data_folder(qecp_version=qecp_version):
    return os.path.join(ap.data_dir, f"qecp_v{qecp_version}")
print(f"[info] rust qecp data folder: {get_qecp_data_folder()}")

def run_flamegraph_qecp_profile_command(name, command):
    qecp_data_folder = get_qecp_data_folder()
    flamegraph_commands = ["flamegraph", "-o", os.path.join(qecp_data_folder, f"{name}.svg")]
    if COMPARE_WITH_VERSION is not None:
        # print(f"[skipped] {name}")
        return
    if sys.platform == "darwin":
        print("[warning] due to limitations of dtrace on macOS, it must run with sudo privilege, see https://github.com/flamegraph-rs/flamegraph for more information")
        flamegraph_commands = ["sudo"] + flamegraph_commands
    return ap.run_profile_command(name, qecp_data_folder, flamegraph_commands + [qecp_path] + command)

def compare_qecp_profile(name):
    def decorator(func):
        if COMPARE_WITH_VERSION is None:
            return
        now_data_folder = get_qecp_data_folder()
        compare_data_folder = get_qecp_data_folder(COMPARE_WITH_VERSION)
        print(f"\n[compare] {name}")
        now_record_path = os.path.join(now_data_folder, f"{name}.toml")
        compare_record_path = os.path.join(compare_data_folder, f"{name}.toml")
        if not os.path.exists(now_record_path):
            print(f"[warning] comparison skipped because file not found: {now_record_path}")
            return
        if not os.path.exists(compare_record_path):
            print(f"[warning] comparison skipped because file not found: {compare_record_path}")
            return
        now_data = toml.load(now_record_path)
        compare_data = toml.load(compare_record_path)
        func(now_data, compare_data)
        return func  # return the function itself to be visible where it's defined
    return decorator



if COMPARE_WITH_VERSION is None:
    ap.confirm_action_to_data_folder(get_qecp_data_folder())

if COMPARE_WITH_VERSION is not None:
    compare_data_folder = get_qecp_data_folder(COMPARE_WITH_VERSION)
    print(f"[info] compare with data folder: {compare_data_folder}")
    assert os.path.exists(compare_data_folder), "compare folder not exists"
