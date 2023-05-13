import qecp

d = 5
noisy_measurements = 3
p = 0.05

# build code and its simulator
code_type = qecp.CodeType.RotatedPlanarCode
# code_type = qecp.CodeType.StandardPlanarCode
code_size = qecp.CodeSize(noisy_measurements, d, d)
simulator = qecp.Simulator(code_type, code_size)

# build noise model
noise_model_builder = qecp.NoiseModelBuilder.Phenomenological
noise_model = qecp.NoiseModel(simulator)
noise_model_builder.apply(simulator, noise_model, p)

# visualizer
visualizer = None
if True:  # change to False to disable visualizer for faster decoding
    visualize_filename = qecp.static_visualize_data_filename()
    visualizer = qecp.Visualizer(filepath=visualize_filename)
    visualizer.add_component_simulator(simulator)
    visualizer.add_component_noise_model(noise_model)  # optional
    visualizer.end_component()

for i in range(10):
    error_count, _erasure_count = simulator.generate_random_errors(noise_model)
    print(f"[{i}] generated {error_count} errors")
    error_pattern = simulator.generate_sparse_error_pattern()
    measurement = simulator.generate_sparse_measurement()

    # your correction
    correction = qecp.SparseCorrection()

    # whether it has a logical error or not
    (logical_i, logical_j) = simulator.validate_correction(correction)
    is_qec_failed = logical_i or logical_j

    if visualizer is not None:
        visualizer.add_case({
            "error_pattern": error_pattern.to_json(),
            "measurement": measurement.to_json(),
            "correction": correction.to_json(),
            "is_qec_failed": is_qec_failed,
        })

if visualizer is not None:
    qecp.print_visualize_link(filename=visualize_filename)
    qecp.helper.open_visualizer(visualize_filename, open_browser=True)
