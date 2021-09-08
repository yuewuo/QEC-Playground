import json

di_vec = [3, 5, 7, 9]
for di in di_vec:
    log_filename = f"study_error_pattern_causing_logical_error_d_{di}.txt"
    with open(log_filename, "r", encoding="utf8") as f:
        lines = f.readlines()
    for line in lines:
        line = line.strip("\r\n ")
        if line[0] == "#":
            continue
        data = json.loads(line)
        if data["error"] is False:
            continue  # don't care cases without logical error
        print(data)
    exit(0)
