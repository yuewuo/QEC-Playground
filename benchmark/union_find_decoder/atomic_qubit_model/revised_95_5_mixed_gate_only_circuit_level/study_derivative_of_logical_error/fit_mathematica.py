
p_pauli_threshold = 0.00578
p_erasure_threshold = 0.0519

"""
Fit task 1:
Mixed model with 95% erasure error + 5% Pauli error

pL = C1 * (p_pauli / p_pth) ^ ((d+1)/2) + C2 * (p_erasure / p_eth) ^ d

minimizes: sum( (ln(C1 * (p_pauli / p_pth) ^ ((d+1)/2) + C2 * (p_erasure / p_eth) ^ d) - ln(pL_experiment))^2 )

Mathematica:
NMinimize[..., {C1, C2}]
"""

data_vec = { "3": [], "5": [], "7": [], "9": [] }
with open("raw_data.txt", "r", encoding="utf-8") as f:
    for line in f.readlines():
        line = line.strip("\r\n ")
        if line == "":
            print("")
            continue
        p, p_pauli, di, T, total_cnt, error_cnt, error_rate, dj, confidence_interval = line.split(" ")
        data_vec[di].append((float(p), float(error_rate), float(confidence_interval)))

for d in [3,5,7,9]:
    data = data_vec[f"{d}"]
    sums = []
    for p, error_rate, confidence_interval in data:
        p_pauli = p * 0.05
        p_erasure = p * 0.95
        sums.append(f"(Log[C1 * {(p_pauli / p_pauli_threshold) ** ((d+1)/2):.12f} + C2 * {(p_erasure / p_erasure_threshold) ** d:.12f}] - Log[{error_rate:.12f}])^2")
    expression = f"NMinimize[{{{' + '.join(sums)}, C1 > 0, C2 > 0}}, {{C1, C2}}]"
    print(d)
    print(expression)

"""
Results:
d = 3: {0.0395095, {C1 -> 0.966011, C2 -> 0.168716}}
d = 5: {0.28723, {C1 -> 1.3019, C2 -> 1.53519}}
d = 7: {0.341174, {C1 -> 2.58495, C2 -> 5.78156}}
d = 9: {0.167091, {C1 -> 9.00415, C2 -> 15.4935}}
"""

dC1C2_vec = [(3, 0.966011, 0.16097), (5, 1.3019, 1.41953), (7, 2.58495, 5.18106), (9, 9.00415, 13.456)]

# generate_fit_data

for d, C1, C2 in dC1C2_vec:
    data = data_vec[f"{d}"]
    filename = f"fit_compare_d_{d}.txt"
    with open(filename, "w", encoding="utf8") as f:   
        for p, error_rate, confidence_interval in data:
            p_pauli = p * 0.05
            p_erasure = p * 0.95
            f.write(f"{p} {error_rate} {confidence_interval} {C1 * (p_pauli / p_pauli_threshold) ** ((d+1)/2) + C2 * (p_erasure / p_erasure_threshold) ** d}\n")
