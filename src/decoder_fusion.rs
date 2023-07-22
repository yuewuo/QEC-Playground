//! minimum-weight perfect matching decoder
//!

use super::model_graph::*;
use super::noise_model::*;
use super::serde_json;
use super::simulator::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
// use super::erasure_graph::*;
use super::decoder_mwpm::*;
use super::derivative::*;
use super::fusion_blossom;
use super::fusion_blossom::mwpm_solver::PrimalDualSolver;
use crate::fusion_blossom::mwpm_solver::*;
use crate::fusion_blossom::util::*;
use crate::fusion_blossom::visualize::*;
use crate::types::QubitType;
use crate::util_macros::*;

/// MWPM decoder based on fusion blossom algorithm, initialized and cloned for multiple threads
#[derive(Derivative, Serialize)]
#[derivative(Debug)]
pub struct FusionDecoder {
    /// shared data helps interface with the fusion blossom algorithm
    pub adaptor: Arc<FusionBlossomAdaptor>,
    /// fusion blossom algorithm: a fast MWPM solver for quantum error correction
    #[serde(skip)]
    #[derivative(Debug = "ignore")]
    pub fusion_solver: fusion_blossom::mwpm_solver::SolverSerial,
    /// save configuration for later usage
    pub config: FusionDecoderConfig,
}

impl Clone for FusionDecoder {
    fn clone(&self) -> Self {
        // construct a new solver instance
        let fusion_solver = if self.config.skip_decoding {
            fusion_blossom::mwpm_solver::SolverSerial::new(&SolverInitializer {
                vertex_num: 0,
                weighted_edges: vec![],
                virtual_vertices: vec![],
            })
        } else {
            fusion_blossom::mwpm_solver::SolverSerial::new(&self.adaptor.initializer)
        };
        Self {
            adaptor: self.adaptor.clone(),
            fusion_solver,
            config: self.config.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FusionDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")] // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")] // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    /// only export Z stabilizers
    #[serde(default = "fusion_default_configs::only_stab_z")]
    pub only_stab_z: bool,
    #[serde(alias = "mhw")] // abbreviation
    #[serde(default = "fusion_default_configs::max_half_weight")]
    pub max_half_weight: usize,
    #[serde(default = "fusion_default_configs::skip_decoding")]
    pub skip_decoding: bool,
    #[serde(default = "fusion_default_configs::log_matchings")]
    pub log_matchings: bool,
}

pub mod fusion_default_configs {
    pub fn only_stab_z() -> bool {
        false
    }
    pub fn max_half_weight() -> usize {
        5000
    }
    pub fn skip_decoding() -> bool {
        false
    }
    pub fn log_matchings() -> bool {
        false
    }
}

