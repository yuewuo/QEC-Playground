use crate::noise_model::*;
use crate::noise_model_builder::*;
use crate::simulator::*;
use crate::types::*;
use std::sync::Arc;

fn is_in_region(d: usize, _t: usize, i: usize, j: usize) -> bool {
    let di = d as isize;
    let (i, j) = (i as isize, j as isize);
    i - j <= di && j - i <= di && i + j >= di && i + j <= 7 * di
}

// assuming the qubit is already in the region, whether it is part of the merging operation
fn is_merging_qubit(d: usize, _t: usize, i: usize, j: usize) -> bool {
    if i + j > 3 * d && i + j < 5 * d {
        return true;
    }
    if i + j == 3 * d || i + j == 5 * d {
        if i % 2 == 0 {
            return true;
        }
    }
    false
}

fn is_second_qubit(d: usize, _t: usize, i: usize, j: usize) -> bool {
    i + j >= 5 * d
}

fn is_present(unit_rounds: usize, d: usize, t: usize, i: usize, j: usize) -> bool {
    if !is_in_region(d, t, i, j) {
        return false;
    }
    let di = d as isize;
    let (t, i, j) = (t as isize, i as isize, j as isize);
    if (i - j == di || j - i == di) && i % 2 == 1 {
        return false;
    }
    if (i + j == di || i + j == 7 * di) && i % 2 == 0 {
        return false;
    }
    // then check for time step: the merging area does not exist except for in the middle
    if is_merging_qubit(d, t as usize, i as usize, j as usize) {
        if (t as usize) <= (unit_rounds - 1) * 6 || (t as usize) > (2 * unit_rounds + 1) * 6 {
            return false;
        }
        // if (t as usize) > 2 * unit_rounds * 6 {
        //     return false;
        // }
    }
    if is_second_qubit(d, t as usize, i as usize, j as usize) {
        if (t as usize) > (2 * unit_rounds + 1) * 6 {
            return false;
        }
    }
    true
}

