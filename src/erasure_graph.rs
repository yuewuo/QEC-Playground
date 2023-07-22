//! Erasure Graph
//!
//! An erasure is a detected event indicating errors happening at a specific position.
//! It's often modeled and simulated as applying random Pauli errors to this position.
//!
//! For MWPM decoder and UF decoder, if an erasure happens at a specific position, some edges will be modified to weight 0.
//! this module calculates the set of such edges for each position, and can be quickly retrieved during simulations.
//!

use super::noise_model::*;
#[cfg(feature = "python_binding")]
use super::pyo3::prelude::*;
use super::simulator::*;
use super::types::*;
use super::util_macros::*;
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// edges connecting two nontrivial measurements generated by a single error
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct ErasureGraph {
    pub nodes: Vec<Vec<Vec<Option<Box<ErasureGraphNode>>>>>,
}

/// an edge cause by an erasure is either connection between measurement nodes or to boundary
#[derive(Debug, Clone, Serialize)]
pub enum ErasureEdge {
    Connection(Position, Position),
    Boundary(Position),
}

/// each node corresponds to a simulator node
#[derive(Debug, Clone, Serialize)]
pub struct ErasureGraphNode {
    /// erasure generated connections, generated from Pauli X, Z and Y errors
    pub erasure_edges: Vec<ErasureEdge>,
}

impl ErasureGraph {
    /// initialize the structure corresponding to a `Simulator`
    pub fn new(simulator: &Simulator) -> Self {
        assert!(
            simulator.volume() > 0,
            "cannot build erasure graph out of zero-sized simulator"
        );
        Self {
            nodes: (0..simulator.height)
                .map(|_| {
                    (0..simulator.vertical)
                        .map(|_| (0..simulator.horizontal).map(|_| None).collect())
                        .collect()
                })
                .collect(),
        }
    }

    /// any valid position of the simulator is a valid position in model graph, but only some of these positions corresponds a valid node in model graph
    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<Box<ErasureGraphNode>> {
        &self.nodes[position.t][position.i][position.j]
    }