impl FusionDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(
        simulator: &Simulator,
        noise_model: Arc<NoiseModel>,
        decoder_configuration: &serde_json::Value,
        parallel: usize,
        use_brief_edge: bool,
    ) -> Self {
        // read attribute of decoder configuration
        let config: FusionDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        let mut simulator = simulator.clone();
        // // build erasure graph
        // let mut erasure_graph = ErasureGraph::new(&simulator);
        // erasure_graph.build(&mut simulator, Arc::clone(&noise_model), parallel);
        // let erasure_graph = Arc::new(erasure_graph);
        // build solver
        let adaptor = FusionBlossomAdaptor::new(&config, &mut simulator, noise_model, parallel, use_brief_edge);
        let fusion_solver = fusion_blossom::mwpm_solver::SolverSerial::new(&adaptor.initializer);
        Self {
            adaptor: Arc::new(adaptor),
            fusion_solver,
            config,
        }
    }

    /// decode given measurement results
    #[allow(dead_code)]
    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        self.decode_with_erasure(sparse_measurement, &SparseErasures::new())
    }

    /// decode given measurement results and detected erasures
    pub fn decode_with_erasure(
        &mut self,
        sparse_measurement: &SparseMeasurement,
        sparse_detected_erasures: &SparseErasures,
    ) -> (SparseCorrection, serde_json::Value) {
        if self.config.skip_decoding {
            return (SparseCorrection::new(), json!({}));
        }
        assert!(sparse_detected_erasures.is_empty(), "fusion decoder doesn't support erasure error yet: we'll do it in the next version to support 0-weight edges and dynamic setting");
        let mut correction = SparseCorrection::new();
        let mut time_fusion = 0.;
        let mut time_build_correction = 0.;
        let mut log_matchings = Vec::with_capacity(0);
        // list nontrivial measurements to be matched
        if !sparse_measurement.is_empty() {
            // run the Blossom algorithm
            let begin = Instant::now();
            let syndrome_pattern = self
                .adaptor
                .generate_syndrome_pattern(sparse_measurement, sparse_detected_erasures);
            self.fusion_solver.solve(&syndrome_pattern);
            let subgraph: Vec<usize> = self.fusion_solver.subgraph();
            if self.config.log_matchings {
                // log the subgraph
                let mut subgraph_edges = vec![];
                for &edge_index in subgraph.iter() {
                    let (vertex_1, vertex_2, _) = self.adaptor.initializer.weighted_edges[edge_index];
                    let position_1 = self.adaptor.vertex_to_position_mapping[vertex_1].clone();
                    let position_2 = self.adaptor.vertex_to_position_mapping[vertex_2].clone();
                    subgraph_edges.push((position_1, position_2));
                }
                log_matchings.push(json!({
                    "name": "subgraph",
                    "description": "elementary fault edges",
                    "edges": subgraph_edges,
                }));
                // also log the perfect matching
                let mut perfect_matching_edges = vec![];
                let perfect_matching = self.fusion_solver.perfect_matching();
                for (node_ptr_1, node_ptr_2) in perfect_matching.peer_matchings.iter() {
                    let vertex_1 = node_ptr_1.get_representative_vertex();
                    let vertex_2 = node_ptr_2.get_representative_vertex();
                    let position_1 = self.adaptor.vertex_to_position_mapping[vertex_1].clone();
                    let position_2 = self.adaptor.vertex_to_position_mapping[vertex_2].clone();
                    perfect_matching_edges.push((position_1, position_2));
                }
                for (node_ptr, virtual_vertex) in perfect_matching.virtual_matchings.iter() {
                    let vertex = node_ptr.get_representative_vertex();
                    let position_1 = self.adaptor.vertex_to_position_mapping[vertex].clone();
                    let position_2 = self.adaptor.vertex_to_position_mapping[*virtual_vertex].clone();
                    perfect_matching_edges.push((position_1, position_2));
                }
                log_matchings.push(json!({
                    "name": "perfect matching",
                    "description": "the paths of the perfect matching",
                    "edges": perfect_matching_edges,
                }));
            }
            self.fusion_solver.clear();
            time_fusion += begin.elapsed().as_secs_f64();
            correction = self.adaptor.subgraph_to_correction(&subgraph);
            time_build_correction += begin.elapsed().as_secs_f64();
        }
        let mut runtime_statistics = json!({
            "to_be_matched": sparse_measurement.len(),
            "time_fusion": time_fusion,
            "time_build_correction": time_build_correction,
        });
        if self.config.log_matchings {
            let runtime_statistics = runtime_statistics.as_object_mut().unwrap();
            runtime_statistics.insert("log_matchings".to_string(), json!(log_matchings));
        }
        (correction, runtime_statistics)
    }
}

// pub type PositionToVertexMap = std::collections::HashMap<Position, usize>;
pub type PositionToVertexMap = std::collections::BTreeMap<Position, usize>;

/// adaptor that connects the `Simulator` with data structures for fusion blossom
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FusionBlossomAdaptor {
    /// filter stabilizers
    pub stabilizer_filter: FusionBlossomStabilizerFilter,
    /// vertex index map to position
    pub vertex_to_position_mapping: Vec<Position>,
    /// position map to vertex index
    pub position_to_vertex_mapping: PositionToVertexMap,
    /// edge map to correction
    pub edge_to_correction_mapping: Vec<SparseCorrection>,
    /// fusion blossom initializer
    pub initializer: SolverInitializer,
    /// fusion blossom position for visualization
    pub positions: Vec<VisualizePosition>,
}

