import math

# define function
print("""
nPr[n_, r_] := n!/(n - r)!;
nCr[n_, r_] := Binomial[n, r];
combination[d_, j_, pet_] := d * d * nPr[d, j] * Power[pet, j];
estimate1[d_, pe_, pp_, tA_, tE_, tP_, pet_] := tA * Sum[combination[d, j, pet] * Sum[nCr[i+d+j-2*i, i] * Power[pe * tE, (d+j-2*i)] * Power[pp * tP, i], {i, 0, Floor[(d-1)/2] + j - 1}], {j, 0, Floor[(d-1)/2] - 1}];
""")

di_vec = [3,5,7,9]

for di in di_vec:
    data = []
    raw_data_filename = f"raw_data_d_{di}.txt"
    with open(raw_data_filename, "r", encoding="utf8") as f:
        for line in f.readlines():
            line = line.strip("\r\n ")
            if line == "":
                continue
            p, p_pauli, di, T, total_cnt, error_cnt, error_rate, dj, confidence_interval = line.split(" ")
            if float(p) < 0.02 and float(p) > 0.01317:
                # ignore those around the threshold
                data.append((int(di), float(p), float(error_rate)))
    print(f"""data{di} = {{{",".join([f"{{{d},{p:.12f},{math.log(error_rate):.6f}}}" for d,p,error_rate in data])}}};""")

tE = 2  # tune_E
tP = 2  # tune_P
pet = 6  # possibility_each_turning
for di in di_vec:
    # print(f"""NMinimize[{{Total[Map[(#[[3]] - Log[estimate1[#[[1]], #[[2]]*0.95, #[[2]]*0.05, tA, {tE}, {tP}, {pet}]])^2 &, data{di}]], tA > 0}}, {{tA}}]""")

    # test individual parameters
    # print(f"""NMinimize[{{Total[Map[(#[[3]] - Log[estimate1[#[[1]], #[[2]]*0.95, #[[2]]*0.05, tA, tE, {tP}, {pet}]])^2 &, data{di}]], tA > 0, tE > 0}}, {{tA, tE}}]""")
    # print(f"""NMinimize[{{Total[Map[(#[[3]] - Log[estimate1[#[[1]], #[[2]]*0.95, #[[2]]*0.05, tA, tE, tP, {pet}]])^2 &, data{di}]], tA > 0, tE > 0, tP > 0}}, {{tA, tE, tP}}]""")
    print(f"""NMinimize[{{Total[Map[(#[[3]] - Log[estimate1[#[[1]], #[[2]]*0.95, #[[2]]*0.05, tA, tE, tE, {pet}]])^2 &, data{di}]], tA > 0, tE > 0}}, {{tA, tE}}]""")



"""

Plot with Slider

nPr[n_, r_] := n!/(n - r)!;
nCr[n_, r_] := Binomial[n, r];
combination[d_, j_, pet_] := d * d * nPr[d, j] * Power[pet, j];
estimateErrorRate[d_, pe_, pp_, tA_, tE_, tP_, pet_] := tA * Sum[combination[d, j, pet] * Sum[nCr[i+d+j-2*i, i] * Power[pe * tE, (d+j-2*i)] * Power[pp * tP, i], {i, 0, (d-1)/2+j-1}], {j, 0, (d-1)/2-1}];
Manipulate[LogPlot[Table[estimateErrorRate[d, 0.95*10^logp, 0.05*10^logp, tA, tE, tP, pet], {d, 3, 9, 2}], {logp, -5, -1}], {tA, 0, 100}, {tE, 1, 10}, {tP, 1, 10}, {pet, 1, 10}]

"""
