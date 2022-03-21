"""
common utilities for profiling
"""

import os, sys, subprocess, re
import auto_profiler as ap

qecp_path = os.path.join(ap.rust_dir, "target", "release", "rust_qecp")

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
    compile_code_if_necessary()
    stdout, returncode, _ = ap.timed_run_command([qecp_path, "--help"])
    assert returncode == 0, "get version command fails..."
    title_line = stdout.split("\n")[0]
    reg = re.compile("^QECPlayground (?P<version>\d+\.\d+\.\d+)$")
    reg_match = reg.match(title_line)
    assert reg_match is not None, "cannot extract version information, please check the output format of rust_qecp"
    return reg_match.groupdict()['version']
qecp_version = get_version()
print(f"[info] rust qecp version: {qecp_version}")

qecp_data_folder = os.path.join(ap.data_dir, f"qecp_v{qecp_version}")
print(f"[info] rust qecp data folder: {qecp_data_folder}")

def run_flamegraph_qecp_profile_command(name, command):
    flamegraph_commands = ["flamegraph", "-o", os.path.join(qecp_data_folder, f"{name}.svg")]
    if sys.platform == "darwin":
        print("[warning] due to limitations of dtrace on macOS, it must run with sudo privilege, see https://github.com/flamegraph-rs/flamegraph for more information")
        flamegraph_commands = ["sudo"] + flamegraph_commands
    return ap.run_profile_command(name, qecp_data_folder, flamegraph_commands + [qecp_path] + command)

ap.confirm_action_to_data_folder(qecp_data_folder)