impl FusionBlossomAdaptor {
    pub fn new(
        config: &FusionDecoderConfig,
        simulator: &mut Simulator,
        noise_model_graph: Arc<NoiseModel>,
        parallel_init: usize,
        use_brief_edge: bool,
    ) -> Self {
        let mut model_graph = ModelGraph::new(simulator);
        model_graph.build(
            simulator,
            noise_model_graph,
            &config.weight_function,
            parallel_init,
            config.use_combined_probability,
            use_brief_edge,
        );
        let stabilizer_filter = if config.only_stab_z {
            FusionBlossomStabilizerFilter::StabZOnly
        } else {
            FusionBlossomStabilizerFilter::None
        };
        // generate model graph, and then generate a graph initializer and positions
        // when the syndrome is partial: e.g. only StabX or only StabZ, the graph initializer should also contain only part of it
        let mut initializer: SolverInitializer = SolverInitializer::new(0, vec![], vec![]);
        let mut positions: Vec<VisualizePosition> = vec![];
        let mut vertex_to_position_mapping = vec![];
        let mut position_to_vertex_mapping = PositionToVertexMap::new();
        simulator_iter!(simulator, position, node, {
            // first insert nodes and build mapping, different from decoder_fusion, here we add virtual nodes as well
            if position.t != 0 && node.gate_type.is_measurement() && !stabilizer_filter.ignore_node(node) {
                let vertex_index = initializer.vertex_num;
                if !simulator.is_node_real(position) {
                    initializer.virtual_vertices.push(vertex_index);
                }
                positions.push(VisualizePosition::new(
                    position.i as f64,
                    position.j as f64,
                    position.t as f64 / simulator.measurement_cycles as f64 * 2.,
                ));
                vertex_to_position_mapping.push(position.clone());
                position_to_vertex_mapping.insert(position.clone(), vertex_index);
                initializer.vertex_num += 1;
            }
        });
        let mut weighted_edges_unscaled = Vec::<(usize, usize, f64)>::new();
        let mut edge_to_correction_mapping = Vec::new();
        simulator_iter!(simulator, position, node, {
            // then add edges and also virtual nodes
            if position.t != 0
                && node.gate_type.is_measurement()
                && simulator.is_node_real(position)
                && !stabilizer_filter.ignore_node(node)
            {
                let model_graph_node = model_graph.get_node_unwrap(position);
                let vertex_index = position_to_vertex_mapping[position];
                if let Some(model_graph_boundary) = &model_graph_node.boundary {
                    let virtual_position = model_graph_boundary
                        .virtual_node
                        .as_ref()
                        .expect("virtual boundary required to plot properly in fusion blossom");
                    let virtual_index = position_to_vertex_mapping[virtual_position];
                    weighted_edges_unscaled.push((vertex_index, virtual_index, model_graph_boundary.weight));
                    edge_to_correction_mapping.push(model_graph_boundary.correction.as_ref().clone());
                }
                for (peer_position, model_graph_edge) in model_graph_node.edges.iter() {
                    let peer_idx = position_to_vertex_mapping[peer_position];
                    if vertex_index < peer_idx {
                        // avoid duplicate edges
                        weighted_edges_unscaled.push((vertex_index, peer_idx, model_graph_edge.weight));
                        edge_to_correction_mapping.push(model_graph_edge.correction.as_ref().clone());
                    }
                }
            }
        });
        initializer.weighted_edges = {
            // re-weight edges and parse to integer
            let mut maximum_weight = 0.;
            for (_, _, weight) in weighted_edges_unscaled.iter() {
                if weight > &maximum_weight {
                    maximum_weight = *weight;
                }
            }
            let scale: f64 = config.max_half_weight as f64 / maximum_weight;
            weighted_edges_unscaled
                .iter()
                .map(|(a, b, weight)| (*a, *b, 2 * (weight * scale).ceil() as fusion_blossom::util::Weight))
                .collect()
        };
        Self {
            initializer,
            positions,
            vertex_to_position_mapping,
            position_to_vertex_mapping,
            stabilizer_filter,
            edge_to_correction_mapping,
        }
    }

