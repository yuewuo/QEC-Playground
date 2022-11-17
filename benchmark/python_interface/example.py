from benchmarker import fetch_noise_model, Position
import os

"""
example using tailored surface code of code distance 3

It customizes the noise model to that the phenomenological noise model in arXiv:1907.02554v2
"""

code_distance = 5
noisy_measurement_rounds = 5

# this parameters only specify the size of the tailored surface code, but no error rate
configuration = ["--code_type", "RotatedTailoredCode"]


# now fetch the noise model with default probability = 0
noise_model = fetch_noise_model(code_distance, code_distance, noisy_measurement_rounds, p=0, configuration=configuration)


# print out a single qubit in the middle to see what field it has, initially all error rate is configured to 0
middle_qubit = noise_model.at(Position(6, 3, 3))
print(middle_qubit)


# compute phenomenological noise model
p = 0.01
bias_eta = 100
px = p / (1. + bias_eta) / 2.
py = px
pz = p - 2. * px
measurement_error_rate = p


# update the noise model
for mt in range(noisy_measurement_rounds):
    t = mt * 6  # add error before each measurement, the first measurement happens at t = 6
    for i in range(noise_model.vertical):
        for j in range(noise_model.horizontal):
            qubit = noise_model.at(Position(t, i, j))
            if qubit is not None and not qubit.is_virtual:  # rotated code will leave some position no qubits
                if qubit.type == "Data":

                    # single-qubit error rate
                    qubit.pX = px
                    qubit.pZ = pz
                    qubit.pY = py

                    # although not used here, you many find it useful in other noise models
                    qubit.pE = 0  # erasure error rate, when erasure happens, a random Pauli I,X,Y,Z is applied to the qubit and the decoder has information of "where" it is
                    if qubit.peer is not None:
                        # correlated two-qubit Pauli errors
                        qubit.pIX = 0
                        qubit.pIZ = 0
                        qubit.pIY = 0
                        qubit.pXI = 0
                        qubit.pXX = 0
                        qubit.pXZ = 0
                        qubit.pXY = 0
                        qubit.pZI = 0
                        qubit.pZX = 0
                        qubit.pZZ = 0
                        qubit.pZY = 0
                        qubit.pYI = 0
                        qubit.pYX = 0
                        qubit.pYZ = 0
                        qubit.pYY = 0

                        # correlated two-qubit erasure errors
                        qubit.pIE = 0
                        qubit.pEI = 0
                        qubit.pEE = 0

                        # you can also check the error rate is indeed set 
                        qubit.pEE = 0.3
                        assert qubit.pEE == 0.3
                        qubit.pEE = 0

                elif qubit.type == "StabX":
                    qubit.pZ = measurement_error_rate  # Z error will flip the measurement result, mimicing measurement error
                elif qubit.type == "StabY":
                    qubit.pZ = measurement_error_rate  # Z error will flip the measurement result, mimicing measurement error
                else:
                    # it's good to output unrecognized qubit type, to reduce bug
                    print(f"[warning] unrecognized qubit at [{t}][{i}][{j}]")


# it's tedious to check your noise model is as expected by just printing them out
# instead, I developed a visualization tool of any noise model
# unfortunately, due to technical simplicity I didn't implement arbitrary sized visualization, so currently the code_distance and measurement_rounds must be 5
# (you can still run benchmark for other code size of course, just without visualization)
if code_distance == 5 and noisy_measurement_rounds == 5:
    # noise_model.visualize()  # by default use web server: https://qec.wuyue98.cn, you can easily share the link with others
    # noise_model.visualize("http://localhost:8066", None)  # if you decide to start local server using `cargo run --release -- server --port 8066`, uncomment this line
    pass
else:
    print("[info] visualization is not supported for code distance and measurement rounds other than 5, skipped")

print(middle_qubit)  # check what's after the change

# you can also save this noise model to a file, and run benchmark using this noise model later
#    it can be loaded using `cargo run --release -- tool benchmark ...... --load_noise_model_from_file /path/to/example.json`
filepath = os.path.join(os.path.dirname(__file__), f"example.json")
noise_model.save(filepath)


# this will automatically create a temporary file for the noise model and then run benchmark
print("\n[info] start benchmarking...\n")
stdout, returncode = noise_model.run_benchmark(max_N=100000, min_error_cases=3000, time_budget=60, verbose=True)  # set a time limit of 1min
print("\n" + stdout)
assert returncode == 0, "command fails..."
