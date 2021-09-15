import math, random, scipy.stats
import numpy as np

for filename in ["raw_data.txt", "raw_data_pure_pauli.txt"]:
    print(filename)
    with open(filename, "r", encoding="utf8") as f:
        lines = f.readlines()
        data = []
        idx = 0
        for line in lines:
            line = line.strip("\r\n ")
            if line == "":
                print("")
                continue
            p, p_pauli, di, T, total_cnt, error_cnt, error_rate, dj, confidence_interval = line.split(" ")
            p = float(p)
            di = int(di)
            error_rate = float(error_rate)
            confidence_interval = float(confidence_interval)
            if len(data) > 0 and di != data[-1][2]:
                idx = 0
            if idx >= 4 and idx % 2 == 0:
                # five points for slope estimation
                p_vec = [data[i][1] for i in range(-4,0)] + [p]
                error_rate_vec = [data[i][3] for i in range(-4,0)] + [error_rate]
                confidence_interval_vec = [data[i][4] for i in range(-4,0)] + [confidence_interval]
                X = [math.log(e) for e in p_vec]
                # print(line, X, [math.log(e) for e in error_rate_vec])
                baseline_slope, _, _, _, _ = scipy.stats.linregress(X, [math.log(e) for e in error_rate_vec])
                slope_vec = []
                for random_round in range(100):
                    Y = [math.log(e) for e in error_rate_vec]
                    for i in range(5):
                        Y[i] += random.gauss(0, confidence_interval_vec[i] / 1.96)
                    slope, intercept, _, _, _ = scipy.stats.linregress(X, Y)
                    if abs(baseline_slope - slope) > 0.5:
                        # print(f"ignoring bad point: baseline_slope: {baseline_slope}, slope: {slope}")
                        continue
                    slope_vec.append(slope)
                    # print(line, slope)
                slope = np.mean(slope_vec)
                slope_confidence_interval = 1.96 * np.std(slope_vec)
                print(data[-2][0], slope, slope_confidence_interval)
            data.append((line, p, di, error_rate, confidence_interval))
            idx += 1
