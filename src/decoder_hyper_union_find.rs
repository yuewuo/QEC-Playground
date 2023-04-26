//! union-find decoder on hypergraph

use crate::model_hypergraph::*;
use std::sync::{Arc};
use serde::{Serialize, Deserialize};
use super::decoder_mwpm::*;
use super::model_graph::*;
use super::simulator::*;
use super::noise_model::*;
use std::time::Instant;
use crate::mwps::mwps_solver::*;
use crate::mwps::util::*;


pub struct HyperUnionFindDecoder {
    /// model hypergraph
    pub model_hypergraph: Arc<ModelHypergraph>,
    /// save configuration for later usage
    pub config: HyperUnionFindDecoderConfig,
    /// (approximate) minimum-weight parity subgraph solver
    pub solver: SolverUnionFind,
    /// the initializer of the solver, used for customized clone
    pub initializer: Arc<SolverInitializer>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HyperUnionFindDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")]  // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    /// the maximum integer weight after scaling
    #[serde(alias = "mhw")]  // abbreviation
    #[serde(default = "hyper_union_find_default_configs::max_weight")]
    pub max_weight: usize,
}

pub mod hyper_union_find_default_configs {
    pub fn max_weight() -> usize { 1000000 }
}

impl Clone for HyperUnionFindDecoder {
    fn clone(&self) -> Self {
        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver: SolverUnionFind::new(&self.initializer),
            initializer: self.initializer.clone(),
        }
    }
}

impl HyperUnionFindDecoder {

    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, noise_model: Arc<NoiseModel>, decoder_configuration: &serde_json::Value, parallel: usize, use_brief_edge: bool) -> Self {
        // read attribute of decoder configuration
        let config: HyperUnionFindDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_hypergraph = ModelHypergraph::new(&simulator);
        model_hypergraph.build(&mut simulator, Arc::clone(&noise_model), &config.weight_function, parallel, config.use_combined_probability, use_brief_edge);
        let model_hypergraph = Arc::new(model_hypergraph);
        let (vertex_num, weighted_edges) = model_hypergraph.generate_mwps_hypergraph(config.max_weight);
        let initializer = Arc::new(SolverInitializer::new(vertex_num, weighted_edges));
        let solver = SolverUnionFind::new(&initializer);
        Self {
            model_hypergraph,
            config,
            solver,
            initializer,
        }
    }

    /// decode given measurement results
    #[allow(dead_code)]
    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        self.decode_with_erasure(sparse_measurement, &SparseErasures::new())
    }

    /// decode given measurement results and detected erasures
    pub fn decode_with_erasure(&mut self, sparse_measurement: &SparseMeasurement, sparse_detected_erasures: &SparseErasures) -> (SparseCorrection, serde_json::Value) {
        if sparse_detected_erasures.len() > 0 {
            unimplemented!()
        }
        // run decode
        let begin = Instant::now();
        let defect_vertices: Vec<_> = sparse_measurement.iter().map(|position| {
            *self.model_hypergraph.vertex_indices.get(position).expect("measurement cannot happen at impossible position")
        }).collect();
        let syndrome_pattern = SyndromePattern::new(defect_vertices, vec![]);
        self.solver.solve(&syndrome_pattern);
        let subgraph = self.solver.subgraph();
        self.solver.clear();
        let time_decode = begin.elapsed().as_secs_f64();
        // build correction
        let begin = Instant::now();
        let mut correction = SparseCorrection::new();
        for &edge_index in subgraph.iter() {
            correction.extend(&self.model_hypergraph.weighted_edges[edge_index].1.hyperedge.correction);
        }
        let time_build_correction = begin.elapsed().as_secs_f64();
        (correction, json!({
            "time_decode": time_decode,
            "time_build_correction": time_build_correction,
        }))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::code_builder::*;
    use super::super::types::ErrorType::*;

    #[test]
    fn hyper_union_find_decoder_code_capacity() {  // cargo test hyper_union_find_decoder_code_capacity -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.001;
        // build simulator
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build noise model
        let mut noise_model = NoiseModel::new(&simulator);
        simulator.set_error_rates(&mut noise_model, p, p, p, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        // build decoder
        let enable_all = true;
        let mut hyper_union_find_decoder = HyperUnionFindDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        if true || enable_all {  // debug 5
            simulator.clear_all_errors();
            // {"[0][4][6]":"Z","[0][5][9]":"Z","[0][7][1]":"Z","[0][9][1]":"Z"}
            simulator.set_error_check(&noise_model, &pos!(0, 4, 6), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 5, 9), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 7, 1), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 9, 1), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 4, should fail
            simulator.clear_all_errors();
            // {"[0][1][5]":"Z","[0][5][3]":"Z","[0][5][7]":"Z","[0][7][7]":"Z"}
            simulator.set_error_check(&noise_model, &pos!(0, 1, 5), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 5, 3), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 5, 7), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 7, 7), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        }
        if false || enable_all {  // debug 3
            simulator.clear_all_errors();
            // {"[0][6][6]":"Z","[0][8][2]":"Z","[0][8][4]":"Z"}
            simulator.set_error_check(&noise_model, &pos!(0, 6, 6), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 8, 2), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 8, 4), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 2
            simulator.clear_all_errors();
            // {"[0][3][9]":"Z","[0][8][8]":"Z"}
            simulator.set_error_check(&noise_model, &pos!(0, 3, 9), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 8, 8), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 1
            simulator.clear_all_errors();
            simulator.set_error_check(&noise_model, &pos!(0, 6, 4), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 6, 6), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 5, 7), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
    }

}
