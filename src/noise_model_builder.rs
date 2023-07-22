//! build customized noise model by giving a name
//!

use super::clap::ValueEnum;
use super::code_builder::*;
use super::noise_model::*;
use super::simulator::*;
use super::types::*;
use super::util_macros::*;
use crate::serde::{Deserialize, Serialize};
#[cfg(feature = "python_binding")]
use pyo3::prelude::*;
use std::collections::BTreeSet;
use std::sync::Arc;

/// commonly used noise models
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub enum NoiseModelBuilder {
    /// add data qubit errors and measurement errors individually
    Phenomenological,
    /// same as phenomenological, but with half unfixed stabilizers
    PhenomenologicalInit,
    /// tailored surface code with Bell state initialization (logical |+> state) to fix 3/4 of all stabilizers
    TailoredScBellInitPhenomenological,
    TailoredScBellInitCircuit,
    /// arXiv:2104.09539v1 Sec.IV.A
    GenericBiasedWithBiasedCX,
    /// arXiv:2104.09539v1 Sec.IV.A
    GenericBiasedWithStandardCX,
    /// 100% erasure errors only on the data qubits before the gates happen and on the ancilla qubits before the measurement
    ErasureOnlyPhenomenological,
    /// errors happen at 4 stages in each measurement round (although removed errors happening at initialization and measurement stage, measurement errors can still occur when curtain error applies on the ancilla after the last gate)
    OnlyGateErrorCircuitLevel,
    /// mixed erasure error and Pauli errors only on the data qubits before the gates happen and on the ancilla qubits before the measurement
    MixedPhenomenological,
    /// Fault-tolerant weighted union-find decoding on the toric code
    DepolarizingNoise,
    /// the noise model in stim: after_clifford_depolarization, before_round_data_depolarization, before_measure_flip_probability, after_reset_flip_probability;
    /// see https://github.com/quantumlib/Stim/blob/main/doc/python_api_reference_vDev.md#stim.Circuit.generated
    StimNoiseModel,
}

#[cfg(feature = "python_binding")]
#[pymethods]
impl NoiseModelBuilder {
    #[pyo3(name = "apply", signature = (simulator, noise_model, p, noise_model_configuration=None, bias_eta=0.5, pe=0.))]
    fn trait_apply(
        &self,
        simulator: &mut Simulator,
        noise_model: &mut NoiseModel,
        p: f64,
        noise_model_configuration: Option<PyObject>,
        bias_eta: f64,
        pe: f64,
    ) {
        let noise_model_configuration = noise_model_configuration
            .map(|v| crate::util::pyobject_to_json(v))
            .unwrap_or(json!({}));
        self.apply(simulator, noise_model, &noise_model_configuration, p, bias_eta, pe)
    }
}

