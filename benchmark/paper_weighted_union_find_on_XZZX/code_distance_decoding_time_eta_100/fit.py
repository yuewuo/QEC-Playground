import math, random, scipy.stats
import numpy as np

for filename in ["processed_MWPM.txt", "processed_UF.txt"]:
    X = []
    Y = []
    Yavr = []
    with open(filename, "r", encoding="utf8") as f:
        lines = f.readlines()
        for line in lines[1:]:
            line = line.strip("\r\n ")
            spt = line.split(" ")
            d = int(spt[0])
            t = float(spt[6])
            tavr = float(spt[9])
            if d <= 4:
                continue
            X.append(d)
            Y.append(t)
            Yavr.append(tavr)
    # print(X)
    # print(Y)
    slope, _, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Y])
    slope_avr, _, _, _, _ = scipy.stats.linregress([math.log(d) for d in X], [math.log(t) for t in Yavr])
    print(filename, slope, slope_avr)
