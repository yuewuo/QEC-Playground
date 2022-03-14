from benchmarker import fetch_error_model

"""
example using tailored surface code of code distance 3

It customizes the error model to that the phenomenological error model in arXiv:1907.02554v2
"""

code_distance = 5
measurement_rounds = 5

# this parameters only specify the size of the tailored surface code, but no error rate
configuration = ["--use_rotated_tailored_code"]

# now fetch the error model with default probability = 0
error_model = fetch_error_model(code_distance, code_distance, measurement_rounds, p=0, configuration=configuration)

# print out a single qubit in the middle to see what field it has, initially all error rate is configured to 0
middle_qubit = error_model.at(14, 2, 2)
print(middle_qubit)

# compute phenomenological error model
p = 0.01
bias_eta = 100
px = p / (1. + bias_eta) / 2.
py = px
pz = p - 2. * px
measurement_error_rate = p

# update the error model
for mt in range(measurement_rounds):
    t = 11 + mt * 6  # add error before each measurement, the first measurement happens at t = 12
    for i in range(code_distance * 2 - 1):
        for j in range(code_distance * 2 - 1):
            qubit = error_model.at(t, i, j)
            if qubit is not None:  # rotated code will leave some position no qubits
                if qubit.type == "Data":

                    # single-qubit error rate
                    qubit.pX = px
                    qubit.pZ = pz
                    qubit.pY = py

                    # although not used here, you many find it useful in other error models
                    qubit.pE = 0
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

# it's tedious to check your error model is as expected by just printing them out
# instead, I developed a visualization tool of any error model
# unfortunately, due to technical simplicity I didn't implement arbitrary sized visualization, currently the code_distance and measurement_rounds must be 5
if code_distance == 5 and measurement_rounds == 5:
    # error_model.visualize("https://qec.wuyue98.cn")
    error_model.visualize("http://localhost:8066", None)  # if you decide to start local server using `cargo run --release -- server --port 8066`, uncomment this line
else:
    print("[info] visualization is not supported for code distance and measurement rounds other than 5, skipped")
