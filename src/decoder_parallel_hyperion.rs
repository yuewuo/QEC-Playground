//! Parallel Hyperion decoder: BP+MWPF with time-based graph partitioning

use super::decoder_mwpm::*;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use crate::decoder_parallel_hyper_union_find::graph_time_partition;
use crate::model_hypergraph::*;
use crate::mwpf::{bp::bp::*, mwpf_solver::*, util::*};
use num_traits::{FromPrimitive, One};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

pub struct ParallelHyperionDecoder {
    /// model hypergraph
    pub model_hypergraph: Arc<ModelHypergraph>,
    /// save configuration for later usage
    pub config: ParallelHyperionDecoderConfig,
    /// (approximate) minimum-weight parity factor solver (parallel version)
    pub solver: SolverParallelJointSingleHair,
    /// the initializer of the solver, used for customized clone
    pub initializer: Arc<SolverInitializer>,
    /// bp decoder if in use
    pub bp_decoder: Option<BpDecoder>,
    /// initial log ratios for bp decoder if in use
    pub initial_log_ratios: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParallelHyperionDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")] // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")] // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    #[serde(default = "parallel_hyperion_default_configs::uniform_weights")]
    pub uniform_weights: bool,
    #[serde(default = "parallel_hyperion_default_configs::default_hyperion_config")]
    pub hyperion_config: serde_json::Value,
    #[serde(default = "parallel_hyperion_default_configs::substitute_with_simple_graph")]
    pub substitute_with_simple_graph: bool,
    #[serde(default = "parallel_hyperion_default_configs::use_bp")]
    pub use_bp: bool,
    #[serde(default = "parallel_hyperion_default_configs::bp_iteration")]
    pub bp_iteration: usize,
    #[serde(default = "parallel_hyperion_default_configs::bp_application_ratio")]
    pub bp_application_ratio: f64,
    #[serde(default = "parallel_hyperion_default_configs::split_num")]
    pub split_num: usize,
    #[serde(default = "parallel_hyperion_default_configs::partition_config")]
    pub partition_config: Option<PartitionConfig>,
}

pub mod parallel_hyperion_default_configs {
    use mwpf::util::PartitionConfig;

    pub fn uniform_weights() -> bool {
        false
    }
    pub fn default_hyperion_config() -> serde_json::Value {
        json!({})
    }
    pub fn substitute_with_simple_graph() -> bool {
        false
    }
    pub fn use_bp() -> bool {
        false
    }
    pub fn bp_iteration() -> usize {
        1
    }
    pub fn bp_application_ratio() -> f64 {
        0.1
    }
    pub fn split_num() -> usize {
        2
    }
    pub fn partition_config() -> Option<PartitionConfig> {
        None
    }
}

impl Clone for ParallelHyperionDecoder {
    fn clone(&self) -> Self {
        let partition_config = graph_time_partition(&self.initializer, &self.model_hypergraph.vertex_positions, self.config.split_num);
        let partition_info = partition_config.info();
        let solver = SolverParallelJointSingleHair::new(&self.initializer, &partition_info, self.config.hyperion_config.clone());

        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver,
            initializer: self.initializer.clone(),
            bp_decoder: self.bp_decoder.clone(),
            initial_log_ratios: self.initial_log_ratios.clone(),
        }
    }
}

