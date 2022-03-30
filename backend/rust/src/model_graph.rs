//! build model graph from simulator and measurement results
//! 

use super::simulator::*;
use super::util_macros::*;
use std::collections::{BTreeMap};
use super::either::Either;
use super::types::*;
use super::error_model::*;
use std::sync::{Arc};
use serde::{Serialize};
use super::float_cmp;

/// edges connecting two nontrivial measurements generated by a single error
#[derive(Debug, Clone, Serialize)]
pub struct ModelGraph {
    pub nodes: Vec::< Vec::< Vec::< Option< Box< ModelGraphNode > > > > >,
}

/// only defined for measurement nodes (including virtual measurement nodes)
#[derive(Debug, Clone, Serialize)]
pub struct ModelGraphNode {
    /// used when building the graph, record all possible edges that connect the two measurement syndromes.
    /// (this might be dropped to save memory usage after election)
    all_edges: BTreeMap<Position, Vec<ModelGraphEdge>>,
    /// the elected edges, to make sure each pair of nodes only have one edge
    edges: BTreeMap<Position, ModelGraphEdge>,
    /// all boundary edges defined by a single qubit error generating only one nontrivial measurement.
    all_boundaries: Vec<ModelGraphBoundary>,
    /// the elected boundary out of all, note that `virtual_node` here is always set to None
    boundary: Option<ModelGraphBoundary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelGraphEdge {
    /// the probability of this edge to happen
    probability: f64,
    /// the error that causes this edge
    error_pattern: Arc<SparseErrorPattern>,
    /// the correction pattern that can recover this error
    correction: Arc<SparseCorrection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelGraphBoundary {
    /// the probability of this boundary edge to happen
    probability: f64,
    /// the error that causes this boundary edge
    error_pattern: Arc<SparseErrorPattern>,
    /// the correction pattern that can recover this error
    correction: Arc<SparseCorrection>,
    /// if virtual node presents, record it, otherwise the model graph is still constructed successfully
    virtual_node: Option<Position>,
}

// /// edges associating four nontrivial measurements generated by a single error
// pub struct TailoredModelGraph {

// }

impl ModelGraph {
    /// initialize the structure corresponding to a `Simulator`
    pub fn new(simulator: &Simulator) -> Self {
        assert!(simulator.volume() > 0, "cannot build graph out of zero-sized simulator");
        Self {
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if t != 0 && t % simulator.measurement_cycles == 0 && simulator.is_node_exist(position) {
                            let node = simulator.get_node_unwrap(position);
                            if node.gate_type.is_measurement() {  // only define model graph node for measurements
                                return Some(Box::new(ModelGraphNode {
                                    all_edges: BTreeMap::new(),
                                    edges: BTreeMap::new(),
                                    all_boundaries: Vec::new(),
                                    boundary: None,
                                }))
                            }
                        }
                        None
                    }).collect()
                }).collect()
            }).collect(),
        }
    }

    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<Box<ModelGraphNode>> {
        &self.nodes[position.t][position.i][position.j]
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ ModelGraphNode {
        self.get_node(position).as_ref().unwrap()
    }

    /// get mutable reference `self.nodes[t][i][j]` and unwrap
    pub fn get_node_mut_unwrap(&'_ mut self, position: &Position) -> &'_ mut ModelGraphNode {
        self.nodes[position.t][position.i][position.j].as_mut().unwrap()
    }

    /// build model graph given the simulator
    pub fn build(&mut self, simulator: &mut Simulator, error_model: &ErrorModel) {
        debug_assert!({
            let mut state_clean = true;
            simulator_iter!(simulator, position, node, {
                // here I omitted the condition `t % measurement_cycles == 0` for a stricter check
                if position.t != 0 && node.gate_type.is_measurement() {
                    let model_graph_node = self.get_node_unwrap(position);
                    if model_graph_node.all_edges.len() > 0 || model_graph_node.edges.len() > 0 {
                        state_clean = false;
                    }
                }
            });
            if !state_clean {
                println!("[warning] state must be clean before calling `build`, please make sure you don't call this function twice");
            }
            state_clean
        });
        // calculate all possible errors to be iterated
        let mut all_possible_errors: Vec<Either<ErrorType, CorrelatedPauliErrorType>> = Vec::new();
        for error_type in ErrorType::all_possible_errors().drain(..) {
            all_possible_errors.push(Either::Left(error_type));
        }
        for correlated_error_type in CorrelatedPauliErrorType::all_possible_errors().drain(..) {
            all_possible_errors.push(Either::Right(correlated_error_type));
        }
        // clear the states in simulator including pauli, erasure errors and propagated errors
        simulator.clear_all_errors();
        // iterate over all possible errors at all possible positions
        simulator_iter!(simulator, position, {
            let error_model_node = error_model.get_node_unwrap(position);
            // whether it's possible to have erasure error at this node
            let possible_erasure_error = error_model_node.erasure_error_rate > 0. || error_model_node.correlated_erasure_error_rates.is_some() || {
                let node = simulator.get_node_unwrap(position);
                if let Some(gate_peer) = node.gate_peer.as_ref() {
                    let peer_error_model_node = error_model.get_node_unwrap(gate_peer);
                    if let Some(correlated_erasure_error_rates) = &peer_error_model_node.correlated_erasure_error_rates {
                        correlated_erasure_error_rates.error_probability() > 0.
                    } else { false }
                } else { false }
            };
            for error in all_possible_errors.iter() {
                let p = match error {
                    Either::Left(error_type) => {
                        error_model_node.pauli_error_rates.error_rate(error_type)
                    },
                    Either::Right(error_type) => {
                        match &error_model_node.correlated_pauli_error_rates {
                            Some(correlated_pauli_error_rates) => {
                                correlated_pauli_error_rates.error_rate(error_type)
                            },
                            None => 0.,
                        }
                    },
                }; // probability of this error to occur
                let is_erasure = possible_erasure_error && error.is_left();
                if p > 0. || is_erasure {  // use possible errors to build `all_edges`
                    // simulate the error and measure it
                    let mut sparse_errors = SparseErrorPattern::new();
                    match error {
                        Either::Left(error_type) => {
                            sparse_errors.add(position.clone(), error_type.clone());
                        },
                        Either::Right(error_type) => {
                            sparse_errors.add(position.clone(), error_type.my_error());
                            let node = simulator.get_node_unwrap(position);
                            let gate_peer = node.gate_peer.as_ref().expect("correlated error must corresponds to a two-qubit gate");
                            sparse_errors.add((**gate_peer).clone(), error_type.peer_error());
                        },
                    }
                    let sparse_errors = Arc::new(sparse_errors);  // make it immutable and shared
                    let (sparse_correction, sparse_measurement_real, sparse_measurement_virtual) = simulator.fast_measurement_given_few_errors(&sparse_errors);
                    let sparse_correction = Arc::new(sparse_correction);  // make it immutable and shared
                    let sparse_measurement_real = sparse_measurement_real.to_vec();
                    let sparse_measurement_virtual = sparse_measurement_virtual.to_vec();
                    if sparse_measurement_real.len() == 0 {  // no way to detect it, ignore
                        continue
                    }
                    // println!("{:?} at {} will cause measurement errors: real {:?} and virtual {:?}", error, position, sparse_measurement_real, sparse_measurement_virtual);
                    if sparse_measurement_real.len() == 1 {  // boundary edge
                        let position = &sparse_measurement_real[0];
                        if p > 0. || is_erasure {  // add this boundary edge
                            let model_graph_node = self.get_node_mut_unwrap(position);
                            model_graph_node.all_boundaries.push(ModelGraphBoundary {
                                probability: p,
                                error_pattern: sparse_errors.clone(),
                                correction: sparse_correction.clone(),
                                virtual_node: if sparse_measurement_virtual.len() == 1 {
                                    Some(sparse_measurement_virtual[0].clone())
                                } else {
                                    None
                                },
                            });
                        }
                    }
                    if sparse_measurement_real.len() == 2 {  // normal edge
                        let position1 = &sparse_measurement_real[0];
                        let position2 = &sparse_measurement_real[1];
                        let node1 = simulator.get_node_unwrap(position1);
                        let node2 = simulator.get_node_unwrap(position2);
                        // edge only happen when qubit type is the same (to isolate X and Z decoding graph in CSS surface code)
                        let is_same_type = node1.qubit_type == node2.qubit_type;
                        if is_same_type && (p > 0. || is_erasure) {
                            self.add_edge_between(position1, position2, p, sparse_errors.clone(), sparse_correction.clone());
                            self.add_edge_between(position2, position1, p, sparse_errors.clone(), sparse_correction.clone());
                        }
                    }
                }
            }
        });
        self.elect_edges(simulator, true);  // by default use combined probability
    }

    /// add asymmetric edge from `source` to `target`; in order to create symmetric edge, call this function twice with reversed input
    pub fn add_edge_between(&mut self, source: &Position, target: &Position, probability: f64, error_pattern: Arc<SparseErrorPattern>, correction: Arc<SparseCorrection>) {
        let node = self.get_node_mut_unwrap(source);
        if !node.all_edges.contains_key(target) {
            node.all_edges.insert(target.clone(), Vec::new());
        }
        node.all_edges.get_mut(target).unwrap().push(ModelGraphEdge {
            probability: probability,
            error_pattern: error_pattern,
            correction: correction,
        })
    }

    /// if there are multiple edges connecting two stabilizer measurements, elect the best one
    pub fn elect_edges(&mut self, simulator: &Simulator, use_combined_probability: bool) {
        for t in (simulator.measurement_cycles..simulator.height).step_by(simulator.measurement_cycles) {
            simulator_iter_real!(simulator, position, node, t => t, if node.gate_type.is_measurement() {
                let model_graph_node = self.get_node_mut_unwrap(position);
                // elect normal edges
                for (target, edges) in model_graph_node.all_edges.iter() {
                    let mut elected_idx = 0;
                    let mut elected_probability = edges[0].probability;
                    for i in 1..edges.len() {
                        let edge = &edges[i];
                        // update `elected_probability`
                        if use_combined_probability {
                            elected_probability = elected_probability * (1. - edge.probability) + edge.probability * (1. - elected_probability);  // XOR
                        } else {
                            elected_probability = elected_probability.max(edge.probability);
                        }
                        // update `elected_idx`
                        let best_edge = &edges[elected_idx];
                        if edge.probability > best_edge.probability {
                            elected_idx = i;  // set as best, use its 
                        }
                    }
                    let elected = ModelGraphEdge {
                        probability: elected_probability,
                        error_pattern: edges[elected_idx].error_pattern.clone(),
                        correction: edges[elected_idx].correction.clone(),
                    };
                    // update elected edge
                    // println!("{} to {} elected probability: {}", position, target, elected.probability);
                    model_graph_node.edges.insert(target.clone(), elected);
                }
                // elect boundary edge
                if model_graph_node.all_boundaries.len() > 0 {
                    let mut elected_idx = 0;
                    let mut elected_probability = model_graph_node.all_boundaries[0].probability;
                    for i in 1..model_graph_node.all_boundaries.len() {
                        let edge = &model_graph_node.all_boundaries[i];
                        // update `elected_probability`
                        if use_combined_probability {
                            elected_probability = elected_probability * (1. - edge.probability) + edge.probability * (1. - elected_probability);  // XOR
                        } else {
                            elected_probability = elected_probability.max(edge.probability);
                        }
                        // update `elected_idx`
                        let best_edge = &model_graph_node.all_boundaries[elected_idx];
                        if edge.probability > best_edge.probability {
                            elected_idx = i;  // set as best, use its 
                        }
                    }
                    let elected = ModelGraphBoundary {
                        probability: elected_probability,
                        error_pattern: model_graph_node.all_boundaries[elected_idx].error_pattern.clone(),
                        correction: model_graph_node.all_boundaries[elected_idx].correction.clone(),
                        virtual_node: None,
                    };
                    // update elected edge
                    // println!("{} to virtual boundary elected probability: {}", position, elected.probability);
                    model_graph_node.boundary = Some(elected);
                } else {
                    model_graph_node.boundary = None;
                }
            });
        }
        // sanity check, two nodes on one edge have the same edge information, should be a cheap sanity check
        debug_assert!({
            let mut sanity_check_passed = true;
            for t in (simulator.measurement_cycles..simulator.height).step_by(simulator.measurement_cycles) {
                simulator_iter_real!(simulator, position, node, t => t, if node.gate_type.is_measurement() {
                    let model_graph_node = self.get_node_unwrap(position);
                    for (target, edge) in model_graph_node.edges.iter() {
                        let target_model_graph_node = self.get_node_unwrap(target);
                        let reverse_edge = target_model_graph_node.edges.get(position).expect("edge should be symmetric");
                        if !float_cmp::approx_eq!(f64, edge.probability, reverse_edge.probability, ulps = 5) {
                            println!("[warning] the edge between {} and {} has unequal probability {} and {}"
                                , position, target, edge.probability, reverse_edge.probability);
                            sanity_check_passed = false;
                        }
                    }
                });
            }
            sanity_check_passed
        });
    }
}
