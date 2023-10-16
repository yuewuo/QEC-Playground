//! build model graph from simulator and measurement results
//!

use super::either::Either;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use super::types::*;
use super::util_macros::*;
use super::visualize::*;
#[cfg(feature = "hyperion")]
use mwpf::util::HyperEdge;
#[cfg(feature = "python_binding")]
use pyo3::prelude::*;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// edges connecting two nontrivial measurements generated by a single error
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct ModelHypergraph {
    /// vertex index
    pub vertex_indices: HashMap<Position, usize>,
    /// edge index
    pub edge_indices: HashMap<DefectVertices, usize>,
    /// get position by vertex index
    pub vertex_positions: Vec<Position>,
    /// all the weighted edges indexed by edge_index
    pub weighted_edges: Vec<(DefectVertices, ModelHyperedgeGroup)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefectVertices(Vec<Position>);

impl DefectVertices {
    pub fn new(mut defect_vertices: Vec<Position>) -> Self {
        assert!(!defect_vertices.is_empty(), "defect vertices cannot be empty");
        defect_vertices.sort();
        Self(defect_vertices)
    }
}

impl Serialize for DefectVertices {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut formatted = format!("{}", self.0[0]);
        for i in 1..self.0.len() {
            formatted += format!("+{}", self.0[i]).as_str();
        }
        serializer.serialize_str(formatted.as_str())
    }
}

pub struct DefectVerticesVisitor {}

impl<'de> Visitor<'de> for DefectVerticesVisitor {
    type Value = DefectVertices;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            r#"defect vertices should look like "[0][10][13]+[0][10][15]+[6][10][13]""#
        )
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let split = s.split('+');
        let mut positions = vec![];
        for position in split {
            let position_visitor = PositionVisitor {};
            positions.push(position_visitor.visit_str(position)?);
        }
        if positions.is_empty() {
            return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self));
        }
        Ok(DefectVertices::new(positions))
    }
}

impl<'de> Deserialize<'de> for DefectVertices {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // the new-ed position just works like a helper type that implements Visitor trait, not optimized for efficiency
        deserializer.deserialize_str(DefectVerticesVisitor {})
    }
}

impl Ord for DefectVertices {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.len().cmp(&other.0.len()) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => {
                for i in 0..self.0.len() {
                    if self.0[i] > other.0[i] {
                        return Ordering::Greater;
                    }
                    if self.0[i] < other.0[i] {
                        return Ordering::Less;
                    }
                }
                Ordering::Equal
            }
        }
    }
}

impl PartialOrd for DefectVertices {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl QecpVisualizer for ModelHypergraph {
    fn component_info(&self, abbrev: bool) -> (String, serde_json::Value) {
        let name = "model_hypergraph";
        let info = json!({
            "vertex_indices": self.vertex_indices,
            "edge_indices": self.edge_indices,
            "vertex_positions": self.vertex_positions,
            "weighted_edges": self.weighted_edges.iter().map(|(defect_vertices, hyperedge_group)| {
                (defect_vertices, json!({
                    "all_hyperedges": hyperedge_group.all_hyperedges.iter().map(|hyperedge| {
                        hyperedge.component_edge_info(abbrev)
                    }).collect::<Vec<_>>(),
                    "hyperedge": hyperedge_group.hyperedge.component_edge_info(abbrev),
                }))
            }).collect::<Vec<_>>(),
        });
        (name.to_string(), info)
    }
}

/// only defined for measurement nodes (including virtual measurement nodes)
#[derive(Debug, Clone, Serialize)]
pub struct ModelHyperedgeGroup {
    /// used when building the hypergraph, record all possible hyperedges
    /// (this might be dropped to save memory usage after election)
    pub all_hyperedges: Vec<ModelHyperedge>,
    /// the elected edges, to make sure each pair of nodes only have one edge
    pub hyperedge: ModelHyperedge,
}

impl ModelHyperedgeGroup {
    pub fn new(hyperedge: ModelHyperedge) -> Self {
        Self {
            all_hyperedges: vec![hyperedge.clone()],
            hyperedge,
        }
    }
    pub fn add<F>(&mut self, hyperedge: ModelHyperedge, use_combined_probability: bool, use_brief_edge: bool, weight_of: F)
    where
        F: Fn(f64) -> f64 + Copy,
    {
        let is_new_edge_better = hyperedge.probability > self.hyperedge.probability;
        let new_probability = if use_combined_probability {
            hyperedge.probability * (1. - self.hyperedge.probability)
                + self.hyperedge.probability * (1. - hyperedge.probability) // XOR
        } else if is_new_edge_better {
            hyperedge.probability
        } else {
            self.hyperedge.probability
        };
        if is_new_edge_better {
            self.hyperedge = hyperedge.clone();
        }
        if use_brief_edge {
            // only keep one hyperedge in the list
            if is_new_edge_better {
                self.all_hyperedges[0] = hyperedge;
            }
        } else {
            // keep all hyperedges
            self.all_hyperedges.push(hyperedge);
        }
        self.hyperedge.probability = new_probability;
        self.hyperedge.weight = weight_of(new_probability);
    }
    pub fn merge<F>(&mut self, other: Self, use_combined_probability: bool, use_brief_edge: bool, weight_of: F)
    where
        F: Fn(f64) -> f64 + Copy,
    {
        for hyperedge in other.all_hyperedges.into_iter() {
            self.add(hyperedge, use_combined_probability, use_brief_edge, weight_of);
        }
    }
}

/// without concrete correction, can be used to save memory but not all error pattern will be recorded
#[derive(Debug, Clone, Serialize)]
pub struct ModelHyperedge {
    /// the probability of this edge to happen
    pub probability: f64,
    /// the weight of this edge computed by the (combined) probability, e.g. ln((1-p)/p)
    pub weight: f64,
    /// the error that causes this edge
    pub error_pattern: Arc<SparseErrorPattern>,
    /// the correction pattern that can recover this error
    pub correction: Arc<SparseCorrection>,
}

impl ModelHyperedge {
    fn component_edge_info(&self, abbrev: bool) -> serde_json::Value {
        json!({
            if abbrev { "p" } else { "probability" }: self.probability,
            if abbrev { "w" } else { "weight" }: self.weight,
            if abbrev { "e" } else { "error_pattern" }: self.error_pattern,
            if abbrev { "c" } else { "correction" }: self.correction,
        })
    }
}

impl ModelHypergraph {
    /// initialize the structure corresponding to a `Simulator`
    pub fn new(simulator: &Simulator) -> Self {
        assert!(simulator.volume() > 0, "cannot build model graph out of zero-sized simulator");
        Self {
            vertex_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            vertex_positions: Vec::new(),
            weighted_edges: Vec::new(),
        }
    }

