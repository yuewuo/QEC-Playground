//! minimum-weight perfect matching decoder
//! 

use serde::{Serialize, Deserialize};
use super::simulator::*;
use super::error_model::*;
use super::model_graph::*;
use super::serde_json;
use std::sync::{Arc};
use std::time::Instant;
use super::erasure_graph::*;
use super::decoder_mwpm::*;
use super::fusion_blossom;
use super::fusion_blossom::mwpm_solver::PrimalDualSolver;
use std::collections::BTreeMap;
use super::derivative::*;


#[derive(Debug, Clone, Serialize)]
pub struct FusionDecoderSharedData {
    // TODO: optimize for the placement of positions to better leverage cache system of the decoder: group near ones together
    pub vertex_num: usize,
    pub real_vertex_num: usize,
    pub weighted_edges: Vec<(usize, usize, fusion_blossom::util::Weight)>,
    pub virtual_vertices: Vec<usize>,
    pub position_to_vertex_mapping: BTreeMap<Position, usize>,  // only be able to match real vertices
    pub vertex_to_position_mapping: Vec<Position>,
}

/// MWPM decoder based on fusion blossom algorithm, initialized and cloned for multiple threads
#[derive(Derivative, Serialize)]
#[derivative(Debug)]
pub struct FusionDecoder {
    /// model graph is immutably shared
    pub model_graph: Arc<ModelGraph>,
    /// erasure graph is immutably shared
    pub erasure_graph: Arc<ErasureGraph>,
    /// shared data helps interface with the fusion blossom algorithm
    pub shared_data: Arc<FusionDecoderSharedData>,
    /// fusion blossom algorithm: a fast MWPM solver for quantum error correction
    #[serde(skip)]
    #[derivative(Debug="ignore")]
    pub fusion_solver: fusion_blossom::mwpm_solver::SolverSerial,
    /// save configuration for later usage
    pub config: FusionDecoderConfig,
    /// an immutably shared simulator that is used to change model graph on the fly for correcting erasure errors
    pub simulator: Arc<Simulator>,
}