impl ParallelHyperionDecoder {
    /// create a new parallel Hyperion decoder with decoder configuration
    pub fn new(
        simulator: &Simulator,
        noise_model: Arc<NoiseModel>,
        decoder_configuration: &serde_json::Value,
        parallel: usize,
        use_brief_edge: bool,
    ) -> Self {
        // read attribute of decoder configuration
        let mut config: ParallelHyperionDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_hypergraph = ModelHypergraph::new(&simulator);
        if config.substitute_with_simple_graph {
            let mut model_graph = ModelGraph::new(&simulator);
            model_graph.build(
                &mut simulator,
                noise_model,
                &config.weight_function,
                parallel,
                config.use_combined_probability,
                use_brief_edge,
            );
            model_hypergraph.load_from_model_graph(&model_graph);
        } else {
            model_hypergraph.build(
                &mut simulator,
                Arc::clone(&noise_model),
                &config.weight_function,
                parallel,
                config.use_combined_probability,
                use_brief_edge,
            );
        }
        let model_hypergraph = Arc::new(model_hypergraph);
        let (vertex_num, weighted_edges) = model_hypergraph.generate_mwpf_hypergraph();

        let check_size = weighted_edges.len();

        let mut initializer = SolverInitializer::new(vertex_num, weighted_edges);
        if config.uniform_weights {
            initializer.uniform_weights(Weight::one());
        }
        let initializer = Arc::new(initializer);

        // set up partition config
        config.partition_config = Some(graph_time_partition(&initializer, &model_hypergraph.vertex_positions, config.split_num));
        let partition_info = config.partition_config.clone().unwrap().info();
        let solver = SolverParallelJointSingleHair::new(&initializer, &partition_info, config.hyperion_config.clone());

        // set up BP decoder
        let mut bp_decoder_option = None;
        let mut initial_log_ratios_option = None;

        if config.use_bp {
            let mut pcm = BpSparse::new(vertex_num, check_size, 0);
            let mut initial_log_ratios = Vec::with_capacity(check_size);
            let mut channel_probabilities = Vec::with_capacity(check_size);

            for (col_index, (defect_vertices, hyperedge_group)) in model_hypergraph.weighted_edges.iter().enumerate() {
                channel_probabilities.push(hyperedge_group.hyperedge.probability);
                for vertex_position in defect_vertices.0.iter() {
                    let row_index = model_hypergraph.vertex_indices.get(vertex_position).unwrap();
                    pcm.insert_entry(*row_index, col_index);
                }
                initial_log_ratios.push(hyperedge_group.hyperedge.weight as f64);
            }

            let bp_decoder = BpDecoder::new_3(pcm, channel_probabilities, config.bp_iteration).unwrap();

            bp_decoder_option = Some(bp_decoder);
            initial_log_ratios_option = Some(initial_log_ratios);
        }

        Self {
            model_hypergraph,
            config,
            solver,
            initializer,
            bp_decoder: bp_decoder_option,
            initial_log_ratios: initial_log_ratios_option,
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
        if !sparse_detected_erasures.is_empty() {
            unimplemented!()
        }
        // run decode
        let begin = Instant::now();

        let mut syndrome_array = vec![];
        if self.config.use_bp {
            syndrome_array = vec![0; self.model_hypergraph.vertex_indices.len()];
        }

        let defect_vertices: Vec<_> = sparse_measurement
            .iter()
            .map(|position| {
                let temp = *self
                    .model_hypergraph
                    .vertex_indices
                    .get(position)
                    .expect("measurement cannot happen at impossible position");
                if self.config.use_bp {
                    syndrome_array[temp] = 1;
                }
                (0, temp)
            })
            .collect();

        let syndrome_pattern = SyndromePattern::new_vertices(defect_vertices);

        // update partition config with defect vertices and recreate solver
        if let Some(ref mut temp_partition_config) = self.config.partition_config {
            temp_partition_config.defect_vertices = FastIterSet::from_iter(syndrome_pattern.defect_vertices.iter().map(|v| v.1));
        }
        let partition_info = self.config.partition_config.clone().unwrap().info();
        self.solver = SolverParallelJointSingleHair::new(&self.initializer, &partition_info, self.config.hyperion_config.clone());

        let decoder_begin = Instant::now();

        // run BP weight update if enabled
        if self.config.use_bp {
            self.bp_decoder
                .as_mut()
                .unwrap()
                .set_log_domain_bp(&self.initial_log_ratios.as_ref().unwrap());

            self.bp_decoder.as_mut().unwrap().decode(&syndrome_array);
            let llrs = self
                .bp_decoder
                .as_ref()
                .unwrap()
                .log_prob_ratios
                .iter()
                .map(|v| Weight::from_f64(*v).unwrap())
                .collect();

            self.solver.update_weights(llrs, self.config.bp_application_ratio.into());
        }

        let time_decode_bp = decoder_begin.elapsed().as_secs_f64();

        self.solver.solve(syndrome_pattern);
        let subgraph = self.solver.subgraph();
        self.solver.clear();

        let time_decode_mwpf = decoder_begin.elapsed().as_secs_f64() - time_decode_bp;
        let time_decode = begin.elapsed().as_secs_f64();

        // build correction
        let begin = Instant::now();
        let mut correction = SparseCorrection::new();
        for &edge_index in subgraph.iter() {
            correction.extend(&self.model_hypergraph.weighted_edges[edge_index].1.hyperedge.correction);
        }
        let time_build_correction = begin.elapsed().as_secs_f64();
        (
            correction,
            json!({
                "time_decode": time_decode,
                "time_build_correction": time_build_correction,
                "time_decode_bp": time_decode_bp,
                "time_decode_mwpf": time_decode_mwpf,
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::code_builder::*;
    use super::super::types::ErrorType::*;
    use super::*;

    /// Test with RotatedPlanarCode, code capacity (noisy_measurements = 0)
    #[test]
    fn parallel_hyperion_decoder_rotated_planar_code() {
        // cargo test parallel_hyperion_decoder_rotated_planar_code -- --nocapture
        let d = 5;
        let noisy_measurements = 0;
        let p = 0.001;
        let mut simulator = Simulator::new(CodeType::RotatedPlanarCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        let mut noise_model = NoiseModel::new(&simulator);
        simulator.set_error_rates(&mut noise_model, p, p, p, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        let mut decoder =
            ParallelHyperionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        {
            let mut logical_error_count = 0;
            let total_rounds = 6;
            for round in 0..total_rounds {
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, _runtime_statistics) = decoder.decode(&sparse_measurement);
                code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
                let (logical_i, logical_j) = simulator.validate_correction(&correction);
                if logical_i || logical_j {
                    logical_error_count += 1;
                }
                if round < 5 {
                    println!("round {}: logical_i={}, logical_j={}", round, logical_i, logical_j);
                }
            }
            println!("logical error rate: {}/{}", logical_error_count, total_rounds);
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }

    /// Test with RotatedTailoredCode under biased noise (code capacity, no measurement noise)
    #[test]
    fn parallel_hyperion_decoder_tailored_code_capacity() {
        // cargo test parallel_hyperion_decoder_tailored_code_capacity -- --nocapture
        let d = 5;
        let noisy_measurements = 0; // perfect measurement
        let p = 0.005;
        let bias_eta = 1e6;
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        let mut noise_model = NoiseModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut noise_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        let mut decoder =
            ParallelHyperionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        {
            // specific Z error pattern on tailored code
            simulator.clear_all_errors();
            simulator.set_error_check(&noise_model, &pos!(0, 5, 5), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 6, 4), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            println!("sparse measurement: {:?}", sparse_measurement);
            let (correction, _runtime_statistics) = decoder.decode(&sparse_measurement);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        {
            // random error generation test under biased noise
            let mut logical_error_count = 0;
            let total_rounds = 6;
            for round in 0..total_rounds {
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, _runtime_statistics) = decoder.decode(&sparse_measurement);
                code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
                let (logical_i, logical_j) = simulator.validate_correction(&correction);
                if logical_i || logical_j {
                    logical_error_count += 1;
                }
                if round < 5 {
                    println!("round {}: logical_i={}, logical_j={}", round, logical_i, logical_j);
                }
            }
            println!("logical error rate: {}/{}", logical_error_count, total_rounds);
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }

    /// Test with StandardTailoredCode (non-rotated) under biased noise, code capacity
    #[test]
    fn parallel_hyperion_decoder_standard_tailored_code() {
        // cargo test parallel_hyperion_decoder_standard_tailored_code -- --nocapture
        let d = 5;
        let noisy_measurements = 0;
        let p = 0.001;
        let bias_eta = 1e6;
        let mut simulator = Simulator::new(CodeType::StandardTailoredCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        let mut noise_model = NoiseModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut noise_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        let mut decoder =
            ParallelHyperionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        {
            let mut logical_error_count = 0;
            let total_rounds = 6;
            for round in 0..total_rounds {
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, _runtime_statistics) = decoder.decode(&sparse_measurement);
                code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
                let (logical_i, logical_j) = simulator.validate_correction(&correction);
                if logical_i || logical_j {
                    logical_error_count += 1;
                }
                if round < 5 {
                    println!("round {}: logical_i={}, logical_j={}", round, logical_i, logical_j);
                }
            }
            println!("logical error rate: {}/{}", logical_error_count, total_rounds);
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }

    /// Test with RotatedTailoredCode, moderate bias (not infinitely biased)
    #[test]
    fn parallel_hyperion_decoder_tailored_moderate_bias() {
        // cargo test parallel_hyperion_decoder_tailored_moderate_bias -- --nocapture
        let d = 5;
        let noisy_measurements = 0;
        let p = 0.002;
        let bias_eta = 100.;
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        let mut noise_model = NoiseModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut noise_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        let mut decoder =
            ParallelHyperionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        {
            let mut logical_error_count = 0;
            let total_rounds = 6;
            for round in 0..total_rounds {
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, _runtime_statistics) = decoder.decode(&sparse_measurement);
                code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
                let (logical_i, logical_j) = simulator.validate_correction(&correction);
                if logical_i || logical_j {
                    logical_error_count += 1;
                }
                if round < 5 {
                    println!("round {}: logical_i={}, logical_j={}", round, logical_i, logical_j);
                }
            }
            println!("logical error rate: {}/{}", logical_error_count, total_rounds);
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }

    /// Test with RotatedTailoredCode using substitute_with_simple_graph config option
    #[test]
    fn parallel_hyperion_decoder_tailored_simple_graph() {
        // cargo test parallel_hyperion_decoder_tailored_simple_graph -- --nocapture
        let d = 5;
        let noisy_measurements = 0;
        let p = 0.005;
        let bias_eta = 1e6;
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        let mut noise_model = NoiseModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut noise_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        let decoder_config = json!({
            "substitute_with_simple_graph": true,
        });
        let mut decoder =
            ParallelHyperionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &decoder_config, 1, false);
        {
            let mut logical_error_count = 0;
            let total_rounds = 6;
            for round in 0..total_rounds {
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, _runtime_statistics) = decoder.decode(&sparse_measurement);
                code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
                let (logical_i, logical_j) = simulator.validate_correction(&correction);
                if logical_i || logical_j {
                    logical_error_count += 1;
                }
                if round < 5 {
                    println!("round {}: logical_i={}, logical_j={}", round, logical_i, logical_j);
                }
            }
            println!("logical error rate (simple graph): {}/{}", logical_error_count, total_rounds);
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }

    /// Test with RotatedTailoredCode under phenomenological noise (noisy_measurements > 0).
    #[test]
    fn parallel_hyperion_decoder_tailored_phenomenological() {
        // cargo test parallel_hyperion_decoder_tailored_phenomenological -- --nocapture
        let d = 5;
        let noisy_measurements = 5;
        let p = 0.001;
        let bias_eta = 100.;
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        let mut noise_model = NoiseModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut noise_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        let mut decoder =
            ParallelHyperionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        {
            let mut logical_error_count = 0;
            let total_rounds = 5;
            for round in 0..total_rounds {
                println!("round: {:?}", round);
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, _runtime_statistics) = decoder.decode(&sparse_measurement);
                code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
                let (logical_i, logical_j) = simulator.validate_correction(&correction);
                if logical_i || logical_j {
                    logical_error_count += 1;
                }
                if round < 5 {
                    println!("round {}: logical_i={}, logical_j={}", round, logical_i, logical_j);
                }
            }
            println!("logical error rate: {}/{}", logical_error_count, total_rounds);
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }

    /// Test with BP enabled on RotatedTailoredCode.
    #[test]
    fn parallel_hyperion_decoder_tailored_with_bp() {
        // cargo test parallel_hyperion_decoder_tailored_with_bp -- --nocapture 
        let d = 5;
        let noisy_measurements = 0;
        let p = 0.005;
        let bias_eta = 1e6;
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        let mut noise_model = NoiseModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut noise_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        let decoder_config = json!({
            "use_bp": true,
            "bp_iteration": 3,
            "bp_application_ratio": 0.1,
        });
        let mut decoder =
            ParallelHyperionDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &decoder_config, 1, false);
        {
            let mut logical_error_count = 0;
            let total_rounds = 6;
            for round in 0..total_rounds {
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, runtime_statistics) = decoder.decode(&sparse_measurement);
                code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
                let (logical_i, logical_j) = simulator.validate_correction(&correction);
                if logical_i || logical_j {
                    logical_error_count += 1;
                }
                if round < 3 {
                    println!(
                        "round {}: logical_i={}, logical_j={}, stats={}",
                        round, logical_i, logical_j, runtime_statistics
                    );
                }
            }
            println!("logical error rate with BP: {}/{}", logical_error_count, total_rounds);
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }
}
