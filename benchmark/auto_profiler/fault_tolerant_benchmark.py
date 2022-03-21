import qecp_profiler as qp

time_budget = 20  # seconds for each test

def main():
    profile_pure_simulation()
    pass

def profile_pure_simulation():
    for d in [5, 15, 25]:
        p = 0.01
        name = f"pure_simulation_d_{d}"
        qp.run_flamegraph_qecp_profile_command(name, ["tool", "fault_tolerant_benchmark", f"[{d}]", f"[{d}]", f"[{p}]", "--bypass_correction", "--max_N", "0", "--min_error_cases", "0", "--time_budget", f"{time_budget}"])


if __name__ == "__main__":
    main()