    pub fn generate_syndrome_pattern(
        &self,
        sparse_measurement: &SparseMeasurement,
        sparse_detected_erasures: &SparseErasures,
    ) -> SyndromePattern {
        assert!(sparse_detected_erasures.is_empty(), "erasure not implemented");
        let mut syndrome_pattern = SyndromePattern::new_empty();
        for defect_vertex in sparse_measurement.iter() {
            if self.position_to_vertex_mapping.contains_key(defect_vertex) {
                syndrome_pattern
                    .defect_vertices
                    .push(*self.position_to_vertex_mapping.get(defect_vertex).unwrap());
            }
        }
        syndrome_pattern
    }

    pub fn subgraph_to_correction(&self, subgraph: &[EdgeIndex]) -> SparseCorrection {
        let mut correction = SparseCorrection::new();
        for &edge_index in subgraph.iter() {
            correction.extend(&self.edge_to_correction_mapping[edge_index]);
        }
        correction
    }

    pub fn assert_eq(&self, other: &Self) -> Result<(), String> {
        if self.initializer.vertex_num != other.initializer.vertex_num {
            return Err(format!(
                "vertex_num differs {} != {}",
                self.initializer.vertex_num, other.initializer.vertex_num
            ));
        }
        if self.initializer.weighted_edges.len() != other.initializer.weighted_edges.len() {
            return Err(format!(
                "weighted edges length differs {} != {}",
                self.initializer.weighted_edges.len(),
                other.initializer.weighted_edges.len()
            ));
        }
        for index in 0..self.initializer.weighted_edges.len() {
            if self.initializer.weighted_edges[index] != other.initializer.weighted_edges[index] {
                return Err(format!(
                    "the {}-th weighted edge differs: {:?} != {:?}",
                    index, self.initializer.weighted_edges[index], other.initializer.weighted_edges[index]
                ));
            }
        }
        if self.initializer.virtual_vertices.len() != other.initializer.virtual_vertices.len() {
            return Err(format!(
                "virtual vertices length differs {} != {}",
                self.initializer.virtual_vertices.len(),
                other.initializer.virtual_vertices.len()
            ));
        }
        for index in 0..self.initializer.virtual_vertices.len() {
            if self.initializer.virtual_vertices[index] != other.initializer.virtual_vertices[index] {
                return Err(format!(
                    "the {}-th virtual vertex differs: {:?} != {:?}",
                    index, self.initializer.virtual_vertices[index], other.initializer.virtual_vertices[index]
                ));
            }
        }
        if self.positions.len() != other.positions.len() {
            return Err(format!(
                "positions length differs {} != {}",
                self.positions.len(),
                other.positions.len()
            ));
        }
        for index in 0..self.positions.len() {
            let pos1 = &self.positions[index];
            let pos2 = &other.positions[index];
            if pos1.t != pos2.t || pos1.i != pos2.i || pos1.j != pos2.j {
                return Err(format!("the {}-th position differs: {:?} != {:?}", index, pos1, pos2));
            }
        }
        if self.stabilizer_filter != other.stabilizer_filter {
            return Err("filter mismatch".to_string());
        }
        if self.vertex_to_position_mapping.len() != other.vertex_to_position_mapping.len() {
            return Err(format!(
                "vertex_to_position_mapping length differs {} != {}",
                self.vertex_to_position_mapping.len(),
                other.vertex_to_position_mapping.len()
            ));
        }
        for index in 0..self.vertex_to_position_mapping.len() {
            if self.vertex_to_position_mapping[index] != other.vertex_to_position_mapping[index] {
                return Err(format!(
                    "the {}-th position differs: {:?} != {:?}",
                    index, self.vertex_to_position_mapping[index], other.vertex_to_position_mapping[index]
                ));
            }
        }
        if self.position_to_vertex_mapping.len() != other.position_to_vertex_mapping.len() {
            return Err(format!(
                "position_to_vertex_mapping length differs {} != {}",
                self.position_to_vertex_mapping.len(),
                other.position_to_vertex_mapping.len()
            ));
        }
        debug_assert_eq!(
            self.position_to_vertex_mapping, other.position_to_vertex_mapping,
            "they should be equal"
        );
        if self.edge_to_correction_mapping.len() != other.edge_to_correction_mapping.len() {
            return Err(format!(
                "edge_to_correction_mapping length differs {} != {}",
                self.edge_to_correction_mapping.len(),
                other.edge_to_correction_mapping.len()
            ));
        }
        for index in 0..self.edge_to_correction_mapping.len() {
            let correction1 = &self.edge_to_correction_mapping[index];
            let correction2 = &other.edge_to_correction_mapping[index];
            if correction1.to_vec() != correction2.to_vec() {
                return Err(format!(
                    "the {}-th correction differs: {:?} != {:?}",
                    index, correction1, correction2
                ));
            }
        }
        debug_assert_eq!(
            self.initializer.weighted_edges, other.initializer.weighted_edges,
            "up to this step, they should be equal"
        );
        debug_assert_eq!(
            self.initializer.virtual_vertices, other.initializer.virtual_vertices,
            "up to this step, they should be equal"
        );
        debug_assert_eq!(
            self.vertex_to_position_mapping, other.vertex_to_position_mapping,
            "up to this step, they should be equal"
        );
        Ok(())
    }
}

