import math

for filename in ["raw_data.txt", "raw_data_pure_pauli.txt"]:
    print(filename)
    with open(filename, "r", encoding="utf8") as f:
        lines = f.readlines()
        last = None
        last_last = None
        for line in lines:
            line = line.strip("\r\n ")
            if line == "":
                print("")
                continue
            p, p_pauli, di, T, total_cnt, error_cnt, error_rate, dj, confidence_interval = line.split(" ")
            if last_last is not None:
                this_p = p
                this_di = di
                this_error_rate = error_rate
                this_confidence_interval = confidence_interval
                p, p_pauli, di, T, total_cnt, error_cnt, error_rate, dj, confidence_interval = last_last.split(" ")
                if di == this_di:
                    delta_lnp = math.log(float(p)) - math.log(float(this_p))
                    delta_lne = math.log(float(error_rate)) - math.log(float(this_error_rate))
                    derivative = delta_lne / delta_lnp
                    derivative_uncertainty = (float(this_confidence_interval) + float(confidence_interval)) / delta_lnp / math.sqrt(2)
                    print(last + f" {derivative} {derivative_uncertainty}")
            last_last = last
            last = line
