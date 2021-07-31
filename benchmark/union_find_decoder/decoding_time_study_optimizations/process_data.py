"""
version b330a31 (initial setup)

version 0b7a778 (change all HashSet/Map to BTreeSet/Map because it has lower iteration complexity)

version 3048622 (make HashSet larger to reduce time for querying)

version ec839ad (add path compression)

"""

version = "ec839ad"  # select the version for data processing
log_filename = f"decoding_time_study_optimizations_{version}.txt"


import sys, os, json, math
import scipy.stats

fixed_configuration = None
configurations = []
data_vec = []
p = 0.01

with open(log_filename, "r", encoding="utf-8") as f:
    lines = f.readlines()
    for line in lines:
        line = line.strip(" \r\n")
        if line == "":  # ignore empty line
            continue
        if line[:3] == "#f ":
            fixed_configuration = json.loads(line[3:])
        elif line[:2] == "# ":
            configurations.append(json.loads(line[2:]))
            data_vec.append([])
        else:
            data_vec[-1].append(json.loads(line))

print(fixed_configuration)

def average(lst):
    return sum(lst) / len(lst)

step_vec = ["time_uf_grow", "time_uf_merge", "time_uf_replace", "time_uf_update", "time_uf_remove", "time_run_to_stable"]
fitting_data = [[] for si in range(len(step_vec))]
plot_data = [[] for si in range(len(step_vec))]

for i in range(0, len(configurations)):
    config = configurations[i]
    vec = data_vec[i]
    ratio = config["p"] / p
    assert ratio > 0.99 and ratio < 1.01, "target p don't match"
    time_build_decoders_vec = []
    time_run_to_stable_vec = []
    time_build_decoders_run_to_stable_vec = []
    time_steps_vec = [[] for si in range(len(step_vec))]
    for e in vec:
        time_run_to_stable_vec.append(e["time_run_to_stable"])
        for si in range(len(step_vec)):
            step_name = step_vec[si]
            time_steps_vec[si].append(e[step_name])
    print(f"time_run_to_stable: {average(time_run_to_stable_vec)}")
    # di_vec = [5, 7, 9, 11, 13, 17, 21, 25, 29, 37, 43, 53, 63]
    if config["di"] >= 43:
        for si in range(len(step_vec)):
            step_name = step_vec[si]
            fitting_data_step = fitting_data[si]
            fitting_data_step.append((config["di"], average(time_steps_vec[si])))
    # plot data printed to file
    for si in range(len(step_vec)):
        step_name = step_vec[si]
        plot_data_step = plot_data[si]
        plot_data_step.append((config["di"], average(time_steps_vec[si]), len(time_steps_vec[si])))

for si in range(len(step_vec)):
    step_name = step_vec[si]
    fitting_data_step = fitting_data[si]
    X = [math.log(e[0]) for e in fitting_data_step]
    Y = [math.log(e[1]) for e in fitting_data_step]
    slope, intercept, r, _, _ = scipy.stats.linregress(X, Y)
    print("\n\n")
    print(f"p = {p}, step_name = {step_name}")
    print(fitting_data_step)
    print(f"slope = {slope}")
    print(f"intercept = {intercept}")
    print(f"r_square = {r**2}")
    for e in fitting_data_step:
        print(f"{e[0]} {e[1]}")

for si in range(len(step_vec)):
    step_name = step_vec[si]
    plot_data_step = plot_data[si]
    with open(f"{step_name}_{version}.txt", "w", encoding="utf-8") as f:
        for (di, avr, count) in plot_data_step:
            f.write(f"{di} {avr} {count}\n")
