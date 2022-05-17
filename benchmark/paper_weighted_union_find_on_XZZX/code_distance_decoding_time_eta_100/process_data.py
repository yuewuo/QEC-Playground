import json
import numpy as np
import math, random, scipy.stats

print_title = f"<di> <dj> <T> <percent_0_tol> <percent_10_tol> <percent_50_tol> <percent_100_tol> <sample_cnt> <error_cnt> <avr_all> <std_all> <max_all> <avr_err> <std_err> <max_err>"

def generate_print(di, dj, T, data, time_field_name):
    time_vec = np.sort([e[time_field_name] for e in data])
    sample_cnt = len(time_vec)
    # time regardless of error
    avr_all = np.average(time_vec)
    max_all = np.amax(time_vec)
    std_all = np.std(time_vec)
    return f"{di} {dj} {T} {sample_cnt} {avr_all} {std_all} {max_all}"

def fit(content, starting_d):
    X = []
    Y = []
    Yavr = []
    lines = content.split("\n")
    for line in lines[1:]:
        line = line.strip("\r\n ")
        if line == "":
            continue
        spt = line.split(" ")
        d = int(spt[0])
        t = float(spt[4])
        tavr = float(spt[5])
        if d < starting_d:
            continue
        X.append(d)
        Y.append(t)
        Yavr.append(tavr)
    # print(X)
    # print(Y)
    slope, _, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Y])
    slope_avr, _, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Yavr])
    return slope, slope_avr


def process_file(log_filepath, pairs, time_field_name, starting_d=0):
    content = ""

    configurations = []
    data_vec = []
    with open(log_filepath, "r", encoding="utf-8") as f:
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
    
    # sanity check
    assert len(pairs) == len(configurations)
    for i in range(len(pairs)):
        di, dj, T = pairs[i]
        configuration = configurations[i]
        assert configuration["di"] == di and configuration["dj"] == dj and configuration["noisy_measurements"] == T

    # process each config
    content += "# " + print_title + "\n"
    for i in range(len(pairs)):
        di, dj, T = pairs[i]
        data = data_vec[i]
        content += generate_print(di, dj, T, data, time_field_name) + "\n"
    
    slope, slope_avr = fit(content, starting_d)
    content += f"\n# slope = {slope}, slope_avr = {slope_avr}\n"

    return content