pub struct FusionBlossomSyndromeExporter {
    /// logger
    pub solver_error_pattern_logger: Mutex<fusion_blossom::mwpm_solver::SolverErrorPatternLogger>,
    /// adaptor
    pub adaptor: Arc<FusionBlossomAdaptor>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FusionBlossomStabilizerFilter {
    None,
    StabZOnly,
}

impl FusionBlossomStabilizerFilter {
    pub fn ignore_node(&self, node: &SimulatorNode) -> bool {
        match self {
            Self::None => false,
            Self::StabZOnly => node.qubit_type != QubitType::StabZ,
        }
    }
}

pub mod fusion_blossom_syndrome_exporter_default_configs {
    pub fn only_stab_z() -> bool {
        false
    }
    pub fn max_half_weight() -> usize {
        5000
    }
}

impl FusionBlossomSyndromeExporter {
    pub fn new(fusion_decoder: &FusionDecoder, filename: String) -> Self {
        let solver_error_pattern_logger: SolverErrorPatternLogger = SolverErrorPatternLogger::new(
            &fusion_decoder.adaptor.initializer,
            &fusion_decoder.adaptor.positions,
            json!({ "filename": filename }),
        );
        Self {
            solver_error_pattern_logger: Mutex::new(solver_error_pattern_logger),
            adaptor: fusion_decoder.adaptor.clone(),
        }
    }
    pub fn add_syndrome(&self, sparse_measurement: &SparseMeasurement, sparse_detected_erasures: &SparseErasures) {
        use fusion_blossom::mwpm_solver::*;
        let syndrome_pattern = self
            .adaptor
            .generate_syndrome_pattern(sparse_measurement, sparse_detected_erasures);
        self.solver_error_pattern_logger
            .lock()
            .unwrap()
            .solve_visualizer(&syndrome_pattern, None);
    }
}

#[derive(Debug, Clone)]
pub struct FusionBlossomAdaptorExtender {
    pub base: FusionBlossomAdaptor,
    /// the number of noisy_measurements of the first
    pub noisy_measurements: usize,
    /// how many vertices in each measurement cycle
    pub vertex_num_cycle: usize,
    /// repeat region of `weighted_edges`
    pub edge_repeat_region: (usize, usize),
    /// repeat region of `virtual_vertices`, note that this repeat region may not align with `edge_repeat_region`
    pub virtual_repeat_region: (usize, usize),
    /// repeat the position
    pub position_repeat_region: (usize, usize),
    /// measurement cycle, `t` will be biased by repeat * measurement_delta_t
    pub measurement_delta_t: f64,
    /// measurement cycle
    pub measurement_cycle: usize,
}

impl FusionBlossomAdaptorExtender {
    pub fn new(first: FusionBlossomAdaptor, second: FusionBlossomAdaptor, noisy_measurements: usize) -> Self {
        assert!(
            second.initializer.weighted_edges.len() > first.initializer.weighted_edges.len(),
            "must differ"
        );
        let edge_num_differ = second.initializer.weighted_edges.len() - first.initializer.weighted_edges.len();
        let edge_repeat_start = first.initializer.weighted_edges.len() / 2 - edge_num_differ / 2;
        let edge_repeat_end = edge_repeat_start + edge_num_differ;
        let (u1, v1, w1) = first.initializer.weighted_edges[edge_repeat_start];
        let (u2, v2, w2) = first.initializer.weighted_edges[edge_repeat_end];
        assert_eq!(
            w1, w2,
            "should be the same edge at different cycles, consider increasing T to eliminate boundary effects"
        );
        assert_eq!(
            u2 - u1,
            v2 - v1,
            "should be the same edge at different cycles, consider increasing T to eliminate boundary effects"
        );
        let vertex_num_cycle = u2 - u1;
        let virtual_num_differ = second.initializer.virtual_vertices.len() - first.initializer.virtual_vertices.len();
        let virtual_repeat_start = first.initializer.virtual_vertices.len() / 2 - virtual_num_differ / 2;
        let virtual_repeat_end = virtual_repeat_start + virtual_num_differ;
        assert_eq!(
            vertex_num_cycle,
            first.initializer.virtual_vertices[virtual_repeat_end]
                - first.initializer.virtual_vertices[virtual_repeat_start]
        );
        let position_repeat_start = first.positions.len() / 2 - vertex_num_cycle / 2;
        let position_repeat_end = position_repeat_start + vertex_num_cycle;
        assert_eq!(
            vertex_num_cycle,
            second.positions.len() - first.positions.len(),
            "position number mismatch"
        );
        let measurement_delta_t = first.positions[position_repeat_end].t - first.positions[position_repeat_start].t;
        let measurement_cycle = first.vertex_to_position_mapping[position_repeat_end].t
            - first.vertex_to_position_mapping[position_repeat_start].t;
        let extender = Self {
            base: first,
            noisy_measurements,
            vertex_num_cycle,
            edge_repeat_region: (edge_repeat_start, edge_repeat_end),
            virtual_repeat_region: (virtual_repeat_start, virtual_repeat_end),
            position_repeat_region: (position_repeat_start, position_repeat_end),
            measurement_delta_t,
            measurement_cycle,
        };
        // use the second simulator to verify the correctness (partially)
        second.assert_eq(&extender.generate(noisy_measurements + 1, false)).unwrap();
        // return the verified extender
        extender
    }

