import qecp_profiler as qp

time_budget = 60  # seconds for each test

command_prefix = ["tool", "fault_tolerant_benchmark"]

def main():
    profile_pure_simulation()
    profile_CSS_decoding()
    pass

def profile_pure_simulation():
    for d in [5, 15, 25]:
        p = 0.01
        name = f"pure_simulation_d_{d}"
        qp.run_flamegraph_qecp_profile_command(name, command_prefix + [f"[{d}]", f"[{d}]", f"[{p}]", "--bypass_correction", "--max_N", "0", "--min_error_cases", "0", "--time_budget", f"{time_budget}"])

def profile_CSS_decoding():
    for p in [0.005, 0.002]:
        for d in [5, 9, 13]:
            name = f"CSS_decoding_d_{d}_p_{p}"
            qp.run_flamegraph_qecp_profile_command(name, command_prefix + [f"[{d}]", f"[{d}]", f"[{p}]", "--max_N", "0", "--min_error_cases", "0", "--time_budget", f"{time_budget}"])

if __name__ == "__main__":
    main()
