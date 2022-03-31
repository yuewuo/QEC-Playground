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
use super::model_graph::*;
use super::float_cmp;
use super::util;

/// edges connecting two nontrivial measurements generated by a single error
#[derive(Debug, Clone, Serialize)]
pub struct TailoredModelGraph {
    /// `(positive_node, negative_node, neutral_node)`, where neutral node only contains 
    pub nodes: Vec::< Vec::< Vec::< Option< Box< TripleTailoredModelGraphNode > > > > >,
}

/// only defined for measurement nodes (including virtual measurement nodes)
#[derive(Debug, Clone, Serialize)]
pub struct TailoredModelGraphNode {
    /// used when building the graph, record all possible edges that connect the two measurement syndromes.
    /// (this might be dropped to save memory usage after election)
    pub all_edges: BTreeMap<Position, Vec<TailoredModelGraphEdge>>,
    /// the elected edges, to make sure each pair of nodes only have one edge
    pub edges: BTreeMap<Position, TailoredModelGraphEdge>,
}

pub type TripleTailoredModelGraphNode = [TailoredModelGraphNode; 3];

#[derive(Debug, Clone, Serialize)]
pub struct TailoredModelGraphEdge {
    /// the probability of this edge to happen
    pub probability: f64,
    /// the weight of this edge computed by the (combined) probability, e.g. ln((1-p)/p)
    pub weight: f64,
    /// the error that causes this edge
    pub error_pattern: Arc<SparseErrorPattern>,
    /// the correction pattern that can recover this error
    pub correction: Arc<SparseCorrection>,
}