    pub fn generate(&self, noisy_measurements: usize, skip_decoding: bool) -> FusionBlossomAdaptor {
        let base = &self.base;
        let vertex_num_cycle = self.vertex_num_cycle;
        let (edge_repeat_start, edge_repeat_end) = self.edge_repeat_region;
        let (virtual_repeat_start, virtual_repeat_end) = self.virtual_repeat_region;
        let (position_repeat_start, position_repeat_end) = self.position_repeat_region;
        let mut result = base.clone();
        assert!(noisy_measurements >= self.noisy_measurements);
        if noisy_measurements == self.noisy_measurements {
            return result;
        }
        let repeat: usize = noisy_measurements - self.noisy_measurements;
        result.initializer.vertex_num += vertex_num_cycle * repeat;
        result.initializer.weighted_edges.drain(edge_repeat_end..);
        result
            .initializer
            .weighted_edges
            .reserve_exact(repeat * (edge_repeat_end - edge_repeat_start));
        result.initializer.virtual_vertices.drain(virtual_repeat_end..);
        result
            .initializer
            .virtual_vertices
            .reserve_exact(repeat * (virtual_repeat_end - virtual_repeat_start));
        result.positions.drain(position_repeat_end..);
        result
            .positions
            .reserve_exact(repeat * (position_repeat_end - position_repeat_start));
        assert_eq!(base.vertex_to_position_mapping.len(), base.positions.len(), "should be equal");
        if skip_decoding {
            result.vertex_to_position_mapping.drain(..);
            result.edge_to_correction_mapping.drain(..);
        } else {
            result.edge_to_correction_mapping.drain(edge_repeat_end..);
            result
                .edge_to_correction_mapping
                .reserve_exact(repeat * (edge_repeat_end - edge_repeat_start));
            result.vertex_to_position_mapping.drain(position_repeat_end..);
            result
                .vertex_to_position_mapping
                .reserve_exact(repeat * (position_repeat_end - position_repeat_start));
        }
        for index in position_repeat_end..base.vertex_to_position_mapping.len() {
            result
                .position_to_vertex_mapping
                .remove(&base.vertex_to_position_mapping[index])
                .expect("has key");
        }
        for i in 1..repeat + 1 {
            for index in edge_repeat_start..edge_repeat_end {
                let (u, v, w) = base.initializer.weighted_edges[index];
                result
                    .initializer
                    .weighted_edges
                    .push((u + i * vertex_num_cycle, v + i * vertex_num_cycle, w));
                if !skip_decoding {
                    result
                        .edge_to_correction_mapping
                        .push(base.edge_to_correction_mapping[index].clone());
                }
            }
            for index in virtual_repeat_start..virtual_repeat_end {
                let v = base.initializer.virtual_vertices[index];
                result.initializer.virtual_vertices.push(v + i * vertex_num_cycle);
            }
            for index in position_repeat_start..position_repeat_end {
                let mut v = base.positions[index].clone();
                v.t += i as f64 * self.measurement_delta_t;
                result.positions.push(v);
                let mut p = base.vertex_to_position_mapping[index].clone();
                p.t += i * self.measurement_cycle;
                result
                    .position_to_vertex_mapping
                    .insert(p.clone(), i * (position_repeat_end - position_repeat_start) + index);
                if !skip_decoding {
                    result.vertex_to_position_mapping.push(p);
                }
            }
        }
        for index in edge_repeat_end..base.initializer.weighted_edges.len() {
            let (u, v, w) = base.initializer.weighted_edges[index];
            result
                .initializer
                .weighted_edges
                .push((u + repeat * vertex_num_cycle, v + repeat * vertex_num_cycle, w));
            if !skip_decoding {
                result
                    .edge_to_correction_mapping
                    .push(base.edge_to_correction_mapping[index].clone());
            }
        }
        for index in virtual_repeat_end..base.initializer.virtual_vertices.len() {
            let v = base.initializer.virtual_vertices[index];
            result.initializer.virtual_vertices.push(v + repeat * vertex_num_cycle);
        }
        for index in position_repeat_end..base.positions.len() {
            let mut v = base.positions[index].clone();
            v.t += repeat as f64 * self.measurement_delta_t;
            result.positions.push(v);
            let mut p = base.vertex_to_position_mapping[index].clone();
            p.t += repeat * self.measurement_cycle;
            result
                .position_to_vertex_mapping
                .insert(p.clone(), repeat * (position_repeat_end - position_repeat_start) + index);
            if !skip_decoding {
                result.vertex_to_position_mapping.push(p);
            }
        }
        if !skip_decoding {
            for correction in result.edge_to_correction_mapping.iter_mut() {
                let mut new_correction = SparseCorrection::new();
                for (position, error) in correction.iter() {
                    let mut position = position.clone();
                    position.t += repeat * self.measurement_cycle;
                    new_correction.add(position, *error);
                }
                *correction = new_correction;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::code_builder::*;
    use super::super::noise_model_builder::*;
    use super::*;

    #[test]
    fn fusion_decoder_debug_1() {
        // cargo test fusion_decoder_debug_1 -- --nocapture
        let d = 5;
        let noisy_measurements = 0; // perfect measurement
        let p = 0.;
        let pe = 0.1;
        // build simulator
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build noise model
        let mut noise_model = NoiseModel::new(&simulator);
        let noise_model_builder = NoiseModelBuilder::ErasureOnlyPhenomenological;
        noise_model_builder.apply(&mut simulator, &mut noise_model, &json!({}), p, 1., pe);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        // build decoder
        let decoder_config = json!({});
        let mut fusion_decoder = FusionDecoder::new(
            &Arc::new(simulator.clone()),
            Arc::clone(&noise_model),
            &decoder_config,
            1,
            false,
        );
        // load errors onto the simulator
        let sparse_error_pattern: SparseErrorPattern =
            serde_json::from_value(json!({"[0][1][5]":"Z","[0][2][6]":"Z","[0][4][4]":"X","[0][5][7]":"X","[0][9][7]":"Y"}))
                .unwrap();
        // let sparse_detected_erasures: SparseErasures = serde_json::from_value(json!({"erasures":["[0][1][3]","[0][1][5]","[0][2][6]","[0][4][4]","[0][5][7]","[0][6][6]","[0][9][7]"]})).unwrap();
        simulator
            .load_sparse_error_pattern(&sparse_error_pattern, &noise_model)
            .expect("success");
        // simulator.load_sparse_detected_erasures(&sparse_detected_erasures).expect("success");
        simulator.propagate_errors();
        let sparse_measurement = simulator.generate_sparse_measurement();
        println!("sparse_measurement: {:?}", sparse_measurement);
        let sparse_detected_erasures = simulator.generate_sparse_detected_erasures();
        let (correction, _runtime_statistics) =
            fusion_decoder.decode_with_erasure(&sparse_measurement, &sparse_detected_erasures);
        println!("correction: {:?}", correction);
        code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        let (logical_i, logical_j) = simulator.validate_correction(&correction);
        assert!(!logical_i && !logical_j);
    }

    #[test]
    fn fusion_decoder_debug_2() {
        // cargo test fusion_decoder_debug_2 -- --nocapture
        let d = 7;
        let noisy_measurements = 0; // perfect measurement
        let p = 0.1;
        // build simulator
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build noise model
        let mut noise_model = NoiseModel::new(&simulator);
        simulator.set_error_rates(&mut noise_model, p / 3., p / 3., p / 3., 0.);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        // build decoder
        let decoder_config = json!({});
        let mut fusion_decoder = FusionDecoder::new(
            &Arc::new(simulator.clone()),
            Arc::clone(&noise_model),
            &decoder_config,
            1,
            false,
        );
        // load errors onto the simulator
        let sparse_error_pattern: SparseErrorPattern = serde_json::from_value(json!({"[0][1][9]":"X","[0][4][8]":"Y","[0][5][9]":"Z","[0][6][10]":"Z","[0][7][11]":"Z","[0][8][6]":"X","[0][8][12]":"Z","[0][9][5]":"Y","[0][12][2]":"Y","[0][12][6]":"X"})).unwrap();
        simulator
            .load_sparse_error_pattern(&sparse_error_pattern, &noise_model)
            .expect("success");
        // simulator.load_sparse_detected_erasures(&sparse_detected_erasures).expect("success");
        simulator.propagate_errors();
        let sparse_measurement = simulator.generate_sparse_measurement();
        println!("sparse_measurement: {:?}", sparse_measurement);
        let sparse_detected_erasures = simulator.generate_sparse_detected_erasures();
        let (correction, _runtime_statistics) =
            fusion_decoder.decode_with_erasure(&sparse_measurement, &sparse_detected_erasures);
        println!("correction: {:?}", correction);
        code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        let (logical_i, logical_j) = simulator.validate_correction(&correction);
        assert!(!logical_i && !logical_j);
    }

    #[test]
    fn adaptor_extender() {
        // cargo test adaptor_extender -- --nocapture
        let di = 3;
        let dj = 3;
        let p = 0.001;
        let build_adaptor = |noisy_measurements: usize| -> FusionBlossomAdaptor {
            let config = json!({
                "only_stab_z": true,
                "use_combined_probability": false,
                "max_half_weight": 500,
            });
            let mut simulator = Simulator::new(CodeType::RotatedPlanarCode, CodeSize::new(noisy_measurements, di, dj));
            let mut noise_model = NoiseModel::new(&simulator);
            NoiseModelBuilder::StimNoiseModel.apply(&mut simulator, &mut noise_model, &json!({}), p, 0.5, 0.);
            code_builder_sanity_check(&simulator).unwrap();
            noise_model_sanity_check(&simulator, &noise_model).unwrap();
            let fusion_decoder = FusionDecoder::new(&simulator, Arc::new(noise_model), &config, 1, true);
            Arc::try_unwrap(fusion_decoder.adaptor).unwrap()
        };
        let noisy_measurements = 4;
        let first = build_adaptor(noisy_measurements);
        let second = build_adaptor(noisy_measurements + 1);
        let extender = FusionBlossomAdaptorExtender::new(first, second, noisy_measurements);
        println!("extender built successfully");
        // test a larger instance
        let test_noisy_measurement = 7;
        let generated = extender.generate(test_noisy_measurement, false);
        let ground_truth = build_adaptor(test_noisy_measurement);
        generated.assert_eq(&ground_truth).unwrap();
    }
}
