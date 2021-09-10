import json

di_vec = [3, 5, 7, 9]

for di in di_vec:
    log_filename = f"study_error_pattern_causing_logical_error_d_{di}.txt"
    with open(log_filename, "r", encoding="utf8") as f:
        lines = f.readlines()
    N = 0
    M = 0
    Pe = 0
    Pp = 0
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
        if has_erasure_count == 0:
            # pure Pauli cause error
            Pp += 1
        if no_erasure_count == 0:
            # pure erasure cause error
            Pe += 1
    # M/N = logical error rate
    # Pe/N = pure erasure caused logical error
    # Pp/N = pure Pauli caused logical error
    print(f"d = {di}: M/N = {M/N}, Pe/N = {Pe/N}, Pp/N = {Pp/N},  (N={N}, M={M}, Pe={Pe}, Pp={Pp})")
