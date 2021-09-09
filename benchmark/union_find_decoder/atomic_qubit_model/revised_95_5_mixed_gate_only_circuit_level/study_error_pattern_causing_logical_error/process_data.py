import json

di_vec = [3, 5, 7, 9]
quantization = 50

for di in di_vec:
    log_filename = f"study_error_pattern_causing_logical_error_d_{di}.txt"
    with open(log_filename, "r", encoding="utf8") as f:
        lines = f.readlines()
    quantized_erasure_acc = [0 for i in range(quantization)]
    for line in lines:
        line = line.strip("\r\n ")
        if line[0] == "#":
            continue
        data = json.loads(line)
        if data["error"] is False:
            continue  # don't care cases without logical error
        has_erasure_count = 0
        no_erasure_count = 0
        for element in data["error_pattern"]:
            t, i, j, error, has_erasure = element
            if has_erasure:
                has_erasure_count += 1
            else:
                no_erasure_count += 1
        erasure_ratio = has_erasure_count / (has_erasure_count + no_erasure_count)
        quantized_erasure_ratio = max(0, min(quantization - 1, int(erasure_ratio * quantization)))
        quantized_erasure_acc[quantized_erasure_ratio] += 1
    print(f"d = {di}")
    print(quantized_erasure_acc)
    overall_size = sum(quantized_erasure_acc)
    print([e / overall_size for e in quantized_erasure_acc])
    print("")
    for i in range(quantization):
        print((i + 0.5) / quantization, quantized_erasure_acc[i], quantized_erasure_acc[i] / overall_size)
    print("")
