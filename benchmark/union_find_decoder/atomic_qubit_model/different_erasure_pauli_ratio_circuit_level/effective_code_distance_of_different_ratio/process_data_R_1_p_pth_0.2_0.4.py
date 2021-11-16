import math, random, scipy.stats
import numpy as np

pauli_ratio = "1"
filename = f"pauli_ratio_{pauli_ratio}.txt"
data = []
with open(filename, "r", encoding="utf8") as f:
    lines = f.readlines()
    for line in lines:
        line = line.strip(" \r\n")
        if line == "":
            continue
        spt = line.split(" ")
        p_pth = float(spt[0])
        pL = float(spt[7])
        pL_dev = float(spt[9])
        if p_pth >= 0.2 and p_pth <= 0.4:
            data.append((p_pth, pL, pL_dev))
X = [math.log(p_pth) for p_pth, pL, pL_dev in data]
slope_vec = []
for random_round in range(100):
    Y = [math.log(pL) for p_pth, pL, pL_dev in data]
    for i in range(len(data)):
        Y[i] += random.gauss(0, data[i][2] / 1.96)
    slope, intercept, _, _, _ = scipy.stats.linregress(X, Y)
    slope_vec.append(slope)
    # print(line, slope)
slope = np.mean(slope_vec)
slope_confidence_interval = 1.96 * np.std(slope_vec)
print(pauli_ratio, slope, slope_confidence_interval)
