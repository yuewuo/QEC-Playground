import qecp_profiler as qp

time_budget = 60  # seconds for each test

command_prefix = ["tool", "fault_tolerant_benchmark"]

def main():
    # profile_pure_simulation()
    # profile_CSS_decoding()
    profile_XZZX_complex_noise_model_decoding()
    pass

def profile_pure_simulation():
    for d in [5, 15, 25]:
        p = 0.01
        name = f"pure_simulation_d_{d}"
        qp.run_flamegraph_qecp_profile_command(name, command_prefix + [f"[{d}]", f"[{d}]", f"[{p}]", "--bypass_correction", "--max_N", "0", "--min_error_cases", "0", "--time_budget", f"{time_budget}"])

def profile_CSS_decoding():
    for decoder in ["MWPM", "UF"]:
        for p in [0.005, 0.002]:
            for d in [5, 9, 13]:
                name = f"CSS_decoding_{decoder}_d_{d}_p_{p}"
                command = command_prefix + [f"[{d}]", f"[{d}]", f"[{p}]", "--max_N", "0", "--min_error_cases", "0", "--decoder", f"{decoder}", "--time_budget", f"{time_budget}"]
                if decoder == "UF":
                    command += ["--max_half_weight", "10"]
                qp.run_flamegraph_qecp_profile_command(name, command)

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


if __name__ == "__main__":
    main()
