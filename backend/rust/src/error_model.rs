//! # Error Model
//!
//! customized error rate with high flexibility
//! 

use super::simulator::*;
use super::util_macros::*;
use super::types::*;
use serde::{Serialize, Deserialize};
use super::code_builder::*;
use super::util_macros::*;
use std::sync::Arc;

/// describing an error model, strictly corresponding to an instance of `Simulator`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorErrorModel {
    /// each error model node corresponds to a simulator node, this allows immutable sharing between threads
    pub nodes: Vec::< Vec::< Vec::< Option<Arc <SimulatorErrorModelNode> > > > >,
}

/// error model node corresponds to 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorErrorModelNode {
    /// without losing generality, errors are applied after the gate
    #[serde(rename = "pp")]
    pub pauli_error_rates: PauliErrorRates,
    #[serde(rename = "pe")]
    pub erasure_error_rate: f64,
    #[serde(rename = "corr_pp")]
    pub correlated_pauli_error_rates: Option<CorrelatedPauliErrorRates>,
    #[serde(rename = "corr_pe")]
    pub correlated_erasure_error_rates: Option<CorrelatedErasureErrorRates>,
}

impl SimulatorErrorModelNode {
    pub fn new() -> Self {
        Self {
            pauli_error_rates: PauliErrorRates::default(),
            erasure_error_rate: 0.,
            correlated_pauli_error_rates: None,
            correlated_erasure_error_rates: None,
        }
    }

    /// check if this place has error rate = 0
    pub fn is_noiseless(&self) -> bool {
        if self.pauli_error_rates.error_probability() > 0. {
            return false
        }
        if self.erasure_error_rate > 0. {
            return false
        }
        if self.correlated_pauli_error_rates.is_some() && self.correlated_pauli_error_rates.as_ref().unwrap().error_probability() > 0. {
            return false
        }
        if self.correlated_erasure_error_rates.is_some() && self.correlated_erasure_error_rates.as_ref().unwrap().error_probability() > 0. {
            return false
        }
        true
    }
}

impl SimulatorErrorModel {
    pub fn new(simulator: &Simulator) -> Self {
        assert!(simulator.volume() > 0, "cannot build error model out of zero-sized simulator");
        let default_error_model_node = Arc::new(SimulatorErrorModelNode::new());
        Self {
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        if simulator.is_node_exist(&pos!(t, i, j)) {
                            Some(default_error_model_node.clone())
                        } else {
                            None
                        }
                    }).collect()
                }).collect()
            }).collect()
        }
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ SimulatorErrorModelNode {
        self.nodes[position.t][position.i][position.j].as_ref().unwrap()
    }

    /// each node is immutable, but one can assign a new node
    pub fn set_node(&mut self, position: &Position, node: Option<Arc<SimulatorErrorModelNode>>) {
        self.nodes[position.t][position.i][position.j] = node;
    }
}

/// check if error rates are not zero at perfect measurement ranges or at virtual nodes,
/// also check for error rate constrains on virtual nodes
pub fn error_model_sanity_check(simulator: &Simulator, error_model: &SimulatorErrorModel) -> Result<(), String> {
    match simulator.code_type.builtin_code_information() {
        Some(BuiltinCodeInformation{ measurement_cycles, noisy_measurements, .. }) => {
            // check that no errors present in the final perfect measurement rounds
            let expected_height = measurement_cycles * (noisy_measurements + 1) + 1;
            if simulator.height != expected_height {
                return Err(format!("height {} is not expected {}, don't know where is perfect measurement", simulator.height, expected_height))
            }
            for t in simulator.height - measurement_cycles .. simulator.height {
                simulator_iter!(simulator, position, _node, t => t, {
                    let error_model_node = error_model.get_node_unwrap(position);
                    if !error_model_node.is_noiseless() {
                        return Err(format!("detected noisy position {} within final perfect measurement", position))
                    }
                });
            }
            // check all no error rate at virtual nodes
            simulator_iter_virtual!(simulator, position, _node, {  // only check for virtual nodes
                let error_model_node = error_model.get_node_unwrap(position);
                if !error_model_node.is_noiseless() {
                    return Err(format!("detected noisy position {} which is virtual node", position))
                }
            });
        }, _ => { println!("[warning] code doesn't provide enough information for sanity check") }
    }
    simulator_iter!(simulator, position, node, {
        let error_model_node = error_model.get_node_unwrap(position);
        if node.is_virtual {  // no errors on virtual node is allowed, because they don't physically exist
            if error_model_node.pauli_error_rates.error_probability() > 0. {
                return Err(format!("virtual position at {} have non-zero pauli_error_rates: {:?}", position, error_model_node.pauli_error_rates))
            }
            if error_model_node.erasure_error_rate > 0. {
                return Err(format!("virtual position at {} have non-zero erasure_error_rate: {}", position, error_model_node.erasure_error_rate))
            }
            if let Some(correlated_pauli_error_rates) = &error_model_node.correlated_pauli_error_rates {
                if correlated_pauli_error_rates.error_probability() > 0. {
                    return Err(format!("virtual position at {} have non-zero correlated_pauli_error_rates: {:?}", position, correlated_pauli_error_rates))
                }
            }
            if let Some(correlated_erasure_error_rates) = &error_model_node.correlated_erasure_error_rates {
                if correlated_erasure_error_rates.error_probability() > 0. {
                    return Err(format!("virtual position at {} have non-zero correlated_erasure_error_rates: {:?}", position, correlated_erasure_error_rates))
                }
            }
        }
        // if node.is_peer_virtual {  // no correlated errors if peer position is virtual, because this two-qubit gate doesn't physically exist
        //     if node.pauli_error_rates.error_probability() > 0. {
        //         return Err(format!("virtual position at {} have non-zero pauli_error_rates: {:?}", position, node.pauli_error_rates))
        //     }
        // }
    });
    Ok(())
}
