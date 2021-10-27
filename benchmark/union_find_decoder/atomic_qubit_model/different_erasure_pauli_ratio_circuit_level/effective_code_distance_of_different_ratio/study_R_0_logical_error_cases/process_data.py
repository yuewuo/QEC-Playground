import json, os


quantization = 1
max_case = 150

p_str_vec = []
with open(os.path.join(os.path.dirname(__file__), f"pauli_ratio_0.0_extended_range.txt"), "r", encoding="utf8") as f:
    lines = f.readlines()
    for line in lines:
        line = line.strip(" \r\n")
        if line == "":
            continue
        p_str = line.split(" ")[1]
        p_str_vec.append(p_str)


def main():
    for p_str in p_str_vec:
        log_filename = f"runtime_statistics_{p_str}.txt"
        with open(log_filename, "r", encoding="utf8") as f:
            lines = f.readlines()
        quantize_length = max_case // quantization
        quantized_erasure_acc = [0 for i in range(quantize_length + 1)]
        N = 0
        M = 0
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
            erasure_count = 0
            for element in data["error_pattern"]:
                t, i, j, error, has_erasure = element
                if has_erasure:
                    erasure_count += 1
                else:
                    assert False, "erasure only channel, must have erasure error when reported"
            quantized = max(0, min(quantize_length, int(erasure_count / quantization)))
            quantized_erasure_acc[quantized] += 1
        print((p_str, N, M, quantized_erasure_acc))
        # exit(0)

if __name__ == "__main__":
    main()
