import json
import numpy as np
import math, random, scipy.stats

USE_MEDIAN_INSTEAD = False
USE_MAX_INSTEAD = False

print_title = f"<di> <dj> <T> <sample_cnt> [<avr_all> <std_all> <mid_all> <max_all>]..."

def generate_print(di, dj, T, data, time_field_name):
    time_field_data = [time_field_name(e) for e in data]
    if not isinstance(time_field_data[0], list):
        time_field_data = [[e] for e in time_field_data]
    data_vec_len = len(time_field_data[0])
    sample_cnt = len(time_field_data)
    result = f"{di} {dj} {T} {sample_cnt}"
    for i in range(data_vec_len):
        time_vec = np.sort([e[i] for e in time_field_data])
        # time regardless of error
        avr_all = np.average(time_vec)
        mid_all = np.median(time_vec)
        std_all = np.std(time_vec)
        max_all = np.amax(time_vec)
        result += f" {avr_all} {std_all} {mid_all} {max_all}"
    return result

def fit(content, starting_d, ending_d):
    X = []
    Ys = []
    Yavrs = []
    groups = None
    lines = content.split("\n")
    for line in lines[1:]:
        line = line.strip("\r\n ")
        if line == "":
            continue
        spt = line.split(" ")
        if groups == None:
            assert (len(spt) - 4) % 4 == 0, "data must be groups of 4"
            groups = (len(spt) - 4) // 4
            for _ in range(groups):
                Ys.append([])
                Yavrs.append([])
        d = int(spt[0])
        if d < starting_d:
            continue
        if d >= ending_d:
            continue
        X.append(d)
        for i in range(groups):
            bias = 4 + 4 * i
            if USE_MEDIAN_INSTEAD:
                t = float(spt[bias+2])
            elif USE_MAX_INSTEAD:
                t = float(spt[bias+3])
            else:
                t = float(spt[bias])
            tavr = float(spt[bias+1])
            Ys[i].append(t)
            Yavrs[i].append(tavr)
    print(X)
    print(Ys)
    results = []
    for i in range(groups):
        slope, intercept, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Ys[i]])
        slope_avr, _, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Yavrs[i]])
        results.append((slope, slope_avr, intercept))
    return results


# target slope will be used to calculate another intercept, where the value at `starting_d` remains the same
def process_file(log_filepath, pairs, time_field_name, starting_d=0, ending_d=1000, target_slope=3):
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
    
    content += "\n"
    results = fit(content, starting_d, ending_d)
    for slope, slope_avr, intercept in results:
        target_value = slope * math.log(starting_d) + intercept
        intercept_refined = target_value - target_slope * math.log(starting_d)
        content += f"# {slope} {intercept} {slope_avr} {target_slope} {intercept_refined}  # slope, intercept, slope_avr, target_slope, intercept_refined\n"
        # content += f"# slope = {slope}, slope_avr = {slope_avr}, intercept = {intercept}\n"
        # content += f"# fit(x) = exp({slope} * log(x) + ({intercept}))\n"

    return content