    /// build model hypergraph given the simulator
    pub fn build(
        &mut self,
        simulator: &mut Simulator,
        noise_model: Arc<NoiseModel>,
        weight_function: &WeightFunction,
        parallel: usize,
        use_combined_probability: bool,
        use_brief_edge: bool,
    ) {
        match weight_function {
            WeightFunction::Autotune => self.build_with_weight_function(
                simulator,
                noise_model,
                weight_function::autotune,
                parallel,
                use_combined_probability,
                use_brief_edge,
            ),
            WeightFunction::AutotuneImproved => self.build_with_weight_function(
                simulator,
                noise_model,
                weight_function::autotune_improved,
                parallel,
                use_combined_probability,
                use_brief_edge,
            ),
            WeightFunction::Unweighted => self.build_with_weight_function(
                simulator,
                noise_model,
                weight_function::unweighted,
                parallel,
                use_combined_probability,
                use_brief_edge,
            ),
        }
    }

    /// single-thread computation with region
    fn build_with_weight_function_region<F>(
        &mut self,
        simulator: &mut Simulator,
        noise_model: Arc<NoiseModel>,
        weight_of: F,
        t_range: (usize, usize),
        use_combined_probability: bool,
        use_brief_edge: bool,
    ) where
        F: Fn(f64) -> f64 + Copy,
    {
        let (t_start, t_end) = t_range;
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
            for error in all_possible_errors.iter() {
                let p = match error {
                    Either::Left(error_type) => noise_model_node.pauli_error_rates.error_rate(error_type),
                    Either::Right(error_type) => match &noise_model_node.correlated_pauli_error_rates {
                        Some(correlated_pauli_error_rates) => correlated_pauli_error_rates.error_rate(error_type),
                        None => 0.,
                    },
                }; // probability of this error to occur
                let is_erasure = possible_erasure_error && error.is_left();
                if p > 0. || is_erasure {
                    // use possible errors to build `all_edges`
                    // simulate the error and measure it
                    let mut sparse_errors = SparseErrorPattern::new();
                    match error {
                        Either::Left(error_type) => {
                            sparse_errors.add(position.clone(), *error_type);
                        }
                        Either::Right(error_type) => {
                            sparse_errors.add(position.clone(), error_type.my_error());
                            let node = simulator.get_node_unwrap(position);
                            let gate_peer = node
                                .gate_peer
                                .as_ref()
                                .expect("correlated error must corresponds to a two-qubit gate");
                            sparse_errors.add((**gate_peer).clone(), error_type.peer_error());
                        }
                    }
                    let sparse_errors = Arc::new(sparse_errors); // make it immutable and shared
                    let (sparse_correction, sparse_measurement, _) =
                        simulator.fast_measurement_given_few_errors(&sparse_errors);
                    let sparse_correction = Arc::new(sparse_correction); // make it immutable and shared
                    let sparse_measurement = sparse_measurement.to_vec();
                    if sparse_measurement.is_empty() {
                        // no way to detect it, ignore
                        continue;
                    }
                    // println!("{:?} at {} will cause syndrome {:?}", error, position, sparse_measurement);
                    for position in sparse_measurement.iter() {
                        if !self.vertex_indices.contains_key(position) {
                            self.vertex_indices.insert(position.clone(), self.vertex_positions.len());
                            self.vertex_positions.push(position.clone());
                        }
                    }
                    let defect_vertices = DefectVertices::new(sparse_measurement);
                    let model_hyperedge = ModelHyperedge {
                        probability: p,
                        weight: weight_of(p),
                        error_pattern: sparse_errors.clone(),
                        correction: sparse_correction.clone(),
                    };
                    if self.edge_indices.contains_key(&defect_vertices) {
                        let edge_index = self.edge_indices.get(&defect_vertices).unwrap();
                        self.weighted_edges[*edge_index].1.add(
                            model_hyperedge,
                            use_combined_probability,
                            use_brief_edge,
                            weight_of,
                        );
                    } else {
                        self.edge_indices.insert(defect_vertices.clone(), self.weighted_edges.len());
                        self.weighted_edges
                            .push((defect_vertices, ModelHyperedgeGroup::new(model_hyperedge)));
                    }
                }
            }
        });
    }

    /// build model graph given the simulator with customized weight function;
    /// if `optimize_memory_usage` is set to True, then not all edges are recorded but only the optimal one
    pub fn build_with_weight_function<F>(
        &mut self,
        simulator: &mut Simulator,
        noise_model: Arc<NoiseModel>,
        weight_of: F,
        parallel: usize,
        use_combined_probability: bool,
        use_brief_edge: bool,
    ) where
        F: Fn(f64) -> f64 + Copy + Send + Sync + 'static,
    {
        debug_assert!(self.vertex_indices.is_empty(), "must be clean");
        debug_assert!(self.edge_indices.is_empty(), "must be clean");
        debug_assert!(self.vertex_positions.is_empty(), "must be clean");
        debug_assert!(self.weighted_edges.is_empty(), "must be clean");
        if parallel <= 1 {
            self.build_with_weight_function_region(
                simulator,
                noise_model,
                weight_of,
                (0, simulator.height),
                use_combined_probability,
                use_brief_edge,
            );
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
                    instance.build_with_weight_function_region(
                        &mut simulator,
                        noise_model,
                        weight_of,
                        (t_start, t_end),
                        use_combined_probability,
                        use_brief_edge,
                    );
                }));
            }
            for handler in handlers.drain(..) {
                handler.join().unwrap();
            }
            // move the data from instances (without additional large memory allocation)
            for instance in instances.iter() {
                let mut instance = instance.lock().unwrap();
                // copy vertex positions
                for position in instance.vertex_positions.iter() {
                    if !self.vertex_indices.contains_key(position) {
                        self.vertex_indices.insert(position.clone(), self.vertex_positions.len());
                        self.vertex_positions.push(position.clone());
                    }
                }
                // copy edges
                for (defect_vertices, hyperedge_group) in instance.weighted_edges.drain(..) {
                    if self.edge_indices.contains_key(&defect_vertices) {
                        let edge_index = self.edge_indices.get(&defect_vertices).unwrap();
                        self.weighted_edges[*edge_index].1.merge(
                            hyperedge_group,
                            use_combined_probability,
                            use_brief_edge,
                            weight_of,
                        );
                    } else {
                        self.edge_indices.insert(defect_vertices.clone(), self.weighted_edges.len());
                        self.weighted_edges.push((defect_vertices, hyperedge_group));
                    }
                }
            }
        }
    }

    #[cfg(feature = "hyperion")]
    pub fn generate_mwpf_hypergraph(&self, max_weight: usize) -> (usize, Vec<HyperEdge>) {
        // scale all the edges
        let mut maximum_weight = 0.;
        for (_, hyperedge_group) in self.weighted_edges.iter() {
            if hyperedge_group.hyperedge.probability > 0. && hyperedge_group.hyperedge.weight > maximum_weight {
                maximum_weight = hyperedge_group.hyperedge.weight;
            }
        }
        let mut weighted_edges = Vec::with_capacity(self.weighted_edges.len());
        for (defect_vertices, hyperedge_group) in self.weighted_edges.iter() {
            if hyperedge_group.hyperedge.probability > 0. {
                // only add those possible edges; for erasures, handle later
                let scaled_weight = hyperedge_group.hyperedge.weight * max_weight as f64 / maximum_weight;
                let int_weight = scaled_weight.round();
                assert!(int_weight.is_finite(), "weight must be normal");
                assert!(int_weight >= 0., "weight must be non-negative");
                assert!(int_weight <= max_weight as f64, "weight must be smaller than max weight");
                let vertex_indices: Vec<_> = defect_vertices.0.iter().map(|x| self.vertex_indices[x]).collect();
                weighted_edges.push(HyperEdge::new(vertex_indices, int_weight as usize));
            }
        }
        (self.vertex_positions.len(), weighted_edges)
    }

    /// create json object for debugging and viewing
    pub fn to_json(&self, simulator: &Simulator) -> serde_json::Value {
        json!({
            "code_type": simulator.code_type,
            "height": simulator.height,
            "vertical": simulator.vertical,
            "horizontal": simulator.horizontal,
        })
    }
}
