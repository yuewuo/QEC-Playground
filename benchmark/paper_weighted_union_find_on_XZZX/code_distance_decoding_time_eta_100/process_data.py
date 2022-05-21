import json
import numpy as np
import math, random, scipy.stats

USE_MEDIAN_INSTEAD = False
USE_MAX_INSTEAD = False

print_title = f"<di> <dj> <T> <sample_cnt> <avr_all> <std_all> <mid_all> <max_all>"

def generate_print(di, dj, T, data, time_field_name):
    time_vec = np.sort([time_field_name(e) for e in data])
    sample_cnt = len(time_vec)
    # time regardless of error
    avr_all = np.average(time_vec)
    mid_all = np.median(time_vec)
    std_all = np.std(time_vec)
    max_all = np.amax(time_vec)
    return f"{di} {dj} {T} {sample_cnt} {avr_all} {std_all} {mid_all} {max_all}"

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
        if USE_MEDIAN_INSTEAD:
            t = float(spt[6])
        elif USE_MAX_INSTEAD:
            t = float(spt[7])
        else:
            t = float(spt[4])
        tavr = float(spt[5])
        if d < starting_d:
            continue
        X.append(d)
        Y.append(t)
        Yavr.append(tavr)
    print(X)
    print(Y)
    slope, intercept, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Y])
    slope_avr, _, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Yavr])
    return slope, slope_avr, intercept


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
    
    slope, slope_avr, intercept = fit(content, starting_d)
    content += f"\n# slope = {slope}, slope_avr = {slope_avr}, intercept = {intercept}\n"
    content += f"# fit(x) = exp({slope} * log(x) + ({intercept}))\n\n"

    return content