    /// check if a position contains model graph node
    pub fn is_node_exist(&self, position: &Position) -> bool {
        self.get_node(position).is_some()
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ ErasureGraphNode {
        self.get_node(position).as_ref().unwrap()
    }

    /// get mutable `self.nodes[t][i][j]` without position check when compiled in release mode
    #[inline]
    pub fn get_node_mut(&'_ mut self, position: &Position) -> &'_ mut Option<Box<ErasureGraphNode>> {
        &mut self.nodes[position.t][position.i][position.j]
    }

    /// build erasure graph given the simulator and the noise model in a specific region, for parallel initialization
    pub fn build_with_region(
        &mut self,
        simulator: &mut Simulator,
        noise_model: Arc<NoiseModel>,
        t_start: usize,
        t_end: usize,
    ) {
        let all_possible_errors = ErrorType::all_possible_errors();
        // clear the states in simulator including pauli, erasure errors and propagated errors
        simulator.clear_all_errors();
        // iterate over all possible errors at all possible positions
        simulator_iter!(simulator, position, {
            if position.t < t_start || position.t >= t_end {
                continue;
            }
            let noise_model_node = noise_model.get_node_unwrap(position);
            // whether it's possible to have erasure error at this node
            let possible_erasure_error =
                noise_model_node.erasure_error_rate > 0. || noise_model_node.correlated_erasure_error_rates.is_some() || {
                    let node = simulator.get_node_unwrap(position);
                    if let Some(gate_peer) = node.gate_peer.as_ref() {
                        let peer_noise_model_node = noise_model.get_node_unwrap(gate_peer);
                        if let Some(correlated_erasure_error_rates) = &peer_noise_model_node.correlated_erasure_error_rates {
                            correlated_erasure_error_rates.error_probability() > 0.
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };
            if possible_erasure_error {
                let mut erasure_edges = Vec::new();
                for error in all_possible_errors.iter() {
                    // simulate the error and measure it
                    let mut sparse_errors = SparseErrorPattern::new();
                    sparse_errors.add(position.clone(), *error);
                    let sparse_errors = Arc::new(sparse_errors); // make it immutable and shared
                    let (_sparse_correction, sparse_measurement_real, _sparse_measurement_virtual) =
                        simulator.fast_measurement_given_few_errors(&sparse_errors);
                    let sparse_measurement_real = sparse_measurement_real.to_vec();
                    if sparse_measurement_real.is_empty() {
                        // no way to detect it, ignore
                        continue;
                    }
                    if sparse_measurement_real.len() == 1 {
                        // boundary edge
                        let position = &sparse_measurement_real[0];
                        erasure_edges.push(ErasureEdge::Boundary(position.clone()));
                    }
                    if sparse_measurement_real.len() == 2 {
                        // normal edge
                        let position1 = &sparse_measurement_real[0];
                        let position2 = &sparse_measurement_real[1];
                        let node1 = simulator.get_node_unwrap(position1);
                        let node2 = simulator.get_node_unwrap(position2);
                        // edge only happen when qubit type is the same (to isolate X and Z decoding graph in CSS surface code)
                        let is_same_type = node1.qubit_type == node2.qubit_type;
                        if is_same_type {
                            erasure_edges.push(ErasureEdge::Connection(position1.clone(), position2.clone()));
                        }
                    }
                }
                self.nodes[position.t][position.i][position.j] = Some(Box::new(ErasureGraphNode { erasure_edges }))
            }
        });
    }

    /// build erasure graph given the simulator and the noise model
    pub fn build(&mut self, simulator: &mut Simulator, noise_model: Arc<NoiseModel>, parallel: usize) {
        debug_assert!({
            let mut state_clean = true;
            simulator_iter!(simulator, position, _node, {
                // here I omitted the condition `t % measurement_cycles == 0` for a stricter check
                if self.is_node_exist(position) {
                    state_clean = false;
                }
            });
            if !state_clean {
                println!("[warning] state must be clean before calling `build`, please make sure you don't call this function twice");
            }
            state_clean
        });
        if parallel <= 1 {
            self.build_with_region(simulator, noise_model, 0, simulator.height);
        } else {
            // spawn `parallel` threads to compute in parallel
            let mut handlers = Vec::new();
            let mut instances = Vec::new();
            let interval = simulator.height / parallel;
            for parallel_idx in 0..parallel {
                let instance = Arc::new(Mutex::new(self.clone()));
                let mut simulator = simulator.clone();
                instances.push(Arc::clone(&instance));
                let t_start = parallel_idx * interval; // included
                let mut t_end = (parallel_idx + 1) * interval; // excluded
                if parallel_idx == parallel - 1 {
                    t_end = simulator.height; // to make sure every part is included
                }
                let noise_model = Arc::clone(&noise_model);
                handlers.push(std::thread::spawn(move || {
                    let mut instance = instance.lock().unwrap();
                    instance.build_with_region(&mut simulator, noise_model, t_start, t_end);
                }));
            }
            for handler in handlers.drain(..) {
                handler.join().unwrap();
            }
            // move the data from instances (without additional large memory allocation)
            for instance in instances.iter() {
                let mut instance = instance.lock().unwrap();
                simulator_iter!(
                    simulator,
                    position,
                    if instance.is_node_exist(position) {
                        assert!(
                            !self.is_node_exist(position),
                            "critical bug: two parallel tasks should not work on the same vertex"
                        );
                        std::mem::swap(self.get_node_mut(position), instance.get_node_mut(position));
                    }
                );
            }
        }
    }

    /// create json object for debugging and viewing
    pub fn to_json(&self, simulator: &Simulator) -> serde_json::Value {
        json!({
            "code_type": simulator.code_type,
            "height": simulator.height,
            "vertical": simulator.vertical,
            "horizontal": simulator.horizontal,
            "nodes": (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if self.is_node_exist(position) {
                            let node = self.get_node_unwrap(position);
                            Some(json!({
                                "position": position,
                                "erasure_edges": node.erasure_edges,
                            }))
                        } else {
                            None
                        }
                    }).collect::<Vec<Option<serde_json::Value>>>()
                }).collect::<Vec<Vec<Option<serde_json::Value>>>>()
            }).collect::<Vec<Vec<Vec<Option<serde_json::Value>>>>>()
        })
    }
}

/// temporarily remember the weights that has been changed, so that it can revert back
pub struct ErasureGraphModifier<Weight> {
    /// edge with 0 weighted caused by the erasure, used by UF decoder or (indirectly) by MWPM decoder
    pub modified: Vec<(ErasureEdge, Weight)>,
}

impl<Weight> Default for ErasureGraphModifier<Weight> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Weight> ErasureGraphModifier<Weight> {
    pub fn new() -> Self {
        Self { modified: Vec::new() }
    }
    /// record the modified edge
    pub fn push_modified_edge(&mut self, erasure_edge: ErasureEdge, original_weight: Weight) {
        self.modified.push((erasure_edge, original_weight));
    }
    /// if some edges are not recovered
    pub fn has_modified_edges(&self) -> bool {
        !self.modified.is_empty()
    }
    /// retrieve the last modified edge, panic if no more modified edges
    pub fn pop_modified_edge(&mut self) -> (ErasureEdge, Weight) {
        self.modified
            .pop()
            .expect("no more modified edges, please check `has_modified_edges` before calling this method")
    }
}
