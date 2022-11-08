import json

di = 3

log_filename = f"study_error_pattern_causing_logical_error_d_{di}.txt"
with open(log_filename, "r", encoding="utf8") as f:
    lines = f.readlines()
N = 0
M = 0
P1 = 0  # one Pauli and one Erasure
P2 = 0  # cause by all erasures
for line in lines:
    line = line.strip("\r\n ")
    if line[0] == "#":
        continue
    data = json.loads(line)
    N += 1
    has_logical_error = data["error"]
    if has_logical_error:
        M += 1
    else:
        continue
    has_erasure_count = 0
    no_erasure_count = 0
    for element in data["error_pattern"]:
        t, i, j, error, has_erasure = element
        if has_erasure:
            has_erasure_count += 1
        else:
            no_erasure_count += 1
    if has_erasure_count == 1 and no_erasure_count == 1:
        P1 += 1
    if no_erasure_count == 0:
        P2 += 1
# M/N = logical error rate
# P1/N = pure erasure caused logical error
print(f"d = {di}: M/N = {M/N}, P1/N = {P1/N}, P2/N = {P2/N},  (N={N}, M={M}, P1={P1}, P2={P2})")