impl Clone for FusionDecoder {
    fn clone(&self) -> Self {
        let initializer = fusion_blossom::util::SolverInitializer::new(self.shared_data.vertex_num
            , self.shared_data.weighted_edges.clone(), self.shared_data.virtual_vertices.clone());
        let fusion_solver = fusion_blossom::mwpm_solver::SolverSerial::new(&initializer);
        Self {
            model_graph: self.model_graph.clone(),
            erasure_graph: self.erasure_graph.clone(),
            shared_data: self.shared_data.clone(),
            fusion_solver: fusion_solver,
            config: self.config.clone(),
            simulator: self.simulator.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FusionDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. error model
    #[serde(alias = "ucp")]  // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
}

pub mod fusion_default_configs {
    // use super::*;
}

impl FusionDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: Arc<ErrorModel>, decoder_configuration: &serde_json::Value, parallel: usize, use_brief_edge: bool) -> Self {
        // read attribute of decoder configuration
        let config: FusionDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_graph = ModelGraph::new(&simulator);
        model_graph.build(&mut simulator, Arc::clone(&error_model), &config.weight_function, parallel, config.use_combined_probability, use_brief_edge);
        let model_graph = Arc::new(model_graph);
        // build erasure graph
        let mut erasure_graph = ErasureGraph::new(&simulator);
        erasure_graph.build(&mut simulator, Arc::clone(&error_model), parallel);
        let erasure_graph = Arc::new(erasure_graph);
        // build fusion decoder shared information
        let mut shared_data = FusionDecoderSharedData {
            vertex_num: 0,
            real_vertex_num: 0,
            weighted_edges: vec![],
            virtual_vertices: vec![],
            position_to_vertex_mapping: BTreeMap::new(),
            vertex_to_position_mapping: vec![],
        };
        simulator_iter!(simulator, position, node, {  // first insert nodes and build mapping
            if position.t != 0 && node.gate_type.is_measurement() && simulator.is_node_real(position) {
                shared_data.vertex_to_position_mapping.push(position.clone());
                shared_data.position_to_vertex_mapping.insert(position.clone(), shared_data.vertex_num);
                shared_data.vertex_num += 1;
                shared_data.real_vertex_num += 1;
            }
        });
        let mut weighted_edges_unscaled = Vec::<(usize, usize, f64)>::new();
        simulator_iter!(simulator, position, node, {  // then add edges and also virtual nodes
            if position.t != 0 && node.gate_type.is_measurement() && simulator.is_node_real(position) {
                let model_graph_node = model_graph.get_node_unwrap(position);
                let vertex_idx = shared_data.position_to_vertex_mapping[&position];
                if let Some(model_graph_boundary) = &model_graph_node.boundary {
                    let virtual_idx = shared_data.vertex_num;
                    shared_data.vertex_num += 1;
                    shared_data.virtual_vertices.push(virtual_idx);
                    weighted_edges_unscaled.push((vertex_idx, virtual_idx, model_graph_boundary.weight));
                }
                for (peer_position, model_graph_edge) in model_graph_node.edges.iter() {
                    let peer_idx = shared_data.position_to_vertex_mapping[peer_position];
                    if vertex_idx < peer_idx {  // avoid duplicate edges
                        weighted_edges_unscaled.push((vertex_idx, peer_idx, model_graph_edge.weight));
                    }
                }
            }
        });
        shared_data.weighted_edges = {  // re-weight edges and parse to integer
            let mut maximum_weight = 0.;
            for (_, _, weight) in weighted_edges_unscaled.iter() {
                if weight > &maximum_weight {
                    maximum_weight = *weight;
                }
            }
            let scale: f64 = (fusion_blossom::util::Weight::MAX as f64) / 10. / ((shared_data.vertex_num + 1) as f64) / maximum_weight;
            weighted_edges_unscaled.iter().map(|(a, b, weight)| (*a, *b, 2 * (weight * scale).ceil() as fusion_blossom::util::Weight)).collect()
        };
        let initializer = fusion_blossom::util::SolverInitializer::new(shared_data.vertex_num, shared_data.weighted_edges.clone(), shared_data.virtual_vertices.clone());
        let fusion_solver = fusion_blossom::mwpm_solver::SolverSerial::new(&initializer);
        // println!("position_to_vertex_mapping: {:?}", shared_data.position_to_vertex_mapping);
        // println!("weighted_edges: {:?}", shared_data.weighted_edges);
        // println!("virtual_vertices: {:?}", shared_data.virtual_vertices);
        Self {
            model_graph: model_graph,
            erasure_graph: erasure_graph,
            shared_data: Arc::new(shared_data),
            fusion_solver: fusion_solver,
            config: config,
            simulator: Arc::new(simulator),
        }
    }

