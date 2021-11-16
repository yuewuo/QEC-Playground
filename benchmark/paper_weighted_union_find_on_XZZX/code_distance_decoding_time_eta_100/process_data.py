import json
import numpy as np

print_title = f"<di> <dj> <T> <percent_0_tol> <percent_10_tol> <percent_50_tol> <percent_100_tol> <sample_cnt> <error_cnt> <avr_all> <std_all> <max_all> <avr_err> <std_err> <max_err>"

def generate_print(di, dj, T, data, time_field_name):
    time_vec = np.sort([e[time_field_name] for e in data])
    error_time_vec = np.sort([e[time_field_name] for e in data if e["error"] == True])
    sample_cnt = len(time_vec)
    error_cnt = len(error_time_vec)
    # time regardless of error
    avr_all = np.average(time_vec)
    max_all = np.amax(time_vec)
    std_all = np.std(time_vec)
    # time only for those contain error
    avr_err = np.average(error_time_vec)
    max_err = np.amax(error_time_vec)
    std_err = np.std(error_time_vec)
    def percent_tol(percent):  # allows `percent` more errors to happen, what is the maximum 
        err_more = int(percent * len(error_time_vec))
        if err_more >= len(time_vec):
            err_more = len(time_vec)
        if err_more <= 1:
            err_more = 1
        return time_vec[-err_more]
    percent_0_tol = percent_tol(0)
    percent_10_tol = percent_tol(0.1)
    percent_50_tol = percent_tol(0.5)
    percent_100_tol = percent_tol(1)
    return f"{di} {dj} {T} {percent_0_tol} {percent_10_tol} {percent_50_tol} {percent_100_tol} {sample_cnt} {error_cnt} {avr_all} {std_all} {max_all} {avr_err} {std_err} {max_err}"

def process_file(log_filepath, pairs, time_field_name):
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
        assert configuration["di"] == di and configuration["dj"] == dj and configuration["MeasurementRounds"] == T

    # process each config
    content += print_title + "\n"
    for i in range(len(pairs)):
        di, dj, T = pairs[i]
        data = data_vec[i]
        content += generate_print(di, dj, T, data, time_field_name) + "\n"
    return content
