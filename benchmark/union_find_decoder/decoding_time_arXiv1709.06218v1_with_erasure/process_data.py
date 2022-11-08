import sys, os, json, math
import scipy.stats

p_vec = [0.01, 0.02, 0.03, 0.04, 0.05]
# p_vec = [0.02]

def average(lst):
    return sum(lst) / len(lst)

for p in p_vec:

    log_filename = f"decoding_time_arXiv1709.06218v1_with_erasure_p{p}.txt"
    out_filename = f"decode_million_p{p}.txt"

    fixed_configuration = None
    configurations = []
    data_vec = []

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

    # print(fixed_configuration)

    fitting_data = []

    for i in range(0, len(configurations)):
        config = configurations[i]
        vec = data_vec[i]
        # print(config)
        time_run_to_stable_vec = []
        for e in vec:
            time_run_to_stable_vec.append(e["time_run_to_stable"])
        fitting_data.append((config["di"], average(time_run_to_stable_vec)))
    
    with open(out_filename, "w", encoding="utf-8") as f:
        f.write("# <di> <avr> <n=2d(d-1)> <1e6 samples decode time>\n")
        for (di, avr) in fitting_data:
            f.write(f"{di} {avr} {2 * di * (di-1)} {avr * 1e6}\n")

    X = [math.log(e[0]) for e in fitting_data]
    Y = [math.log(e[1]) for e in fitting_data]
    slope, intercept, r, _, _ = scipy.stats.linregress(X, Y)
    print(f"p = {p}")
    print(f"slope = {slope}")
    print(f"intercept = {intercept}")
    print(f"r_square = {r**2}")
    print("")