    /// decode given measurement results
    #[allow(dead_code)]
    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        self.decode_with_erasure(sparse_measurement, &SparseDetectedErasures::new())
    }

    /// decode given measurement results and detected erasures
    pub fn decode_with_erasure(&mut self, sparse_measurement: &SparseMeasurement, sparse_detected_erasures: &SparseDetectedErasures) -> (SparseCorrection, serde_json::Value) {
        assert!(sparse_detected_erasures.len() == 0, "fusion decoder doesn't support erasure error yet: we'll do it in the next version to support 0-weight edges and dynamic setting");
        let mut correction = SparseCorrection::new();
        // list nontrivial measurements to be matched
        let to_be_matched = sparse_measurement.to_vec();
        let mut time_fusion = 0.;
        let mut time_build_correction = 0.;
        if to_be_matched.len() > 0 {
            // run the Blossom algorithm
            let begin = Instant::now();
            let syndrome_vertices: Vec<usize> = to_be_matched.iter().map(|position| self.shared_data.position_to_vertex_mapping[position]).collect();
            let syndrome_pattern = fusion_blossom::util::SyndromePattern::new_vertices(syndrome_vertices);
            self.fusion_solver.solve(&syndrome_pattern);
            let fusion_subgraph = self.fusion_solver.subgraph();
            self.fusion_solver.clear();
            time_fusion += begin.elapsed().as_secs_f64();
            // build correction based on the matching
            let begin = Instant::now();
            for edge_idx in fusion_subgraph.iter() {
                let (a, b, _) = self.shared_data.weighted_edges[*edge_idx];
                if b < self.shared_data.real_vertex_num {
                    let pos_a = &self.shared_data.vertex_to_position_mapping[a];
                    let pos_b = &self.shared_data.vertex_to_position_mapping[b];
                    let matching_correction = self.model_graph.build_correction_matching(pos_a, pos_b);
                    correction.extend(&matching_correction);
                } else {
                    let pos_a = &self.shared_data.vertex_to_position_mapping[a];
                    let boundary_correction = self.model_graph.build_correction_boundary(pos_a);
                    correction.extend(&boundary_correction);
                }
            }
            time_build_correction += begin.elapsed().as_secs_f64();
        }
        (correction, json!({
            "to_be_matched": to_be_matched.len(),
            "time_fusion": time_fusion,
            "time_build_correction": time_build_correction,
        }))
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::code_builder::*;
    use super::super::error_model_builder::*;

    #[test]
    fn fusion_decoder_debug_1() {  // cargo test fusion_decoder_debug_1 -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.;
        let pe = 0.1;
        // build simulator
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let error_model_builder = ErrorModelBuilder::ErasureOnlyPhenomenological;
        error_model_builder.apply(&mut simulator, &mut error_model, &json!({}), p, 1., pe);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({});
        let mut fusion_decoder = FusionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        // load errors onto the simulator
        let sparse_error_pattern: SparseErrorPattern = serde_json::from_value(json!({"[0][1][5]":"Z","[0][2][6]":"Z","[0][4][4]":"X","[0][5][7]":"X","[0][9][7]":"Y"})).unwrap();
        // let sparse_detected_erasures: SparseDetectedErasures = serde_json::from_value(json!({"erasures":["[0][1][3]","[0][1][5]","[0][2][6]","[0][4][4]","[0][5][7]","[0][6][6]","[0][9][7]"]})).unwrap();
        simulator.load_sparse_error_pattern(&sparse_error_pattern).expect("success");
        // simulator.load_sparse_detected_erasures(&sparse_detected_erasures).expect("success");
        simulator.propagate_errors();
        let sparse_measurement = simulator.generate_sparse_measurement();
        println!("sparse_measurement: {:?}", sparse_measurement);
        let sparse_detected_erasures = simulator.generate_sparse_detected_erasures();
        let (correction, _runtime_statistics) = fusion_decoder.decode_with_erasure(&sparse_measurement, &sparse_detected_erasures);
        println!("correction: {:?}", correction);
        code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        let (logical_i, logical_j) = simulator.validate_correction(&correction);
        assert!(!logical_i && !logical_j);
    }

    #[test]
    fn fusion_decoder_debug_2() {  // cargo test fusion_decoder_debug_2 -- --nocapture
        let d = 7;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.1;
        // build simulator
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        simulator.set_error_rates(&mut error_model, p/3., p/3., p/3., 0.);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({});
        let mut fusion_decoder = FusionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        // load errors onto the simulator
        let sparse_error_pattern: SparseErrorPattern = serde_json::from_value(json!({"[0][1][2]":"Y","[0][1][9]":"X","[0][2][1]":"Z","[0][4][8]":"Y","[0][5][2]":"Z","[0][5][9]":"Z","[0][6][10]":"Z","[0][7][11]":"Z","[0][8][6]":"X","[0][8][11]":"X","[0][8][12]":"Z","[0][9][5]":"Y","[0][12][2]":"Y","[0][12][6]":"X","[0][12][13]":"X","[0][13][2]":"Z","[0][13][6]":"Y"})).unwrap();
        simulator.load_sparse_error_pattern(&sparse_error_pattern).expect("success");
        // simulator.load_sparse_detected_erasures(&sparse_detected_erasures).expect("success");
        simulator.propagate_errors();
        let sparse_measurement = simulator.generate_sparse_measurement();
        println!("sparse_measurement: {:?}", sparse_measurement);
        let sparse_detected_erasures = simulator.generate_sparse_detected_erasures();
        let (correction, _runtime_statistics) = fusion_decoder.decode_with_erasure(&sparse_measurement, &sparse_detected_erasures);
        println!("correction: {:?}", correction);
        code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        let (logical_i, logical_j) = simulator.validate_correction(&correction);
        assert!(!logical_i && !logical_j);
        
    }

}