impl TailoredModelGraph {
    /// initialize the structure corresponding to a `Simulator`
    pub fn new(simulator: &Simulator) -> Self {
        assert!(simulator.volume() > 0, "cannot build graph out of zero-sized simulator");
        Self {
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        // tailored model graph contains both real node and virtual node at measurement round
                        if t != 0 && t % simulator.measurement_cycles == 0 && simulator.is_node_exist(position) {
                            let node = simulator.get_node_unwrap(position);
                            if node.gate_type.is_measurement() {  // only define model graph node for measurements
                                return Some(Box::new([TailoredModelGraphNode {
                                    all_edges: BTreeMap::new(),
                                    edges: BTreeMap::new(),
                                }, TailoredModelGraphNode {
                                    all_edges: BTreeMap::new(),
                                    edges: BTreeMap::new(),
                                }, TailoredModelGraphNode {
                                    all_edges: BTreeMap::new(),
                                    edges: BTreeMap::new(),
                                }]))
                            }
                        }
                        None
                    }).collect()
                }).collect()
            }).collect(),
        }
    }

    /// any valid position of the simulator is a valid position in model graph, but only some of these positions corresponds a valid node in model graph
    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<Box<TripleTailoredModelGraphNode>> {
        &self.nodes[position.t][position.i][position.j]
    }

    /// check if a position contains model graph node
    pub fn is_node_exist(&self, position: &Position) -> bool {
        self.get_node(position).is_some()
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ TripleTailoredModelGraphNode {
        self.get_node(position).as_ref().unwrap()
    }

    /// get mutable reference `self.nodes[t][i][j]` and unwrap
    pub fn get_node_mut_unwrap(&'_ mut self, position: &Position) -> &'_ mut TripleTailoredModelGraphNode {
        self.nodes[position.t][position.i][position.j].as_mut().unwrap()
    }

    /// build model graph given the simulator
    pub fn build(&mut self, simulator: &mut Simulator, error_model: &ErrorModel, weight_function: &WeightFunction) {
        match weight_function {
            WeightFunction::Autotune => self.build_with_weight_function(simulator, error_model, weight_function::autotune),
            WeightFunction::AutotuneImproved => self.build_with_weight_function(simulator, error_model, weight_function::autotune_improved),
            WeightFunction::Unweighted => self.build_with_weight_function(simulator, error_model, weight_function::unweighted),
        }
    }

    /// build model graph given the simulator with customized weight function
    pub fn build_with_weight_function<F>(&mut self, simulator: &mut Simulator, error_model: &ErrorModel, weight_of: F) where F: Fn(f64) -> f64 + Copy {
        debug_assert!({
            let mut state_clean = true;
            simulator_iter!(simulator, position, node, {
                // here I omitted the condition `t % measurement_cycles == 0` for a stricter check
                if position.t != 0 && node.gate_type.is_measurement() {
                    let [positive_node, negative_node, neutral_node] = self.get_node_unwrap(position);
                    if positive_node.all_edges.len() > 0 || positive_node.edges.len() > 0 {
                        state_clean = false;
                    }
                    if negative_node.all_edges.len() > 0 || negative_node.edges.len() > 0 {
                        state_clean = false;
                    }
                    if neutral_node.all_edges.len() > 0 || neutral_node.edges.len() > 0 {
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
                    let sparse_measurement: Vec<&Position> = sparse_measurement_real.iter().chain(sparse_measurement_virtual.iter()).collect();
                    assert!(sparse_measurement.len() == 2 || sparse_measurement.len() == 4, "I don't know how to handle other cases, so strictly check it");
                    if sparse_measurement.len() == 2 {
                        let position1 = &sparse_measurement[0];
                        let position2 = &sparse_measurement[1];
                        let node1 = simulator.get_node_unwrap(position1);
                        let node2 = simulator.get_node_unwrap(position2);
                        debug_assert!({
                            // when considering virtual nodes, qubit type should be the same (correct me if it's wrong)
                            node1.qubit_type == node2.qubit_type
                        });
                        if p > 0. || is_erasure {
                            self.add_edge_between(position1, position2, p, weight_of(p), sparse_errors.clone(), sparse_correction.clone());
                        }
                    }
                    if sparse_measurement.len() == 4 {  // tailored edges
                        // tailored surface code decoding method can handle special cases arXiv:1907.02554v2
                        // first find the individual median i and j, then (i, j) must be the center data qubit
                        let mut vec_i = Vec::new();
                        let mut vec_j = Vec::new();
                        for &&Position{ i: im, j: jm, .. } in sparse_measurement.iter() {
                            vec_i.push(im);
                            vec_j.push(jm);
                        }
                        let center_i = util::find_strict_one_median(&mut vec_i);
                        let center_j = util::find_strict_one_median(&mut vec_j);
                        let mut unknown_case_warning = false;
                        match (center_i, center_j) {
                            (Some(center_i), Some(center_j)) => {
                                let mut up = None;
                                let mut down = None;
                                let mut left = None;
                                let mut right = None;
                                let mut counter = 0;
                                for &&Position{ i: im, j: jm, t: tm } in sparse_measurement.iter() {
                                    if im + 1 == center_i && jm == center_j {
                                        up = Some((tm, im, jm));
                                        counter += 1;
                                    }
                                    if im == center_i + 1 && jm == center_j {
                                        down = Some((tm, im, jm));
                                        counter += 1;
                                    }
                                    if im == center_i && jm == center_j + 1 {
                                        right = Some((tm, im, jm));
                                        counter += 1;
                                    }
                                    if im == center_i && jm + 1 == center_j {
                                        left = Some((tm, im, jm));
                                        counter += 1;
                                    }
                                }
                                if counter == sparse_measurement.len() {
                                    // add them to `tailored_positive_edges` and `tailored_negative_edges`
                                    {  // positive: up + right, left + down
                                        for (a, b) in [(up, right), (left, down)] {
                                            match (a, b) {
                                                (Some((t1, i1, j1)), Some((t2, i2, j2))) => {
                                                    // println!("add_edge_case tailored_positive_edges [{}][{}][{}] [{}][{}][{}] with p = {}", t1, i1, j1, t2, i2, j2, p);
                                                    self.add_positive_edge_between(&pos!(t1, i1, j1), &pos!(t2, i2, j2), p, weight_of(p), sparse_errors.clone(), sparse_correction.clone());
                                                }
                                                _ => { unreachable!() }
                                            }
                                        }
                                    }
                                    {  // negative: left + up, down + right
                                        for (a, b) in [(left, up), (down, right)] {
                                            match (a, b) {
                                                (Some((t1, i1, j1)), Some((t2, i2, j2))) => {
                                                    // println!("add_edge_case tailored_negative_edges [{}][{}][{}] [{}][{}][{}] with p = {}", t1, i1, j1, t2, i2, j2, p);
                                                    self.add_negative_edge_between(&pos!(t1, i1, j1), &pos!(t2, i2, j2), p, weight_of(p), sparse_errors.clone(), sparse_correction.clone());
                                                }
                                                _ => { unreachable!() }
                                            }
                                        }
                                    }
                                } else {
                                    unknown_case_warning = true;  // cannot fit them in
                                }
                            }
                            _ => {
                                unknown_case_warning = true;
                            }
                        }
                        if unknown_case_warning {
                            // this cases seem to be normal for circuit-level noise model of tailored surface code: Pauli Y would generate some strange cases, but those are low-biased errors
                            // println!("[warning ]error at {} {}: cannot recognize the pattern of this 4 non-trivial measurements, skipped", position, error);
                            // for position in sparse_measurement.iter() {
                            //     print!("{}, ", position);
                            // }
                            // println!("");
                        }
                    }
                }
            }
        });
        self.elect_edges(simulator, true, weight_of);  // by default use combined probability
    }

    /// add asymmetric edge from `source` to `target` in positive direction; in order to create symmetric edge, call this function twice with reversed input
    pub fn add_one_edge(&mut self, source: &Position, target: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>, correction: Arc<SparseCorrection>, idx: usize) {
        let node = &mut self.get_node_mut_unwrap(source)[idx];
        if !node.all_edges.contains_key(target) {
            node.all_edges.insert(target.clone(), Vec::new());
        }
        node.all_edges.get_mut(target).unwrap().push(TailoredModelGraphEdge {
            probability: probability,
            weight: weight,
            error_pattern: error_pattern,
            correction: correction,
        })
    }

    /// add asymmetric edge from `source` to `target` in positive direction; in order to create symmetric edge, call this function twice with reversed input
    pub fn add_positive_edge(&mut self, source: &Position, target: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>, correction: Arc<SparseCorrection>) {
        self.add_one_edge(source, target, probability, weight, error_pattern.clone(), correction.clone(), 0);
    }

    /// add symmetric edge between `source` and `target` in positive direction
    pub fn add_positive_edge_between(&mut self, position1: &Position, position2: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>
            , correction: Arc<SparseCorrection>) {
        self.add_positive_edge(position1, position2, probability, weight, error_pattern.clone(), correction.clone());
        self.add_positive_edge(position2, position1, probability, weight, error_pattern.clone(), correction.clone());
    }

    /// add asymmetric edge from `source` to `target` in negative direction; in order to create symmetric edge, call this function twice with reversed input
    pub fn add_negative_edge(&mut self, source: &Position, target: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>, correction: Arc<SparseCorrection>) {
        self.add_one_edge(source, target, probability, weight, error_pattern.clone(), correction.clone(), 1);
    }

    /// add symmetric edge between `source` and `target` in negative direction
    pub fn add_negative_edge_between(&mut self, position1: &Position, position2: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>
            , correction: Arc<SparseCorrection>) {
        self.add_negative_edge(position1, position2, probability, weight, error_pattern.clone(), correction.clone());
        self.add_negative_edge(position2, position1, probability, weight, error_pattern.clone(), correction.clone());
    }

    /// add asymmetric edge from `source` to `target` in negative direction; in order to create symmetric edge, call this function twice with reversed input
    pub fn add_neutral_edge(&mut self, source: &Position, target: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>, correction: Arc<SparseCorrection>) {
        self.add_one_edge(source, target, probability, weight, error_pattern.clone(), correction.clone(), 2);
    }

    /// add symmetric edge between `source` and `target` in negative direction
    pub fn add_neutral_edge_between(&mut self, position1: &Position, position2: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>
            , correction: Arc<SparseCorrection>) {
        self.add_neutral_edge(position1, position2, probability, weight, error_pattern.clone(), correction.clone());
        self.add_neutral_edge(position2, position1, probability, weight, error_pattern.clone(), correction.clone());
    }

    /// add asymmetric edge from `source` to `target`; in order to create symmetric edge, call this function twice with reversed input
    pub fn add_edge_between(&mut self, position1: &Position, position2: &Position, probability: f64, weight: f64, error_pattern: Arc<SparseErrorPattern>, correction: Arc<SparseCorrection>) {
        self.add_positive_edge_between(position1, position2, probability, weight, error_pattern.clone(), correction.clone());
        self.add_negative_edge_between(position2, position1, probability, weight, error_pattern.clone(), correction.clone());
        self.add_neutral_edge_between(position2, position1, probability, weight, error_pattern.clone(), correction.clone());
    }

    /// if there are multiple edges connecting two stabilizer measurements, elect the best one
    pub fn elect_edges<F>(&mut self, simulator: &Simulator, use_combined_probability: bool, weight_of: F) where F: Fn(f64) -> f64 + Copy {
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if self.is_node_exist(position) {
            let [positive_node, negative_node, neutral_node] = self.get_node_mut_unwrap(position);
            // elect edges
            for node in [positive_node, negative_node, neutral_node] {
                for (target, edges) in node.all_edges.iter() {
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
                    let elected = TailoredModelGraphEdge {
                        probability: elected_probability,
                        weight: weight_of(elected_probability),
                        error_pattern: edges[elected_idx].error_pattern.clone(),
                        correction: edges[elected_idx].correction.clone(),
                    };
                    // update elected edge
                    // println!("{} to {} elected probability: {}", position, target, elected.probability);
                    node.edges.insert(target.clone(), elected);
                }
            }
        });
        // sanity check, two nodes on one edge have the same edge information, should be a cheap sanity check
        debug_assert!({
            let mut sanity_check_passed = true;
            for t in (simulator.measurement_cycles..simulator.height).step_by(simulator.measurement_cycles) {
                simulator_iter!(simulator, position, node, t => t, if node.gate_type.is_measurement() {
                    for idx in 0..3 {
                        let node = &self.get_node_unwrap(position)[idx];  // idx = 0: positive, 1: negative
                        for (target, edge) in node.edges.iter() {
                            let target_node = &self.get_node_unwrap(target)[idx];  // idx = 0: positive, 1: negative
                            let reverse_edge = target_node.edges.get(position).expect("edge should be symmetric");
                            if !float_cmp::approx_eq!(f64, edge.probability, reverse_edge.probability, ulps = 5) {
                                println!("[warning] the edge between {} and {} has unequal probability {} and {}"
                                    , position, target, edge.probability, reverse_edge.probability);
                                sanity_check_passed = false;
                            }
                        }
                    }
                });
            }
            sanity_check_passed
        });
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
                            let [positive_node, negative_node, neutral_node] = self.get_node_unwrap(position);
                            Some(json!({
                                "position": position,
                                "all_positive_edges": positive_node.all_edges,
                                "positive_edges": positive_node.edges,
                                "all_negative_edges": negative_node.all_edges,
                                "negative_edges": negative_node.edges,
                                "all_neutral_edges": neutral_node.all_edges,
                                "neutral_edges": neutral_node.edges,
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
