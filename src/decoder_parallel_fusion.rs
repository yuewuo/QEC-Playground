//! minimum-weight perfect matching decoder
//!

use super::model_graph::*;
use super::noise_model::*;
use super::serde_json;
use super::simulator::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use crate::decoder_fusion::{FusionBlossomAdaptor, FusionDecoderConfig};
use super::decoder_mwpm::*;
use super::derivative::*;
use crate::fusion_blossom::mwpm_solver::*;
use crate::fusion_blossom::util::*;

/// MWPM decoder based on fusion blossom algorithm, initialized and cloned for multiple threads
#[derive(Derivative, Serialize)]
#[derivative(Debug)]
pub struct ParallelFusionDecoder {
    /// shared data helps interface with the fusion blossom algorithm
    pub adaptor: Arc<FusionBlossomAdaptor>,
    /// fusion blossom algorithm: a fast MWPM solver for quantum error correction
    #[serde(skip)]
    #[derivative(Debug = "ignore")]
    pub fusion_solver: fusion_blossom::mwpm_solver::SolverParallel,
    /// save configuration for later usage
    pub config: ParallelFusionDecoderConfig,
}

impl Clone for ParallelFusionDecoder {
    fn clone(&self) -> Self {
        // construct a new solver instance
        let partition_info = self.config.partition_config.clone().unwrap_or(PartitionConfig::new(self.adaptor.vertex_to_position_mapping.len())).info();
        let fusion_solver = if self.config.skip_decoding {
            fusion_blossom::mwpm_solver::SolverParallel::new(&SolverInitializer {
                vertex_num: 0,
                weighted_edges: vec![],
                virtual_vertices: vec![],
            }, &partition_info, self.config.primal_dual_config.clone())
        } else {
            fusion_blossom::mwpm_solver::SolverParallel::new(&self.adaptor.initializer, &partition_info, self.config.primal_dual_config.clone())
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
pub struct ParallelFusionDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")] // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")] // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    /// only export Z stabilizers
    #[serde(default = "parallel_fusion_default_configs::only_stab_z")]
    pub only_stab_z: bool,
    #[serde(alias = "mhw")] // abbreviation
    #[serde(default = "parallel_fusion_default_configs::max_half_weight")]
    pub max_half_weight: usize,
    #[serde(default = "parallel_fusion_default_configs::skip_decoding")]
    pub skip_decoding: bool,
    #[serde(default = "parallel_fusion_default_configs::log_matchings")]
    pub log_matchings: bool,
    #[serde(default = "parallel_fusion_default_configs::primal_dual_config")]
    pub primal_dual_config: serde_json::Value,
    #[serde(default = "parallel_fusion_default_configs::partition_config")]
    pub partition_config: Option<PartitionConfig>,
}

pub mod parallel_fusion_default_configs {
    use super::*;

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
    pub fn primal_dual_config() -> serde_json::Value { json!({}) }
    pub fn partition_config() -> Option<PartitionConfig> { None }
}

impl ParallelFusionDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(
        simulator: &Simulator,
        noise_model: Arc<NoiseModel>,
        decoder_configuration: &serde_json::Value,
        parallel: usize,
        use_brief_edge: bool,
    ) -> Self {
        // read attribute of decoder configuration
        let config: ParallelFusionDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        let mut simulator = simulator.clone();
        // // build erasure graph
        // let mut erasure_graph = ErasureGraph::new(&simulator);
        // erasure_graph.build(&mut simulator, Arc::clone(&noise_model), parallel);
        // let erasure_graph = Arc::new(erasure_graph);
        // build solver
        let adaptor = FusionBlossomAdaptor::new(&FusionDecoderConfig {
            weight_function: config.weight_function.clone(),
            use_combined_probability: config.use_combined_probability,
            only_stab_z: config.only_stab_z,
            max_half_weight: config.max_half_weight,
            skip_decoding: config.skip_decoding,
            log_matchings: config.log_matchings,
            max_tree_size: usize::MAX,
        }, &mut simulator, noise_model, parallel, use_brief_edge);
        let partition_info = config.partition_config.clone().unwrap_or(PartitionConfig::new(adaptor.vertex_to_position_mapping.len())).info();
        let fusion_solver = fusion_blossom::mwpm_solver::SolverParallel::new(&adaptor.initializer, &partition_info, config.primal_dual_config.clone());
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



#[cfg(test)]
mod tests {
    use super::super::code_builder::*;
    use super::super::noise_model_builder::*;
    use super::*;

    #[test]
    fn parallel_fusion_decoder_debug_1() {
        // cargo test parallel_fusion_decoder_debug_1 -- --nocapture
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
        let decoder_config = json!({
            "partition_config": {
                "vertex_num": 60,
                // "partitions": [(0, 60)],
                // "fusions": [],
                "partitions": [(0, 20), (41, 60)],
                "fusions": [(0, 1)],
            }
        });
        let mut parallel_fusion_decoder = ParallelFusionDecoder::new(
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
            parallel_fusion_decoder.decode_with_erasure(&sparse_measurement, &sparse_detected_erasures);
        println!("correction: {:?}", correction);
        code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        let (logical_i, logical_j) = simulator.validate_correction(&correction);
        assert!(!logical_i && !logical_j);
    }
}
