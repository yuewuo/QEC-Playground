import os, sys

template_filename = os.path.join(os.path.dirname(__file__), f"MWPM_d11_p0.1.txt")

noise_bias_vec = []
with open(template_filename, "r", encoding="utf8") as f:
    for line in f.readlines():
        line = line.strip(" \r\n")
        if line == "" or line.startswith("#"):
            continue
        noise_bias = line.split(" ")[0]
        if noise_bias != "+inf":
            noise_bias_vec.append(float(noise_bias))
# print(noise_bias_vec)
err_tol = 0.0001

p = 0.1
for (filename_prefix, paramters) in [("MWPM", None), ("UF", None)]:
    for di in [11, 13]:
        filename = os.path.join(os.path.dirname(__file__), f"{filename_prefix}_d{di}_p{p}.txt")
        if filename == template_filename:
            continue
        origin_filename = os.path.join(os.path.dirname(__file__), f"{filename_prefix}_d{di}_p{p}_origin.txt")
        file_content = ""
        with open(origin_filename, "r", encoding="utf8") as f:
            for line in f.readlines():
                line = line.strip(" \r\n")
                if line == "" or line.startswith("#"):
                    continue
                noise_bias = line.split(" ")[0]
                if noise_bias == "+inf":
                    file_content += f"# {line}\n"
                    file_content += f"50000 {' '.join(line.split(' ')[1:])}\n"
                else:
                    is_allowed = False
                    for e in noise_bias_vec:
                        if abs(float(noise_bias) - e) <= err_tol:
                            is_allowed = True
                            break
                    if is_allowed:
                        file_content += f"{line}\n"
        # print(file_content, end="")
        with open(filename, "w", encoding="utf8") as f:
            f.write(file_content)
