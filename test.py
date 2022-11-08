import qecp

di = 7
dj = 5
noisy_measurements = 3
code_type = qecp.CodeType.StandardPlanarCode
builtin_code_info = qecp.BuiltinCodeInformation(noisy_measurements, di+1, dj+1)
simulator = qecp.Simulator(code_type, builtin_code_info)
simulator.set_nodes(qecp.Position(0, 1, 1), qecp.ErrorType.Z)
simulator.propagate_errors()
measurement = simulator.generate_sparse_measurement()
result = measurement.nontrivial
for i in range(len(result)):
    print((list(result)[i].t, list(result)[i].i, list(result)[i].j))
