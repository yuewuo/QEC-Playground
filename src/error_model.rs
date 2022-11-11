//! # Error Model
//!
//! customized error rate with high flexibility
//! 

#[cfg(feature="python_binding")]
use super::pyo3::prelude::*;
use super::simulator::*;
use super::util_macros::*;
use super::types::*;
use serde::{Serialize, Deserialize};
use super::code_builder::*;
use std::sync::Arc;
use crate::visualize::*;


/// describing an error model, strictly corresponding to an instance of `Simulator`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct ErrorModel {
    /// each error model node corresponds to a simulator node, this allows immutable sharing between threads
    pub nodes: Vec::< Vec::< Vec::< Option<Arc <ErrorModelNode> > > > >,
}

impl QecpVisualizer for ErrorModel {
    fn component_info(&self, abbrev: bool) -> (String, serde_json::Value) {
        let name = "error_model";
        let info = json!({
            "nodes": (0..self.nodes.len()).map(|t| {
                (0..self.nodes[t].len()).map(|i| {
                    (0..self.nodes[t][i].len()).map(|j| {
                        let position = &pos!(t, i, j);
                        if self.is_node_exist(position) {
                            let node = self.get_node_unwrap(position);
                            Some(json!({
                                if abbrev { "p" } else { "position" }: position,  // for readability
                                if abbrev { "pp" } else { "pauli_error_rates" }: node.pauli_error_rates,
                                if abbrev { "pe" } else { "erasure_error_rate" }: node.erasure_error_rate,
                                if abbrev { "corr_pp" } else { "correlated_pauli_error_rates" }: node.correlated_pauli_error_rates,
                                if abbrev { "corr_pe" } else { "correlated_erasure_error_rates" }: node.correlated_erasure_error_rates,
                            }))
                        } else {
                            None
                        }
                    }).collect::<Vec<Option<serde_json::Value>>>()
                }).collect::<Vec<Vec<Option<serde_json::Value>>>>()
            }).collect::<Vec<Vec<Vec<Option<serde_json::Value>>>>>()
        });
        (name.to_string(), info)
    }
}

/// error model node corresponds to 
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct ErrorModelNode {
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

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pymethods)]
impl ErrorModelNode {
    #[cfg_attr(feature = "python_binding", new)]
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

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pymethods)]
impl ErrorModel {
    #[cfg_attr(feature = "python_binding", new)]
    pub fn new(simulator: &Simulator) -> Self {
        assert!(simulator.volume() > 0, "cannot build error model out of zero-sized simulator");
        let default_error_model_node = Arc::new(ErrorModelNode::new());
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
}

impl ErrorModel {

    /// judge if `[t][i][j]` is valid index of `self.nodes`
    #[inline]
    pub fn is_valid_position(&self, position: &Position) -> bool {
        position.t < self.nodes.len() && position.i < self.nodes[position.t].len() && position.j < self.nodes[position.t][position.i].len()
    }

    /// judge if `self.nodes[t][i][j]` is `Some(_)`
    #[inline]
    pub fn is_node_exist(&self, position: &Position) -> bool {
        self.is_valid_position(position) && self.get_node(position).is_some()
    }

    /// get `self.nodes[t][i][j]` without position check when compiled in release mode
    #[inline]
    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<Arc<ErrorModelNode>> {
        &self.nodes[position.t][position.i][position.j]
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ ErrorModelNode {
        self.nodes[position.t][position.i][position.j].as_ref().unwrap()
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap, returning a clone of the arc
    pub fn get_node_unwrap_arc(&'_ self, position: &Position) -> Arc<ErrorModelNode> {
        self.nodes[position.t][position.i][position.j].as_ref().unwrap().clone()
    }

    /// each node is immutable, but one can assign a new node
    pub fn set_node(&mut self, position: &Position, node: Option<Arc<ErrorModelNode>>) {
        self.nodes[position.t][position.i][position.j] = node;
    }
}

/// check if error rates are not zero at perfect measurement ranges or at (always) virtual nodes,
/// also check for error rate constrains on virtual nodes
pub fn error_model_sanity_check(simulator: &Simulator, error_model: &ErrorModel) -> Result<(), String> {
    match simulator.code_size {
        CodeSize { noisy_measurements, .. } => {
            // check that no errors present in the final perfect measurement rounds
            let expected_height = simulator.measurement_cycles * (noisy_measurements + 1) + 1;
            if simulator.height != expected_height {
                return Err(format!("height {} is not expected {}, don't know where is perfect measurement", simulator.height, expected_height))
            }
            for t in simulator.height - simulator.measurement_cycles .. simulator.height {
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
        }
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
        if node.is_peer_virtual {  // no correlated errors if peer position is virtual, because this two-qubit gate doesn't physically exist
            if let Some(correlated_pauli_error_rates) = &error_model_node.correlated_pauli_error_rates {
                if correlated_pauli_error_rates.error_probability() > 0. {
                    return Err(format!("position at {} have virtual peer but non-zero correlated_pauli_error_rates: {:?}", position, correlated_pauli_error_rates))
                }
            }
            if let Some(correlated_erasure_error_rates) = &error_model_node.correlated_erasure_error_rates {
                if correlated_erasure_error_rates.error_probability() > 0. {
                    return Err(format!("position at {} have virtual peer but non-zero correlated_erasure_error_rates: {:?}", position, correlated_erasure_error_rates))
                }
            }
        }
    });
    Ok(())
}

#[cfg(feature="python_binding")]
#[pyfunction]
pub(crate) fn register(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ErrorModel>()?;
    m.add_class::<ErrorModelNode>()?;
    Ok(())
}
