import qecp_profiler as qp

time_budget = 60  # seconds for each test

command_prefix = ["tool", "fault_tolerant_benchmark"]
new_command_prefix = ["tool", "benchmark"]

pm = "\u00B1"

def main():
    profile_pure_simulation()
    # profile_CSS_decoding()
    # profile_XZZX_complex_noise_model_decoding()
    pass

def profile_pure_simulation():
    for d in [5, 15, 25]:
        p = 0.01
        name = f"pure_simulation_d_{d}"
        # qp.run_flamegraph_qecp_profile_command(name, command_prefix + [f"[{d}]", f"[{d}]", f"[{p}]", "--bypass_correction", "--max_N", "0", "--min_error_cases", "0", "--time_budget", f"{time_budget}"])
        qp.run_flamegraph_qecp_profile_command(name, new_command_prefix + [f"[{d}]", f"[{d}]", f"[{p}]", "--max_repeats", "0", "--min_failed_cases", "0", "--time_budget", f"{time_budget}"])

        @qp.compare_qecp_profile(name)
        def compare(now, compare):
            now_line = now["stdout"][1]
            compare_line = compare["stdout"][1]
            # print(now_line, compare_line)
            now_N = float(now_line.split(" ")[3])
            compare_N = float(compare_line.split(" ")[3])
            # print(now_N, compare_N)
            now_speed = now_N / time_budget
            compare_speed = compare_N / time_budget
            diff_speed = (now_speed - compare_speed) / compare_speed
            print(f"now speed: {now_speed:.2f}/s, compare speed: {compare_speed:.2f}/s, relative different: {diff_speed * 100:.2f}%")

def profile_CSS_decoding():
    for decoder in ["MWPM", "UF"]:
        for p in [0.005, 0.002]:
            for d in [5, 9, 13]:
                name = f"CSS_decoding_{decoder}_d_{d}_p_{p}"
                command = command_prefix + [f"[{d}]", f"[{d}]", f"[{p}]", "--max_N", "0", "--min_error_cases", "0", "--decoder", f"{decoder}", "--time_budget", f"{time_budget}"]
                if decoder == "UF":
                    command += ["--max_half_weight", "10"]
                qp.run_flamegraph_qecp_profile_command(name, command)

                @qp.compare_qecp_profile(name)
                def compare(now, compare):
                    now_line = now["stdout"][1]
                    compare_line = compare["stdout"][1]
                    # print(now_line, compare_line)
                    now_N = float(now_line.split(" ")[3])
                    compare_N = float(compare_line.split(" ")[3])
                    # print(now_N, compare_N)
                    now_speed = now_N / time_budget
                    compare_speed = compare_N / time_budget
                    diff_speed = (now_speed - compare_speed) / compare_speed
                    print(f"now speed: {now_speed:.2f}/s, compare speed: {compare_speed:.2f}/s, relative different: {diff_speed * 100:.2f}%")
                    now_pL = float(now_line.split(" ")[5])
                    now_stddev = float(now_line.split(" ")[7])
                    compare_pL = float(compare_line.split(" ")[5])
                    compare_stddev = float(compare_line.split(" ")[7])
                    diff_pL = now_pL - compare_pL
                    print(f"now logical error rate: {now_pL:.2e} {pm} {now_stddev*now_pL:.1e}, compare logical error rate: {compare_pL:.2e} {pm} {compare_stddev*compare_pL:.1e}, different: {diff_pL:.2e}")


def profile_XZZX_complex_noise_model_decoding():
    decoder = "UF"
    p = 0.03
    pp = p * 0.02
    pe = p * 0.98
    for d in [5, 9, 13]:
        name = f"XZZX_complex_noise_model_d_{d}"
        command = command_prefix + [f"[{d}]", f"[{d}]", f"[{pp}]", "--pes", f"[{pe}]", "--max_N", "0", "--min_error_cases", "0", "--decoder", f"{decoder}", "--time_budget", f"{time_budget}", "--max_half_weight", "10"]
        command += ["--use_xzzx_code", "--error_model", "OnlyGateErrorCircuitLevelCorrelatedErasure", "--error_model_configuration", "{\"use_correlated_pauli\":true}"]
        qp.run_flamegraph_qecp_profile_command(name, command)

        @qp.compare_qecp_profile(name)
        def compare(now, compare):
            now_line = now["stdout"][1]
            compare_line = compare["stdout"][1]
            # print(now_line, compare_line)
            now_N = float(now_line.split(" ")[3])
            compare_N = float(compare_line.split(" ")[3])
            # print(now_N, compare_N)
            now_speed = now_N / time_budget
            compare_speed = compare_N / time_budget
            diff_speed = (now_speed - compare_speed) / compare_speed
            print(f"now speed: {now_speed:.2f}/s, compare speed: {compare_speed:.2f}/s, relative different: {diff_speed * 100:.2f}%")
            now_pL = float(now_line.split(" ")[5])
            now_stddev = float(now_line.split(" ")[7])
            compare_pL = float(compare_line.split(" ")[5])
            compare_stddev = float(compare_line.split(" ")[7])
            diff_pL = now_pL - compare_pL
            print(f"now logical error rate: {now_pL:.2e} {pm} {now_stddev*now_pL:.1e}, compare logical error rate: {compare_pL:.2e} {pm} {compare_stddev*compare_pL:.1e}, different: {diff_pL:.2e}")


if __name__ == "__main__":
    main()