pub fn build_code(simulator: &mut Simulator) {
    let code_size = &simulator.code_size;
    assert_eq!(code_size.di, code_size.dj);
    let d = code_size.di;
    let noisy_measurements = code_size.noisy_measurements;
    assert!(d % 2 == 1, "code distance must be odd integer, current: d = {}", d);
    assert!(
        noisy_measurements % 3 == 0,
        "noisy measurements must be a multiply of 3, normally 3d, current = {noisy_measurements}"
    );
    let unit_rounds = noisy_measurements / 3;
    simulator.measurement_cycles = 6;
    let (vertical, horizontal) = (4 * d + 1, 4 * d + 1);
    let height = simulator.measurement_cycles * (noisy_measurements + 1) + 1;

    // each measurement takes 6 time steps
    let mut nodes = Vec::with_capacity(height);
    for t in 0..height {
        let mut row_i = Vec::with_capacity(vertical);
        for i in 0..vertical {
            let mut row_j = Vec::with_capacity(horizontal);
            for j in 0..horizontal {
                if is_present(unit_rounds, d, t, i, j) {
                    let qubit_type = if (i + j) % 2 == 0 {
                        QubitType::Data
                    } else if i % 2 == 1 {
                        QubitType::StabZ
                    } else {
                        QubitType::StabX
                    };
                    let mut gate_type = GateType::None;
                    let mut gate_peer = None;
                    match t % simulator.measurement_cycles {
                        1 => {
                            // initialization
                            match qubit_type {
                                QubitType::StabZ => {
                                    gate_type = GateType::InitializeZ;
                                }
                                QubitType::StabX => {
                                    gate_type = GateType::InitializeX;
                                }
                                QubitType::Data => {}
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        2 => {
                            // gate 1
                            if qubit_type == QubitType::Data {
                                if i + 1 < vertical && is_present(unit_rounds, d, t, i + 1, j) {
                                    gate_type = if j % 2 == 1 {
                                        GateType::CXGateTarget
                                    } else {
                                        GateType::CXGateControl
                                    };
                                    gate_peer = Some(pos!(t, i + 1, j));
                                }
                            } else if i >= 1 && is_present(unit_rounds, d, t, i - 1, j) {
                                gate_type = if j % 2 == 1 {
                                    GateType::CXGateControl
                                } else {
                                    GateType::CXGateTarget
                                };
                                gate_peer = Some(pos!(t, i - 1, j));
                            }
                        }
                        3 => {
                            // gate 2
                            if j % 2 == 1 {
                                // operate with right
                                if is_present(unit_rounds, d, t, i, j + 1) {
                                    gate_type = GateType::CXGateControl;
                                    gate_peer = Some(pos!(t, i, j + 1));
                                }
                            } else {
                                // operate with left
                                if j >= 1 && is_present(unit_rounds, d, t, i, j - 1) {
                                    gate_type = GateType::CXGateTarget;
                                    gate_peer = Some(pos!(t, i, j - 1));
                                }
                            }
                        }
                        4 => {
                            // gate 3
                            if j % 2 == 1 {
                                // operate with left
                                if j >= 1 && is_present(unit_rounds, d, t, i, j - 1) {
                                    gate_type = GateType::CXGateControl;
                                    gate_peer = Some(pos!(t, i, j - 1));
                                }
                            } else {
                                // operate with right
                                if is_present(unit_rounds, d, t, i, j + 1) {
                                    gate_type = GateType::CXGateTarget;
                                    gate_peer = Some(pos!(t, i, j + 1));
                                }
                            }
                        }
                        5 => {
                            // gate 4
                            if qubit_type == QubitType::Data {
                                if i >= 1 && is_present(unit_rounds, d, t, i - 1, j) {
                                    gate_type = if j % 2 == 1 {
                                        GateType::CXGateTarget
                                    } else {
                                        GateType::CXGateControl
                                    };
                                    gate_peer = Some(pos!(t, i - 1, j));
                                }
                            } else if i + 1 < vertical && is_present(unit_rounds, d, t, i + 1, j) {
                                gate_type = if j % 2 == 1 {
                                    GateType::CXGateControl
                                } else {
                                    GateType::CXGateTarget
                                };
                                gate_peer = Some(pos!(t, i + 1, j));
                            }
                        }
                        0 => {
                            // measurement
                            match qubit_type {
                                QubitType::StabZ => {
                                    gate_type = GateType::MeasureZ;
                                }
                                QubitType::StabX => {
                                    gate_type = GateType::MeasureX;
                                }
                                QubitType::Data => {}
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                    row_j.push(Some(Box::new(SimulatorNode::new(qubit_type, gate_type, gate_peer.clone()))));
                } else {
                    row_j.push(None);
                }
            }
            row_i.push(row_j);
        }
        nodes.push(row_i)
    }
    simulator.vertical = vertical;
    simulator.horizontal = horizontal;
    simulator.height = height;
    simulator.nodes = nodes;
}

pub fn validate_code(_simulator: &mut Simulator, _correction: &SparseCorrection) -> Option<(bool, bool)> {
    // not implemented
    Some((true, true))
}

pub fn apply_noise_model(
    simulator: &mut Simulator,
    noise_model: &mut NoiseModel,
    noise_model_configuration: &serde_json::Value,
    p: f64,
    bias_eta: f64,
    pe: f64,
) {
    // first apply the default stim noise model
    NoiseModelBuilder::StimNoiseModel.apply(simulator, noise_model, noise_model_configuration, p, bias_eta, pe);
    // then remove some of the noise
    let d = (simulator.vertical - 1) / 4;
    let noisy_measurements = (simulator.height - 1) / simulator.measurement_cycles - 1;
    let unit_rounds = noisy_measurements / 3;
    let noiseless_node = Arc::new(NoiseModelNode::new());
    simulator_iter_real!(simulator, position, node, {
        let _ = node;
        let Position { t, i, j } = position.clone();
        if is_second_qubit(d, t, i, j) {
            if t > 2 * unit_rounds * 6 {
                noise_model.set_node(position, Some(noiseless_node.clone()));
            }
        }
    });
}
