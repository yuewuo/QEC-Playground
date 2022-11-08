import os, json, math
import numpy as np
from sklearn.linear_model import LinearRegression

GROUPING = 5

def main():
    # for filename in ["duf_17_0.01.json"]:
    fitted_data = linear_fit_logical_error_rate("results.txt", discard_last_few=2)
    threshold_maximum_clock_cycles = {}
    for filename in os.listdir():
        if filename.endswith(".json"):
            # print("parsing", filename)
            parse_file(filename, filename[:-5] + ".txt", GROUPING)
            d = int(filename.split("_")[1])
            grouped_rate = parse_file(filename, None, 1)
            threshold_clock_cycle = find_threshold_clock_cycle(fitted_data[d], grouped_rate)
            threshold_maximum_clock_cycles[d] = (threshold_clock_cycle, len(grouped_rate) - 1)
    with open("threshold_clock_cycle.txt", "w", encoding="utf-8") as f:
        for d in range(30):
            if d in threshold_maximum_clock_cycles:
                (threshold_clock_cycle, maximum_clock_cycle) = threshold_maximum_clock_cycles[d]
                # cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [3,5,7,9,11,13,15,17,19,21] [1e-1] -p0 -b1000 -e1000000 -m1000000 --only_count_logical_x
                # run the above command to test maximum clock cycle, and fit it using Mathematica using
                # data = {{3,31},{5,106},{7,198},{9,290}}
                # `Fit[data, {1, x, x^2}, x]`
                #
                # d  experiment   theory
                # 3   31            45
                # 5   106           105
                # 7   198           189
                # 9   290           297
                theoretical_maximum_clock_cycle = d * (3 * d + 6)
                print("%d %d %d %d" % (d, threshold_clock_cycle, maximum_clock_cycle, theoretical_maximum_clock_cycle))

def linear_fit_logical_error_rate(in_filepath, discard_last_few=0):
    fitted_data = {}
    with open(in_filepath, "r", encoding="utf-8") as f:
        lines = [l.split(" ") for l in f.readlines()]
        discarded_lines = lines[:-discard_last_few]
        data = [(int(e[1]), float(e[4])) for e in discarded_lines]
    X = np.array([[d[0]] for d in data])
    y = np.log(np.array([d[1] for d in data]))
    reg = LinearRegression().fit(X, y)
    for line in lines:
        d = int(line[1])
        fitted_data[d] = np.exp(reg.predict(np.array([[d]])))[0]
    print("regression score:", reg.score(X, y))
    print("fitted_data:", fitted_data)
    return fitted_data

def parse_file(in_filepath, out_filepath, grouping=1):
    with open(in_filepath, "r", encoding="utf-8") as f:
        data = [(int(e[0]), int(e[1])) for e in json.load(f)]
    data_len = len(data)
    total_sum = 0
    grouped = []
    for i in range(math.ceil(data_len / grouping)):
        i_sum = 0
        for j in range(grouping):
            idx = i * grouping + j
            if idx >= data_len:
                break
            local_sum = data[idx][0] + data[idx][1]
            total_sum += local_sum
            i_sum += local_sum
        grouped.append(i_sum)
    grouped_rate = [e / total_sum for e in grouped]
    lines = []
    for idx, rate in enumerate(grouped_rate):
        lines.append("%d %e" % (idx * grouping, rate))
    if out_filepath is not None:
        with open(out_filepath, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))
    return grouped_rate

def find_threshold_clock_cycle(error_rate_threshold, grouped_rate):
    last_idx = 0
    error_sum = 0
    while last_idx < len(grouped_rate) and error_sum < error_rate_threshold:
        error_sum += grouped_rate[len(grouped_rate) - 1 - last_idx]
        last_idx += 1
    # conservatively find the threshold
    return len(grouped_rate) - last_idx + 1

if __name__ == "__main__":
    main()
