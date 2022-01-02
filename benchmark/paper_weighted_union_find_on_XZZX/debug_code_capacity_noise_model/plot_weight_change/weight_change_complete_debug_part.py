import os, sys
qec_playground_root_dir = os.popen("git rev-parse --show-toplevel").read().strip(" \r\n")
rust_dir = os.path.join(qec_playground_root_dir, "backend", "rust")
fault_toleran_MWPM_dir = os.path.join(qec_playground_root_dir, "benchmark", "fault_tolerant_MWPM")
sys.path.insert(0, fault_toleran_MWPM_dir)
from automated_threshold_evaluation import qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command
from automated_threshold_evaluation import run_qec_playground_command_get_stdout, compile_code_if_necessary
import numpy as np
import matplotlib.pyplot as plt


di = 11
p = 0.07
divide = 10
bias_eta_vec = [str(0.5 * (10 ** (i / divide))) for i in range(4 * divide)]
# print(bias_eta_vec)
parameters = f"-p1 --time_budget 3600 --use_xzzx_code --shallow_error_on_bottom --debug_print_only --debug_print_exhausted_connections".split(" ")


# only plot one node because otherwise it's too mesy
interested_node = "[12][10][9]"  # in the middle
# interested_node = "[12][20][19]"  # on the boundary

interested_addr = [int(e) for e in interested_node[1:-1].split("][")]

results = []
for bias_eta in bias_eta_vec:
    command = qec_playground_fault_tolerant_MWPM_simulator_runner_vec_command([p], [di], [di], [0], parameters + ["--bias_eta", f"{bias_eta}"])
    print(" ".join(command))

    # run experiment
    stdout, returncode = run_qec_playground_command_get_stdout(command, use_tmp_out=True)
    # print("\n" + stdout)
    assert returncode == 0, "command fails..."

    boundary = None
    edges = []

    is_interested = False
    for line in stdout.strip(" \r\n").split("\n"):
        if line[0] == "[":
            addr = line.split(":")[0]
            is_interested = (addr == interested_node)
        elif is_interested:
            head, value = line.split(": ")
            if head == "boundary":
                if value[:4] == "c = ":
                    boundary = float(value[4:])
            if head[:5] == "edge ":
                assert value[:4] == "c = "
                t,i,j = [int(e) for e in head[5:][1:-1].split("][")]
                edges.append(((t,i,j), float(value[4:])))
    results.append((boundary, edges))
    # print(bias_eta, boundary, edges)


fig = plt.figure(f"weight change")
fig.clear()
ax0 = fig.add_subplot(111)
plt.xscale("log")
ax0.set_xticks([0.5, 5, 50, 500, 5000])
ax0.set_xticklabels([0.5, 5, 50, 500, 5000])
# plt.yscale("log")
ax0.set_title(f"direct neighbors of {interested_node}")
ax0.set_xlabel("bias eta")
ax0.set_ylabel("weight")
float_bias_eta_vec = [float(e) for e in bias_eta_vec]
color_cycle = ["black"] + ['#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd', '#8c564b', '#e377c2', '#7f7f7f', '#bcbd22', '#17becf'] + ["#DDDDDD", "#AAAAAA"]
ax0.set_prop_cycle(color=color_cycle)
if results[0][0] is not None:
    boundaries = [results[i][0] for i in range(len(results))]
    ax0.plot(float_bias_eta_vec, boundaries, label="boundary")
for ni in range(len(results[0][1])):
    addr = results[0][1][ni][0]
    for i in range(len(results)):
        if addr != results[i][1][ni][0]:
            print(addr, results[i][1][ni][0])
        assert addr == results[i][1][ni][0]
    if results[0][1][ni][1] < 25 and results[0][1][ni][1] > 15 and addr[0] >= interested_addr[0] and addr[1] > interested_addr[1] and addr[2] > interested_addr[2]:  # only plot part of it for ease of visualization
        values = [results[i][1][ni][1] for i in range(len(results))]
        ax0.plot(float_bias_eta_vec, values, label=f"[{addr[0]}][{addr[1]}][{addr[2]}]")
ax0.legend()
plt.show()
