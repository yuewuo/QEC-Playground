import qecp

di = 7
dj = 5
noisy_measurements = 3
code_type = qecp.CodeType.StandardPlanarCode
code_size = qecp.CodeSize(noisy_measurements, di, dj)
simulator = qecp.Simulator(code_type, code_size)
simulator.set_nodes(qecp.Position(0, 1, 1), qecp.ErrorType.Z)
simulator.propagate_errors()
measurement = simulator.generate_sparse_measurement()
result = measurement.defects
for i in range(len(result)):
    print((list(result)[i].t, list(result)[i].i, list(result)[i].j))
