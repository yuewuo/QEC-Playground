"""
version b330a31 (initial setup)
    time_uf_grow: 3.3527354275627674 0.9686155775808986
    time_uf_merge: 2.468715605448675 0.987396938489815
    time_uf_replace: 1.8994782349478885 0.9837467957441839
    time_uf_update: 3.697717213362454 0.990606809917821
    time_uf_remove: 2.1921364060895505 0.9730189829334444
conclusion: "time_uf_grow" and "time_uf_update" step needs to be examined


version 0b7a778 (change all HashSet/Map to BTreeSet/Map because it has lower iteration complexity)
    time_uf_grow: 3.0661304128080737 0.950191055849983
    time_uf_merge: 2.48933105090094 0.9941893001537699
    time_uf_replace: 2.091536174570542 0.9986348955117652
    time_uf_update: 3.015519134017408 0.9766076585623577
    time_uf_remove: 2.961061023900995 0.9917082865728023


"""

import sys, os, json, math
import scipy.stats

fixed_configuration = None
configurations = []
data_vec = []

with open("pm_decoding_time_study_unweighted_unbiased.txt", "r", encoding="utf-8") as f:
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

# p_vec = [0.01, 0.03, 0.1, 0.3]
p_vec = [0.01]

step_vec = ["time_uf_grow", "time_uf_merge", "time_uf_replace", "time_uf_update", "time_uf_remove", "time_run_to_stable"]
fitting_data_vec = [[[] for si in range(len(step_vec))] for pi in range(len(p_vec))]
plot_data_vec = [[[] for si in range(len(step_vec))] for pi in range(len(p_vec))]

for i in range(0, len(configurations)):
    config = configurations[i]
    vec = data_vec[i]
    idx = -1
    for i in range(len(p_vec)):
        p = p_vec[i]
        ratio = config["p"] / p
        if ratio > 0.99 and ratio < 1.01:
            idx = i
    assert idx >= 0, "must find similar p"
    fitting_data = fitting_data_vec[idx]
    plot_data = plot_data_vec[idx]
    error_count = 0
    success_count = 0
    # these only accounts successful cases
    time_build_decoders_vec = []
    time_run_to_stable_vec = []
    time_build_decoders_run_to_stable_vec = []
    time_steps_vec = [[] for si in range(len(step_vec))]
    for e in vec:
        if e["error"]:
            error_count += 1
        else:
            success_count += 1
            time_build_decoders_vec.append(e["time_build_decoders"])
            time_run_to_stable_vec.append(e["time_run_to_stable"])
            time_build_decoders_run_to_stable_vec.append(e["time_build_decoders"] + e["time_run_to_stable"])
            for si in range(len(step_vec)):
                step_name = step_vec[si]
                time_steps_vec[si].append(e[step_name])
    upper_idx = min(max(0, int(success_count - error_count * 0.1)), success_count - 1)  # this will lead to error rate of 110% x original error rate
    print(f"error: {error_count}, success_count: {success_count}, error_rate: {error_count/(error_count+success_count)}")
    print(f"time_build_decoders: {average(time_build_decoders_vec)}, {sorted(time_build_decoders_vec)[upper_idx]}")
    print(f"time_run_to_stable: {average(time_run_to_stable_vec)}, {sorted(time_run_to_stable_vec)[upper_idx]}")
    print(f"time_build_decoders_run_to_stable: {average(time_build_decoders_run_to_stable_vec)}, {sorted(time_build_decoders_run_to_stable_vec)[upper_idx]}")
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

for i in range(len(p_vec)):
    for si in range(len(step_vec)):
        p = p_vec[i]
        step_name = step_vec[si]
        fitting_data = fitting_data_vec[i][si]
        X = [math.log(e[0]) for e in fitting_data]
        Y = [math.log(e[1]) for e in fitting_data]
        slope, intercept, r, _, _ = scipy.stats.linregress(X, Y)
        print("\n\n")
        print(f"p = {p}, step_name = {step_name}")
        print(fitting_data)
        print(f"slope = {slope}")
        print(f"intercept = {intercept}")
        print(f"r_square = {r**2}")
        for e in fitting_data:
            print(f"{e[0]} {e[1]}")

for si in range(len(step_vec)):
    p = p_vec[0]
    step_name = step_vec[si]
    plot_data = plot_data_vec[i][si]
    with open(f"{step_name}.txt", "w", encoding="utf-8") as f:
        for (di, avr, count) in plot_data:
            f.write(f"{di} {avr} {count}\n")