impl NoiseModelBuilder {
    /// apply noise model
    pub fn apply(
        &self,
        simulator: &mut Simulator,
        noise_model: &mut NoiseModel,
        noise_model_configuration: &serde_json::Value,
        p: f64,
        bias_eta: f64,
        pe: f64,
    ) {
        // commonly used biased qubit error node
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        let mut biased_node = NoiseModelNode::new();
        biased_node.pauli_error_rates.error_rate_X = px;
        biased_node.pauli_error_rates.error_rate_Y = py;
        biased_node.pauli_error_rates.error_rate_Z = pz;
        biased_node.erasure_error_rate = pe;
        let biased_node = Arc::new(biased_node);
        // commonly used pure measurement error node
        let mut pm = p;
        if let Some(value) = noise_model_configuration.get("measurement_error_rate") {
            pm = value.as_f64().expect("measurement_error_rate must be `f64`");
        }
        let mut pure_measurement_node = NoiseModelNode::new();
        pure_measurement_node.pauli_error_rates.error_rate_Y = pm; // Y error will cause pure measurement error for StabX (X basis), StabZ (Z basis), StabY (X basis)
        let pure_measurement_node = Arc::new(pure_measurement_node);
        // commonly used noiseless error node
        let noiseless_node = Arc::new(NoiseModelNode::new());
        // noise model builder
        match self {
            Self::Phenomenological => {
                let simulator = &*simulator; // force simulator to be immutable, to avoid unexpected changes
                assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
                assert!(pe == 0.); // phenomenological noise model doesn't support erasure errors
                if simulator.measurement_cycles == 1 {
                    eprintln!("[warning] setting error rates of unknown code, no perfect measurement protection is enabled");
                }
                simulator_iter_real!(simulator, position, node, {
                    noise_model.set_node(position, Some(noiseless_node.clone())); // clear existing noise model
                    if position.t >= simulator.height - simulator.measurement_cycles {
                        // no error at the final perfect measurement round
                        continue;
                    }
                    if position.t % simulator.measurement_cycles == 0 && node.qubit_type == QubitType::Data {
                        noise_model.set_node(position, Some(biased_node.clone()));
                    }
                    if (position.t + 1) % simulator.measurement_cycles == 0 && node.qubit_type != QubitType::Data {
                        // measurement error must happen before measurement round
                        noise_model.set_node(position, Some(pure_measurement_node.clone()));
                    }
                });
            }
            NoiseModelBuilder::PhenomenologicalInit => {
                // let (noisy_measurements, _, _) = match simulator.code_type {
                //     CodeType::RotatedTailoredCode{ noisy_measurements, dp, dn } => { (noisy_measurements, dp, dn) }
                //     _ => unimplemented!("tailored surface code with Bell state initialization is only implemented for open-boundary rotated tailored surface code")
                // };
                let (noisy_measurements, _, _) = match simulator.code_type {
                    CodeType::RotatedTailoredCode => { (simulator.code_size.noisy_measurements, simulator.code_size.di, simulator.code_size.dj) }
                    _ => unimplemented!("tailored surface code with Bell state initialization is only implemented for open-boundary rotated tailored surface code")
                };
                assert!(noisy_measurements > 0, "to simulate bell initialization, noisy measurement must be set +1 (e.g. set noisy measurement 1 is equivalent to 0 noisy measurements)");
                assert!(simulator.measurement_cycles > 1);
                // change all stabilizers at the first round as virtual
                simulator_iter_mut!(simulator, position, node, t => simulator.measurement_cycles, {
                    if node.qubit_type != QubitType::Data {
                        assert!(node.gate_type.is_measurement());
                        assert!(node.gate_type.is_single_qubit_gate());
                        // since no peer, just set myself as virtual is ok
                        node.is_virtual = true;
                        noise_model.set_node(position, Some(noiseless_node.clone()));  // clear existing noise model
                    }
                });
                let simulator = &*simulator; // force simulator to be immutable, to avoid unexpected changes
                assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
                assert!(pe == 0.); // phenomenological error model doesn't support erasure errors
                if simulator.measurement_cycles == 1 {
                    eprintln!("[warning] setting error rates of unknown code, no perfect measurement protection is enabled");
                }
                // create an error model that is always 50% change of measurement error
                let mut messed_measurement_node = NoiseModelNode::new();
                messed_measurement_node.pauli_error_rates.error_rate_Y = 0.5; // Y error will cause pure measurement error for StabX (X basis), StabZ (Z basis), StabY (X basis)
                let messed_measurement_node = Arc::new(messed_measurement_node);
                simulator_iter_real!(simulator, position, node, {
                    noise_model.set_node(position, Some(noiseless_node.clone())); // clear existing noise model
                    if position.t == simulator.measurement_cycles - 1 && node.qubit_type == QubitType::StabY {
                        noise_model.set_node(position, Some(messed_measurement_node.clone()))
                    } else if position.t >= simulator.measurement_cycles {
                        // no error before the first round
                        if position.t < simulator.height - simulator.measurement_cycles {
                            // no error at the final perfect measurement round
                            if position.t % simulator.measurement_cycles == 0 && node.qubit_type == QubitType::Data {
                                noise_model.set_node(position, Some(biased_node.clone()));
                            }
                            if (position.t + 1) % simulator.measurement_cycles == 0 && node.qubit_type != QubitType::Data {
                                // measurement error must happen before measurement round
                                noise_model.set_node(position, Some(pure_measurement_node.clone()));
                            }
                        }
                    }
                });
            }
            NoiseModelBuilder::TailoredScBellInitPhenomenological => {
                let (noisy_measurements, dp, dn) = match simulator.code_type {
                    CodeType::RotatedTailoredCode => { (simulator.code_size.noisy_measurements, simulator.code_size.di, simulator.code_size.dj) }
                    _ => unimplemented!("tailored surface code with Bell state initialization is only implemented for open-boundary rotated tailored surface code")
                };
                assert!(noisy_measurements > 0, "to simulate bell initialization, noisy measurement must be set +1 (e.g. set noisy measurement 1 is equivalent to 0 noisy measurements)");
                assert!(simulator.measurement_cycles > 1);
                // change all stabilizers at the first round as virtual
                simulator_iter_mut!(simulator, position, node, t => simulator.measurement_cycles, {
                    if node.qubit_type != QubitType::Data {
                        assert!(node.gate_type.is_measurement());
                        assert!(node.gate_type.is_single_qubit_gate());
                        // since no peer, just set myself as virtual is ok
                        node.is_virtual = true;
                        noise_model.set_node(position, Some(noiseless_node.clone()));  // clear existing noise model
                    }
                    noise_model.set_node(position, Some(noiseless_node.clone()));
                });
                let simulator = &*simulator; // force simulator to be immutable, to avoid unexpected changes
                assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
                assert!(pe == 0.); // phenomenological noise model doesn't support erasure errors
                if simulator.measurement_cycles == 1 {
                    eprintln!("[warning] setting error rates of unknown code, no perfect measurement protection is enabled");
                }
                // create an noise model that is always 50% change of measurement error
                let mut messed_measurement_node = NoiseModelNode::new();
                messed_measurement_node.pauli_error_rates.error_rate_Y = 0.5; // Y error will cause pure measurement error for StabX (X basis), StabZ (Z basis), StabY (X basis)
                let messed_measurement_node = Arc::new(messed_measurement_node);
                simulator_iter_real!(simulator, position, node, {
                    noise_model.set_node(position, Some(noiseless_node.clone())); // clear existing noise model
                    if position.t == simulator.measurement_cycles - 1 {
                        for i in 0..((dn + 1) / 2 - 1) {
                            for j in 0..(dp + 1) / 2 {
                                // println!("{:?} {:?} {:?}", position.t, 3 + 2*i + 2*j, dn-1 - 2*i + 2*j);
                                noise_model.set_node(
                                    &pos!(position.t, 3 + 2 * i + 2 * j, dn - 1 - 2 * i + 2 * j),
                                    Some(messed_measurement_node.clone()),
                                );
                            }
                        }
                    } else if position.t >= simulator.measurement_cycles {
                        // no error before the first round
                        if position.t < simulator.height - simulator.measurement_cycles {
                            // no error at the final perfect measurement round
                            if position.t % simulator.measurement_cycles == 0 && node.qubit_type == QubitType::Data {
                                noise_model.set_node(position, Some(biased_node.clone()));
                            }
                            if (position.t + 1) % simulator.measurement_cycles == 0 && node.qubit_type != QubitType::Data {
                                // measurement error must happen before measurement round
                                noise_model.set_node(position, Some(pure_measurement_node.clone()));
                            }
                        }
                    }
                });
            }
            Self::GenericBiasedWithBiasedCX | Self::GenericBiasedWithStandardCX => {
                // (here) FIRST qubit: anc; SECOND: data, due to circuit design
                let mut initialization_error_rate = p; // by default initialization error rate is the same as p
                let mut measurement_error_rate = p;
                let mut config_cloned = noise_model_configuration.clone();
                let config = config_cloned
                    .as_object_mut()
                    .expect("noise_model_configuration must be JSON object");
                if let Some(value) = config.remove("initialization_error_rate") {
                    initialization_error_rate = value.as_f64().expect("f64")
                }
                if let Some(value) = config.remove("measurement_error_rate") {
                    measurement_error_rate = value.as_f64().expect("f64");
                }
                if !config.is_empty() {
                    panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>());
                }
                // normal biased node
                let mut normal_biased_node = NoiseModelNode::new();
                normal_biased_node.pauli_error_rates.error_rate_X = initialization_error_rate / bias_eta;
                normal_biased_node.pauli_error_rates.error_rate_Z = initialization_error_rate;
                normal_biased_node.pauli_error_rates.error_rate_Y = initialization_error_rate / bias_eta;
                let normal_biased_node = Arc::new(normal_biased_node);
                // CZ gate node
                let mut cphase_node = NoiseModelNode::new();
                cphase_node.correlated_pauli_error_rates =
                    Some(CorrelatedPauliErrorRates::default_with_probability(p / bias_eta));
                cphase_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZI = p;
                cphase_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_IZ = p;
                let cphase_node = Arc::new(cphase_node);
                // CZ gate with measurement error
                let mut cphase_measurement_error_node: NoiseModelNode = (*cphase_node).clone();
                cphase_measurement_error_node.pauli_error_rates.error_rate_X = measurement_error_rate / bias_eta;
                cphase_measurement_error_node.pauli_error_rates.error_rate_Z = measurement_error_rate;
                cphase_measurement_error_node.pauli_error_rates.error_rate_Y = measurement_error_rate / bias_eta;
                let cphase_measurement_error_node = Arc::new(cphase_measurement_error_node);
                // CX gate node
                let mut cx_node = NoiseModelNode::new();
                cx_node.correlated_pauli_error_rates =
                    Some(CorrelatedPauliErrorRates::default_with_probability(p / bias_eta));
                cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZI = p;
                match self {
                    Self::GenericBiasedWithStandardCX => {
                        cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_IZ = 0.375 * p;
                        cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZZ = 0.375 * p;
                        cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_IY = 0.125 * p;
                        cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZY = 0.125 * p;
                    }
                    Self::GenericBiasedWithBiasedCX => {
                        cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_IZ = 0.5 * p;
                        cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZZ = 0.5 * p;
                    }
                    _ => {}
                }
                let cx_node = Arc::new(cx_node);
                // CX gate with measurement error
                let mut cx_measurement_error_node: NoiseModelNode = (*cx_node).clone();
                cx_measurement_error_node.pauli_error_rates.error_rate_X = measurement_error_rate / bias_eta;
                cx_measurement_error_node.pauli_error_rates.error_rate_Z = measurement_error_rate;
                cx_measurement_error_node.pauli_error_rates.error_rate_Y = measurement_error_rate / bias_eta;
                let cx_measurement_error_node = Arc::new(cx_measurement_error_node);
                // iterate over all nodes
                simulator_iter_real!(simulator, position, node, {
                    // first clear error rate
                    noise_model.set_node(position, Some(noiseless_node.clone()));
                    if position.t >= simulator.height - simulator.measurement_cycles {
                        // no error on the top, as a perfect measurement round
                        continue;
                    }
                    // do different things for each stage
                    match position.t % simulator.measurement_cycles {
                        1 => {
                            // initialization
                            noise_model.set_node(position, Some(normal_biased_node.clone()));
                        }
                        0 => { // measurement
                             // do nothing
                        }
                        _ => {
                            let has_measurement_error = position.t % simulator.measurement_cycles
                                == simulator.measurement_cycles - 1
                                && node.qubit_type != QubitType::Data;
                            match node.gate_type {
                                GateType::CZGate => {
                                    if node.qubit_type != QubitType::Data {
                                        // this is ancilla
                                        // better check whether peer is indeed data qubit, but it's hard here due to Rust's borrow check
                                        noise_model.set_node(
                                            position,
                                            Some(if has_measurement_error {
                                                cphase_measurement_error_node.clone()
                                            } else {
                                                cphase_node.clone()
                                            }),
                                        );
                                    }
                                }
                                GateType::CXGateControl => {
                                    // this is ancilla in XZZX code, see arXiv:2104.09539v1
                                    noise_model.set_node(
                                        position,
                                        Some(if has_measurement_error {
                                            cx_measurement_error_node.clone()
                                        } else {
                                            cx_node.clone()
                                        }),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                });
            }
            Self::TailoredScBellInitCircuit => {
                let CodeSize { noisy_measurements, di: dp, dj: _dn } = match simulator.code_type {
                    CodeType::RotatedTailoredCodeBellInit => { simulator.code_size.clone() }
                    _ => unimplemented!("tailored surface code with Bell state initialization is only implemented for open-boundary rotated tailored surface code")
                };
                assert!(noisy_measurements > 0, "to simulate bell initialization, noisy measurement must be set +1 (e.g. set noisy measurement 1 is equivalent to 0 noisy measurements)");
                assert!(simulator.measurement_cycles > 1);
                // a bunch of function for determining qubit type during init, copied from code_builder.rs
                let (di, dj) = (dp, dp);
                let is_real = |i: usize, j: usize| -> bool {
                    let is_real_dj = |pi, pj| pi + pj < dj || (pi + pj == dj && pi % 2 == 0 && pi > 0);
                    let is_real_di = |pi, pj| pi + pj < di || (pi + pj == di && pj % 2 == 0 && pj > 0);
                    if i <= dj && j <= dj {
                        is_real_dj(dj - i, dj - j)
                    } else if i >= di && j >= di {
                        is_real_dj(i - di, j - di)
                    } else if i >= dj && j <= di {
                        is_real_di(i - dj, di - j)
                    } else if i <= di && j >= dj {
                        is_real_di(di - i, j - dj)
                    } else {
                        unreachable!()
                    }
                };
                // some criteria for bell init
                let is_bell_init_anc = |i: usize, j: usize| -> bool {
                    is_real(i, j) && i < j + dj - 3 && ((i % 4 == 1 && j % 4 == 0) || (i % 4 == 3 && j % 4 == 2))
                };
                let is_bell_init_top = |i: usize, j: usize| -> bool {
                    is_real(i, j) && i < j + dj - 1 && ((i % 4 == 0 && j % 4 == 0) || (i % 4 == 2 && j % 4 == 2))
                };
                let is_bell_init_left = |i: usize, j: usize| -> bool {
                    is_real(i, j) && i < j + dj - 1 && ((i % 4 == 1 && j % 4 == 3) || (i % 4 == 3 && j % 4 == 1))
                };
                let is_bell_init_right = |i: usize, j: usize| -> bool {
                    is_real(i, j) && i < j + dj - 1 && ((i % 4 == 1 && j % 4 == 1) || (i % 4 == 3 && j % 4 == 3))
                };
                let is_bell_init_bot = |i: usize, j: usize| -> bool {
                    is_real(i, j) && i < j + dj - 1 && ((i % 4 == 2 && j % 4 == 0) || (i % 4 == 0 && j % 4 == 2))
                };
                let is_bell_init_unfixed = |i: usize, j: usize| -> bool {
                    is_real(i, j) && ((i % 4 == 3 && j % 4 == 0) || (i % 4 == 1 && j % 4 == 2))
                };

                ////Error nodes for XY code
                let initialization_error_rate = p;
                // normal bias nodes
                let mut normal_biased_node = NoiseModelNode::new();
                normal_biased_node.pauli_error_rates.error_rate_X = initialization_error_rate / bias_eta;
                normal_biased_node.pauli_error_rates.error_rate_Z = initialization_error_rate;
                normal_biased_node.pauli_error_rates.error_rate_Y = initialization_error_rate / bias_eta;
                let normal_biased_node = Arc::new(normal_biased_node);

                // normal bias + cx node (for init)
                let mut normal_biased_with_cx_node = (*normal_biased_node).clone();
                normal_biased_with_cx_node.correlated_pauli_error_rates =
                    Some(CorrelatedPauliErrorRates::default_with_probability(p / bias_eta));
                normal_biased_with_cx_node
                    .correlated_pauli_error_rates
                    .as_mut()
                    .unwrap()
                    .error_rate_ZI = p;
                normal_biased_with_cx_node
                    .correlated_pauli_error_rates
                    .as_mut()
                    .unwrap()
                    .error_rate_IZ = 0.5 * p;
                normal_biased_with_cx_node
                    .correlated_pauli_error_rates
                    .as_mut()
                    .unwrap()
                    .error_rate_ZZ = 0.5 * p;
                let normal_biased_with_cx_node = Arc::new(normal_biased_with_cx_node);

                // biased CX gate node; CX & CY have same noise model if using bias-preserving gate
                let mut cx_node = NoiseModelNode::new();
                cx_node.correlated_pauli_error_rates =
                    Some(CorrelatedPauliErrorRates::default_with_probability(p / bias_eta));
                cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZI = p;
                cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_IZ = 0.5 * p;
                cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZZ = 0.5 * p;
                let cx_node = Arc::new(cx_node);

                // reversed CX gate node, for convinience
                let mut rev_cx_node = NoiseModelNode::new();
                rev_cx_node.correlated_pauli_error_rates =
                    Some(CorrelatedPauliErrorRates::default_with_probability(p / bias_eta));
                rev_cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_IZ = p;
                rev_cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZI = 0.5 * p;
                rev_cx_node.correlated_pauli_error_rates.as_mut().unwrap().error_rate_ZZ = 0.5 * p;
                let rev_cx_node = Arc::new(rev_cx_node);

                // CX gate with measurement error
                let mut cx_measurement_error_node: NoiseModelNode = (*cx_node).clone();
                cx_measurement_error_node.pauli_error_rates.error_rate_X = initialization_error_rate / bias_eta;
                cx_measurement_error_node.pauli_error_rates.error_rate_Z = initialization_error_rate;
                cx_measurement_error_node.pauli_error_rates.error_rate_Y = initialization_error_rate / bias_eta;
                let cx_measurement_error_node = Arc::new(cx_measurement_error_node);

                let simulator = &*simulator; // force simulator to be immutable, to avoid unexpected changes
                assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
                assert!(pe == 0.); // phenomenological noise model doesn't support erasure errors
                if simulator.measurement_cycles == 1 {
                    eprintln!("[warning] setting error rates of unknown code, no perfect measurement protection is enabled");
                }
                // create an noise model that is always 50% change of measurement error
                let mut messed_measurement_node = NoiseModelNode::new();
                messed_measurement_node.pauli_error_rates.error_rate_Z = 0.5; // Z error will cause pure measurement error for unfixed stabilizer(Y)
                let messed_measurement_node = Arc::new(messed_measurement_node);

                simulator_iter_real!(simulator, position, node, {
                    noise_model.set_node(position, Some(noiseless_node.clone())); // clear existing noise model
                    if position.t > 0 && position.t <= simulator.measurement_cycles {
                        // first measurement_cycle is empty, used to set a perfect measurement
                        let (i, j) = (position.i, position.j);
                        assert!(is_real(i, j), "sim_iter_real should iter over real right?");
                        match position.t {
                            1 => {
                                // if is_bell_init_anc: normal+cx
                                // else: normal
                                if is_bell_init_anc(i, j) && is_bell_init_top(i - 1, j) {
                                    noise_model.set_node(position, Some(normal_biased_with_cx_node.clone()));
                                } else {
                                    noise_model.set_node(position, Some(normal_biased_node.clone()));
                                }
                            }
                            2 => {
                                // if is_bell_init_anc: cx
                                if is_bell_init_anc(i, j) && is_bell_init_left(i, j - 1) {
                                    noise_model.set_node(position, Some(cx_node.clone()));
                                }
                            }
                            3 => {
                                // if is_bell_init_anc: cx
                                if is_bell_init_anc(i, j) && is_bell_init_right(i, j + 1) {
                                    noise_model.set_node(position, Some(cx_node.clone()));
                                }
                            }
                            4 => {
                                // if is_bell_init_anc: cx
                                if is_bell_init_anc(i, j) && is_bell_init_bot(i + 1, j) {
                                    noise_model.set_node(position, Some(cx_node.clone()));
                                }
                            }
                            5 => {
                                // if is_bell_init_anc: rev_cx
                                if is_bell_init_anc(i, j) && is_bell_init_bot(i + 1, j) {
                                    noise_model.set_node(position, Some(rev_cx_node.clone()));
                                }
                            }
                            0 => {
                                // if is_bell_init_anc: cx
                                // if is_bell_init_unfixed: z
                                if is_bell_init_anc(i, j) && is_bell_init_bot(i + 1, j) {
                                    noise_model.set_node(position, Some(cx_measurement_error_node.clone()));
                                }
                                if is_bell_init_unfixed(i, j) {
                                    noise_model.set_node(position, Some(messed_measurement_node.clone()));
                                }
                            }
                            _ => {
                                //nothing
                            }
                        }
                    } else if position.t < simulator.height - simulator.measurement_cycles {
                        // no error before the first round and at final round
                        // do different things for each stage
                        match position.t % simulator.measurement_cycles {
                            1 => {
                                // pauli error on qubits
                                noise_model.set_node(position, Some(normal_biased_node.clone()));
                            }
                            0 => { // measurement
                                 // do nothing
                            }
                            _ => {
                                // gate things
                                let has_measurement_error = position.t % simulator.measurement_cycles
                                    == simulator.measurement_cycles - 1
                                    && node.qubit_type != QubitType::Data; // && position.t < (noisy_measurements - 2) * simulator.measurement_cycles - 2;
                                                                           // println!("position.t: {:?}; err: {:?}", position.t, has_measurement_error);
                                if (node.gate_type == GateType::CXGateControl || node.gate_type == GateType::CYGateControl)
                                    && node.qubit_type != QubitType::Data
                                {
                                    //an ancilla
                                    noise_model.set_node(
                                        position,
                                        Some(if has_measurement_error {
                                            cx_measurement_error_node.clone()
                                        } else {
                                            cx_node.clone()
                                        }),
                                    )
                                }
                            }
                        }
                    }
                });
            }
            Self::ErasureOnlyPhenomenological => {
                assert_eq!(p, 0., "pauli error should be 0 in this noise model");
                let mut erasure_node = NoiseModelNode::new();
                // erasure node must have some non-zero pauli error rate for the decoder to work properly
                erasure_node.pauli_error_rates.error_rate_X = 1e-300; // f64::MIN_POSITIVE ~= 2.22e-308
                erasure_node.pauli_error_rates.error_rate_Z = 1e-300;
                erasure_node.pauli_error_rates.error_rate_Y = 1e-300;
                erasure_node.erasure_error_rate = pe;
                let erasure_node = Arc::new(erasure_node);
                // iterate over all nodes
                simulator_iter_real!(simulator, position, node, {
                    // first clear error rate
                    noise_model.set_node(position, Some(noiseless_node.clone()));
                    if position.t >= simulator.height - simulator.measurement_cycles {
                        // no error on the top, as a perfect measurement round
                        continue;
                    }
                    if position.t % simulator.measurement_cycles == 0 {
                        // add data qubit erasure at the beginning
                        if node.qubit_type == QubitType::Data {
                            noise_model.set_node(position, Some(erasure_node.clone()));
                        }
                    } else if position.t % simulator.measurement_cycles == simulator.measurement_cycles - 1 {
                        // the round before measurement, add erasures
                        if node.qubit_type != QubitType::Data {
                            noise_model.set_node(position, Some(erasure_node.clone()));
                        }
                    }
                });
            }
            Self::MixedPhenomenological => {
                let mut noise_node = biased_node.as_ref().clone();
                // erasure node must have some non-zero pauli error rate for the decoder to work properly
                if p == 0. && pe != 0. {
                    noise_node.pauli_error_rates.error_rate_X = 1e-300; // f64::MIN_POSITIVE ~= 2.22e-308
                    noise_node.pauli_error_rates.error_rate_Z = 1e-300;
                    noise_node.pauli_error_rates.error_rate_Y = 1e-300;
                }
                let noise_node = Arc::new(noise_node);
                // iterate over all nodes
                simulator_iter_real!(simulator, position, node, {
                    // first clear error rate
                    noise_model.set_node(position, Some(noiseless_node.clone()));
                    if position.t >= simulator.height - simulator.measurement_cycles {
                        // no error on the top, as a perfect measurement round
                        continue;
                    }
                    if position.t % simulator.measurement_cycles == 0 {
                        // add data qubit erasure at the beginning
                        if node.qubit_type == QubitType::Data {
                            noise_model.set_node(position, Some(noise_node.clone()));
                        }
                    } else if position.t % simulator.measurement_cycles == simulator.measurement_cycles - 1 {
                        // the round before measurement, add erasures
                        if node.qubit_type != QubitType::Data {
                            noise_model.set_node(position, Some(noise_node.clone()));
                        }
                    }
                });
            }
            Self::OnlyGateErrorCircuitLevel => {
                assert_eq!(bias_eta, 0.5, "bias not supported yet, please use the default value 0.5");
                let mut initialization_error_rate = 0.;
                let mut measurement_error_rate = 0.;
                let mut use_correlated_erasure = false;
                let mut use_correlated_pauli = false;
                let mut before_pauli_bug_fix = false;
                let mut erasure_delay_cycle = 0;
                let mut config_cloned = noise_model_configuration.clone();
                let config = config_cloned
                    .as_object_mut()
                    .expect("noise_model_configuration must be JSON object");
                if let Some(value) = config.remove("initialization_error_rate") {
                    initialization_error_rate = value.as_f64().expect("f64");
                }
                if let Some(value) = config.remove("measurement_error_rate") {
                    measurement_error_rate = value.as_f64().expect("f64");
                }
                if let Some(value) = config.remove("use_correlated_erasure") {
                    use_correlated_erasure = value.as_bool().expect("bool");
                }
                if let Some(value) = config.remove("use_correlated_pauli") {
                    use_correlated_pauli = value.as_bool().expect("bool");
                }
                if let Some(value) = config.remove("before_pauli_bug_fix") {
                    before_pauli_bug_fix = value.as_bool().expect("bool");
                }
                if let Some(value) = config.remove("erasure_delay_cycle") {
                    // erasures that are not corrected immediately, instead an erasure may stay
                    // for `delay_cycle` cycles and all qubits that are related will be effected.
                    erasure_delay_cycle = value.as_u64().expect("u64") as usize;
                }
                if !config.is_empty() {
                    panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>());
                }
                // initialization node
                let mut initialization_node = NoiseModelNode::new();
                initialization_node.pauli_error_rates.error_rate_X = initialization_error_rate / 3.;
                initialization_node.pauli_error_rates.error_rate_Z = initialization_error_rate / 3.;
                initialization_node.pauli_error_rates.error_rate_Y = initialization_error_rate / 3.;
                if erasure_delay_cycle > 0 {
                    initialization_node.erasure_error_rate = 1e-300;
                }
                let initialization_node = Arc::new(initialization_node);
                // noiseless node
                let mut erasure_noiseless_node = noiseless_node.clone();
                if erasure_delay_cycle > 0 {
                    // otherwise erasure graph will not contain enough information
                    let mut erasure_noiseless = NoiseModelNode::new();
                    erasure_noiseless.erasure_error_rate = 1e-300;
                    erasure_noiseless_node = Arc::new(erasure_noiseless);
                }
                // iterate over all nodes
                simulator_iter_real!(simulator, position, node, {
                    // first clear error rate
                    noise_model.set_node(position, Some(noiseless_node.clone()));
                    if position.t >= simulator.height - simulator.measurement_cycles {
                        // no error on the top, as a perfect measurement round
                        continue;
                    }
                    noise_model.set_node(position, Some(erasure_noiseless_node.clone()));
                    // do different things for each stage
                    match position.t % simulator.measurement_cycles {
                        1 => {
                            // initialization
                            if node.qubit_type != QubitType::Data {
                                noise_model.set_node(position, Some(initialization_node.clone()));
                            }
                        }
                        0 => { // measurement
                             // do nothing
                        }
                        _ => {
                            // errors everywhere
                            let mut this_position_use_correlated_pauli = false;
                            let mut error_node = NoiseModelNode::new(); // it's perfectly fine to instantiate an error node for each node: just memory inefficient at large code distances
                            if use_correlated_pauli
                                && node.gate_type.is_two_qubit_gate()
                                && node.qubit_type != QubitType::Data
                            {
                                // this is ancilla
                                this_position_use_correlated_pauli = use_correlated_pauli;
                            }

                            if erasure_delay_cycle > 0 {
                                error_node.erasure_error_rate = 1e-300; // single erasure exists, but just never triggered; for decoders
                                let mut erased_qubits = BTreeSet::new();
                                if use_correlated_erasure {
                                    if node.gate_type.is_two_qubit_gate() && node.qubit_type != QubitType::Data {
                                        let gate_peer = node.gate_peer.as_ref().unwrap();
                                        erased_qubits.insert((position.i, position.j));
                                        erased_qubits.insert((gate_peer.i, gate_peer.j));
                                    }
                                } else {
                                    erased_qubits.insert((position.i, position.j));
                                }
                                if !erased_qubits.is_empty() {
                                    let mut erasures = SparseErasures::new();
                                    let t = position.t;
                                    for dt in 0..erasure_delay_cycle + 1 {
                                        for &(i, j) in erased_qubits.iter() {
                                            erasures.insert_erasure(&pos!(t + dt, i, j));
                                        }
                                        if dt == erasure_delay_cycle {
                                            break;
                                        }
                                        // calculate what are the effected qubits in the next round
                                        let nt = t + dt + 1;
                                        if nt >= simulator.height - simulator.measurement_cycles {
                                            break;
                                        }
                                        let mut next_erased_qubits = BTreeSet::new();
                                        for &(i, j) in erased_qubits.iter() {
                                            let next_node = simulator.get_node_unwrap(&pos!(nt, i, j));
                                            if !next_node.gate_type.is_initialization() {
                                                next_erased_qubits.insert((i, j));
                                            }
                                            if next_node.gate_type.is_two_qubit_gate() && !next_node.is_peer_virtual {
                                                let gate_peer = next_node.gate_peer.as_ref().unwrap();
                                                next_erased_qubits.insert((gate_peer.i, gate_peer.j));
                                            }
                                        }
                                        erased_qubits = next_erased_qubits;
                                    }
                                    noise_model.additional_noise.push(AdditionalNoise {
                                        probability: pe,
                                        pauli_errors: SparseErrorPattern::new(),
                                        erasures,
                                    })
                                }
                            } else if use_correlated_erasure
                                && node.gate_type.is_two_qubit_gate()
                                && node.qubit_type != QubitType::Data
                            {
                                // this is ancilla
                                // better check whether peer is indeed data qubit, but it's hard here due to Rust's borrow check
                                let mut correlated_erasure_error_rates =
                                    CorrelatedErasureErrorRates::default_with_probability(0.);
                                correlated_erasure_error_rates.error_rate_EE = pe;
                                correlated_erasure_error_rates.sanity_check();
                                error_node.correlated_erasure_error_rates = Some(correlated_erasure_error_rates);
                            } else {
                                error_node.erasure_error_rate = pe;
                            }

                            // this bug is hard to find without visualization tool...
                            // so I develop such a tool at https://qec.wuyue98.cn/NoiseModelViewer2D.html
                            // to compare: (in url, %20 is space, %22 is double quote)
                            //     https://qec.wuyue98.cn/NoiseModelViewer2D.html?p=0.01&pe=0.05&parameters=--code_type%20StandardXZZXCode%20--noise_model%20only-gate-error-circuit-level%20--noise_model_configuration%20%27{"use_correlated_pauli":true,"use_correlated_erasure":true}%27
                            //     https://qec.wuyue98.cn/NoiseModelViewer2D.html?p=0.01&pe=0.05&parameters=--code_type%20StandardXZZXCode%20--noise_model%20only-gate-error-circuit-level%20--noise_model_configuration%20%27{"use_correlated_pauli":true,"use_correlated_erasure":true,"before_pauli_bug_fix":true}%27
                            let mut px_py_pz = if before_pauli_bug_fix {
                                if this_position_use_correlated_pauli {
                                    (0., 0., 0.)
                                } else {
                                    (p / 3., p / 3., p / 3.)
                                }
                            } else if use_correlated_pauli {
                                (0., 0., 0.)
                            } else {
                                (p / 3., p / 3., p / 3.)
                            };
                            if position.t % simulator.measurement_cycles == simulator.measurement_cycles - 1
                                && node.qubit_type != QubitType::Data
                            {
                                // add additional measurement error
                                // whether it's X axis measurement or Z axis measurement, the additional error rate is always `measurement_error_rate`
                                px_py_pz = ErrorType::combine_probability(
                                    px_py_pz,
                                    (
                                        measurement_error_rate / 2.,
                                        measurement_error_rate / 2.,
                                        measurement_error_rate / 2.,
                                    ),
                                );
                            }
                            let (px, py, pz) = px_py_pz;
                            error_node.pauli_error_rates.error_rate_X = px;
                            error_node.pauli_error_rates.error_rate_Y = py;
                            error_node.pauli_error_rates.error_rate_Z = pz;
                            if pe > 0. {
                                // need to set minimum pauli error when this is subject to erasure
                                if error_node.pauli_error_rates.error_rate_X == 0. {
                                    error_node.pauli_error_rates.error_rate_X = 1e-300;
                                    // f64::MIN_POSITIVE ~= 2.22e-308
                                }
                                if error_node.pauli_error_rates.error_rate_Y == 0. {
                                    error_node.pauli_error_rates.error_rate_Y = 1e-300;
                                    // f64::MIN_POSITIVE ~= 2.22e-308
                                }
                                if error_node.pauli_error_rates.error_rate_Z == 0. {
                                    error_node.pauli_error_rates.error_rate_Z = 1e-300;
                                    // f64::MIN_POSITIVE ~= 2.22e-308
                                }
                            }
                            if this_position_use_correlated_pauli {
                                let correlated_pauli_error_rates =
                                    CorrelatedPauliErrorRates::default_with_probability(p / 15.); // 15 possible errors equally probable
                                correlated_pauli_error_rates.sanity_check();
                                error_node.correlated_pauli_error_rates = Some(correlated_pauli_error_rates);
                            }
                            noise_model.set_node(position, Some(Arc::new(error_node)));
                        }
                    }
                });
            }
            Self::StimNoiseModel => {
                let mut after_clifford_depolarization = p;
                let mut before_round_data_depolarization = p;
                let mut before_measure_flip_probability = p;
                let mut after_reset_flip_probability = p;
                let mut config_cloned = noise_model_configuration.clone();
                let config = config_cloned
                    .as_object_mut()
                    .expect("noise_model_configuration must be JSON object");
                if let Some(value) = config.remove("after_clifford_depolarization") {
                    after_clifford_depolarization = value.as_f64().expect("f64")
                }
                if let Some(value) = config.remove("before_round_data_depolarization") {
                    before_round_data_depolarization = value.as_f64().expect("f64");
                }
                if let Some(value) = config.remove("before_measure_flip_probability") {
                    before_measure_flip_probability = value.as_f64().expect("f64")
                }
                if let Some(value) = config.remove("after_reset_flip_probability") {
                    after_reset_flip_probability = value.as_f64().expect("f64")
                }
                if !config.is_empty() {
                    panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>());
                }
                // correlated depolarize_2 node
                let mut depolarize_2_node = NoiseModelNode::new();
                let correlated_pauli_error_rates =
                    CorrelatedPauliErrorRates::default_with_probability(after_clifford_depolarization / 15.); // 15 possible errors equally probable
                correlated_pauli_error_rates.sanity_check();
                depolarize_2_node.correlated_pauli_error_rates = Some(correlated_pauli_error_rates);
                let depolarize_2_node = Arc::new(depolarize_2_node);
                // data qubit before round depolarization node
                let mut data_qubit_depolarize_node = NoiseModelNode::new();
                data_qubit_depolarize_node.pauli_error_rates.error_rate_X = before_round_data_depolarization / 3.;
                data_qubit_depolarize_node.pauli_error_rates.error_rate_Y = before_round_data_depolarization / 3.;
                data_qubit_depolarize_node.pauli_error_rates.error_rate_Z = before_round_data_depolarization / 3.;
                let data_qubit_depolarize_node = Arc::new(data_qubit_depolarize_node);
                // measurement flip node: whatever basis is the stabilizer, there is always `before_measure_flip_probability` probability to be flipped
                let mut measure_flip_node = NoiseModelNode::new();
                measure_flip_node.pauli_error_rates.error_rate_X = before_measure_flip_probability / 2.;
                measure_flip_node.pauli_error_rates.error_rate_Y = before_measure_flip_probability / 2.;
                measure_flip_node.pauli_error_rates.error_rate_Z = before_measure_flip_probability / 2.;
                let measure_flip_node = Arc::new(measure_flip_node);
                // reset flip node: whatever basis is the stabilizer, there is always `after_reset_flip_probability` probability to be flipped
                let mut reset_flip_node = NoiseModelNode::new();
                reset_flip_node.pauli_error_rates.error_rate_X = after_reset_flip_probability / 2.;
                reset_flip_node.pauli_error_rates.error_rate_Y = after_reset_flip_probability / 2.;
                reset_flip_node.pauli_error_rates.error_rate_Z = after_reset_flip_probability / 2.;
                let reset_flip_node = Arc::new(reset_flip_node);
                // iterate over all nodes
                simulator_iter_real!(simulator, position, node, {
                    // first clear error rate
                    noise_model.set_node(position, Some(noiseless_node.clone()));
                    if position.t >= simulator.height - simulator.measurement_cycles {
                        // no error on the top, as a perfect measurement round
                        continue;
                    }
                    // do different things for each stage
                    match position.t % simulator.measurement_cycles {
                        1 => {
                            // initialization
                            if node.qubit_type != QubitType::Data {
                                noise_model.set_node(position, Some(reset_flip_node.clone()));
                            } else {
                                noise_model.set_node(position, Some(data_qubit_depolarize_node.clone()));
                            }
                        }
                        0 => { // measurement
                             // do nothing; measurement errors need to be added before this round...
                        }
                        _ => {
                            let mut error_node = noiseless_node.clone();
                            if node.gate_type.is_two_qubit_gate()
                                && node.qubit_type == QubitType::Data
                                && !node.is_peer_virtual
                            {
                                // this is data qubit with actual 2-qubit gate
                                error_node = depolarize_2_node.clone();
                            }
                            if position.t % simulator.measurement_cycles == simulator.measurement_cycles - 1 {
                                if node.qubit_type != QubitType::Data {
                                    error_node = measure_flip_node.clone();
                                } else if position.t == simulator.height - simulator.measurement_cycles - 2 {
                                    let mut new_error_node = error_node.as_ref().clone();
                                    new_error_node.pauli_error_rates = data_qubit_depolarize_node.pauli_error_rates.clone();
                                    error_node = Arc::new(new_error_node);
                                }
                            }
                            noise_model.set_node(position, Some(error_node));
                        }
                    }
                });
            }
            Self::DepolarizingNoise => {
                let mut config_cloned = noise_model_configuration.clone();
                let config = config_cloned
                    .as_object_mut()
                    .expect("noise_model_configuration must be JSON object");
                if !config.is_empty() {
                    panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>());
                }
                // depolarizing node
                let mut depolarizing_node = NoiseModelNode::new();
                depolarizing_node.pauli_error_rates.error_rate_X = p / 3.;
                depolarizing_node.pauli_error_rates.error_rate_Z = p / 3.;
                depolarizing_node.pauli_error_rates.error_rate_Y = p / 3.;
                let depolarizing_node = Arc::new(depolarizing_node);
                // double depolarizing node
                let mut double_depolarizing_node = NoiseModelNode::new();
                double_depolarizing_node.pauli_error_rates.error_rate_X = 2. * p / 3.;
                double_depolarizing_node.pauli_error_rates.error_rate_Z = 2. * p / 3.;
                double_depolarizing_node.pauli_error_rates.error_rate_Y = 2. * p / 3.;
                let double_depolarizing_node = Arc::new(double_depolarizing_node);
                // two qubit depolarizing node
                let mut correlated_depolarizing_node = NoiseModelNode::new();
                let correlated_pauli_error_rates = CorrelatedPauliErrorRates::default_with_probability(p / 15.); // 15 possible errors equally probable
                correlated_depolarizing_node.correlated_pauli_error_rates = Some(correlated_pauli_error_rates);
                let correlated_depolarizing_node = Arc::new(correlated_depolarizing_node);
                // iterate over all nodes
                simulator_iter_real!(simulator, position, node, {
                    // first clear error rate
                    noise_model.set_node(position, Some(noiseless_node.clone()));
                    if position.t == 0 || position.t >= simulator.height - simulator.measurement_cycles {
                        // no error on the top, as a perfect measurement round
                        continue;
                    }
                    // do different things for each stage
                    match position.t % simulator.measurement_cycles {
                        1 => {
                            // initialization
                            noise_model.set_node(position, Some(depolarizing_node.clone()));
                        }
                        0 => {
                            // measurement
                            // do nothing
                            if node.qubit_type == QubitType::Data {
                                noise_model.set_node(position, Some(depolarizing_node.clone()));
                            }
                        }
                        _ => {
                            if node.is_peer_virtual || node.gate_peer.is_none() {
                                if position.t % simulator.measurement_cycles == simulator.measurement_cycles - 1
                                    && node.qubit_type != QubitType::Data
                                {
                                    noise_model.set_node(position, Some(double_depolarizing_node.clone()));
                                } else {
                                    noise_model.set_node(position, Some(depolarizing_node.clone()));
                                }
                            } else {
                                if node.qubit_type == QubitType::Data {
                                    noise_model.set_node(position, Some(correlated_depolarizing_node.clone()));
                                }
                                if position.t % simulator.measurement_cycles == simulator.measurement_cycles - 1
                                    && node.qubit_type != QubitType::Data
                                {
                                    noise_model.set_node(position, Some(depolarizing_node.clone()));
                                    // measurement error
                                }
                            }
                        }
                    }
                });
            }
        }
    }

    /// check as strictly as possible, given the user specified json noise model description
    pub fn apply_noise_model_modifier(
        simulator: &mut Simulator,
        noise_model: &mut NoiseModel,
        modifier: &serde_json::Value,
    ) -> Result<(), String> {
        if modifier.get("code_type").ok_or("missing field: code_type")? != &json!(simulator.code_type) {
            return Err("mismatch: code_type".to_string());
        }
        if modifier.get("height").ok_or("missing field: height")? != &json!(simulator.height) {
            return Err("mismatch: height".to_string());
        }
        if modifier.get("vertical").ok_or("missing field: vertical")? != &json!(simulator.vertical) {
            return Err("mismatch: vertical".to_string());
        }
        if modifier.get("horizontal").ok_or("missing field: horizontal")? != &json!(simulator.horizontal) {
            return Err("mismatch: horizontal".to_string());
        }
        // iterate nodes
        let nodes = modifier
            .get("nodes")
            .ok_or("missing field: nodes".to_string())?
            .as_array()
            .ok_or("format error: nodes".to_string())?;
        if simulator.nodes.len() != nodes.len() {
            return Err("mismatch: nodes.len()".to_string());
        }
        for (t, nodes_t) in nodes.iter().enumerate() {
            let nodes_row_0 = nodes_t.as_array().ok_or(format!("format error: nodes[{}]", t))?;
            if nodes_row_0.len() != simulator.nodes[t].len() {
                return Err(format!("mimsatch: nodes[{}].len()", t));
            }
            for (i, nodes_i) in nodes_row_0.iter().enumerate() {
                let nodes_row_1 = nodes_i.as_array().ok_or(format!("format error: nodes[{}][{}]", t, i))?;
                if nodes_row_1.len() != simulator.nodes[t][i].len() {
                    return Err(format!("mismatch: nodes[{}][{}].len()", t, i));
                }
                for (j, node) in nodes_row_1.iter().enumerate() {
                    if node.is_null() != simulator.nodes[t][i][j].is_none() {
                        return Err(format!("mismatch: nodes[{}][{}][{}].is_none", t, i, j));
                    }
                    if !node.is_null() {
                        let self_node = simulator.nodes[t][i][j].as_mut().unwrap(); // already checked existance
                        if node.get("position").ok_or("missing field: position".to_string())? != &json!(pos!(t, i, j)) {
                            return Err(format!("mismatch position [{}][{}][{}]", t, i, j));
                        }
                        if node.get("qubit_type").ok_or("missing field: qubit_type".to_string())?
                            != &json!(self_node.qubit_type)
                        {
                            return Err(format!("mismatch [{}][{}][{}]: qubit_type", t, i, j));
                        }
                        if node.get("gate_type").ok_or("missing field: gate_type".to_string())?
                            != &json!(self_node.gate_type)
                        {
                            return Err(format!("mismatch [{}][{}][{}]: gate_type", t, i, j));
                        }
                        if node.get("gate_peer").ok_or("missing field: gate_peer".to_string())?
                            != &json!(self_node.gate_peer)
                        {
                            return Err(format!("mismatch [{}][{}][{}]: gate_peer", t, i, j));
                        }
                        // TODO: user can modify the 'is_virtual' attribute to manually discard a measurement event
                        let is_virtual = node
                            .get("is_virtual")
                            .ok_or("missing field: is_virtual".to_string())?
                            .as_bool()
                            .ok_or("wrong field: is_virtual".to_string())?;
                        let is_peer_virtual = node
                            .get("is_peer_virtual")
                            .ok_or("missing field: is_peer_virtual".to_string())?
                            .as_bool()
                            .ok_or("wrong field: is_peer_virtual".to_string())?;
                        assert_eq!(
                            is_virtual, self_node.is_virtual,
                            "is_virtual modification not implemented, needs sanity check"
                        );
                        assert_eq!(
                            is_peer_virtual, self_node.is_peer_virtual,
                            "is_peer_virtual modification not implemented, needs sanity check"
                        );
                        // then copy error rate data
                        let noise_model_node = node
                            .get("noise_model")
                            .ok_or("missing field: noise_model".to_string())?
                            .clone();
                        let noise_model_node: NoiseModelNode =
                            serde_json::from_value(noise_model_node).map_err(|e| format!("{:?}", e))?;
                        noise_model.set_node(&pos!(t, i, j), Some(Arc::new(noise_model_node)));
                    }
                }
            }
        }
        Ok(())
    }
}

impl std::str::FromStr for NoiseModelBuilder {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("Invalid variant: {}", s))
    }
}

#[cfg(feature = "python_binding")]
#[pyfunction]
pub(crate) fn register(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<NoiseModelBuilder>()?;
    Ok(())
}
