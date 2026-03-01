//! Parallel Hypergraph Minimum-Weight Parity Subgraph decoder (Hyperion)
//! 

use mwpf::{mwpf_solver::*, util::*};
use super::decoder_mwpm::*;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use crate::model_hypergraph::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;



pub struct ParallelHyperUnionFindDecoder {
    /// model hypergraph
    pub model_hypergraph: Arc<ModelHypergraph>,
    /// save configuration for later usage
    pub config: ParallelHyperUnionFindDecoderConfig,
    /// (approximate) minimum-weight parity subgraph solver
    pub solver: SolverParallelUnionFind,
    /// the initializer of the solver, used for customized clone
    pub initializer: Arc<SolverInitializer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParallelHyperUnionFindDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")] // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")] // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    #[serde(default = "parallel_hyper_union_find_default_configs::default_hyperion_config")]
    pub hyperion_config: serde_json::Value,
    #[serde(default = "parallel_hyper_union_find_default_configs::partition_config")]
    pub partition_config: Option<PartitionConfig>,
}

pub mod parallel_hyper_union_find_default_configs {
    pub fn default_hyperion_config() -> serde_json::Value {
        json!({})
    }

    use mwpf::util::PartitionConfig;
    pub fn partition_config() -> Option<PartitionConfig> {
        None
    }
}

impl Default for ParallelHyperUnionFindDecoderConfig {
    fn default() -> Self {
        Self {
            weight_function: mwpm_default_configs::weight_function(),
            use_combined_probability: mwpm_default_configs::use_combined_probability(),
            hyperion_config: parallel_hyper_union_find_default_configs::default_hyperion_config(),
            partition_config: parallel_hyper_union_find_default_configs::partition_config(),
        }
    }
}

impl Clone for ParallelHyperUnionFindDecoder {
    fn clone(&self) -> Self {
        let partition_config = graph_time_partition(&self.initializer, &self.model_hypergraph.vertex_positions, 2);
        let partition_info = partition_config.info();
        let solver = SolverParallelUnionFind::new(&self.initializer, &partition_info, self.config.hyperion_config.clone());

        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver,
            initializer: self.initializer.clone(),
        }
    }
}


impl ParallelHyperUnionFindDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(
        simulator: &Simulator,
        noise_model: Arc<NoiseModel>,
        decoder_configuration: &serde_json::Value,
        parallel: usize,
        use_brief_edge: bool,
    ) -> Self {
        // read attribute of decoder configuration
        let mut config: ParallelHyperUnionFindDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap_or_default();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_hypergraph = ModelHypergraph::new(&simulator);
        model_hypergraph.build(
            &mut simulator,
            Arc::clone(&noise_model),
            &config.weight_function,
            parallel,
            config.use_combined_probability,
            use_brief_edge,
        );
        let model_hypergraph = Arc::new(model_hypergraph);
        let (vertex_num, weighted_edges) = model_hypergraph.generate_mwpf_hypergraph();
        let initializer = Arc::new(SolverInitializer::new(vertex_num, weighted_edges));
        config.partition_config = Some(graph_time_partition(&initializer, &model_hypergraph.vertex_positions, 2));
        let partition_info = config.partition_config.clone().unwrap().info();
        let solver = SolverParallelUnionFind::new(&initializer, &partition_info, config.hyperion_config.clone());

        Self {
            model_hypergraph,
            config: config.clone(),
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
        let defect_vertices: Vec<_> = sparse_measurement
            .iter()
            .map(|position| {
                (0, *self
                    .model_hypergraph
                    .vertex_indices
                    .get(position)
                    .expect("measurement cannot happen at impossible position"))
            })
            .collect();
        let syndrome_pattern = SyndromePattern::new_vertices(defect_vertices);

        if let Some(ref mut temp_partition_config) = self.config.partition_config {
            temp_partition_config.defect_vertices = FastIterSet::from_iter(syndrome_pattern.defect_vertices.iter().map(|v| v.1));
        }
        let partition_info = self.config.partition_config.clone().unwrap().info();
        self.solver = SolverParallelUnionFind::new(&self.initializer, &partition_info, self.config.hyperion_config.clone());
        
        self.solver.solve(syndrome_pattern);
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
        (
            correction,
            json!({
                "time_decode": time_decode,
                "time_build_correction": time_build_correction,
            }),
        )
    }
}


/// Build a time-based partition config by splitting vertices along the time axis.
#[allow(clippy::unnecessary_cast)]
pub fn graph_time_partition(initializer: &SolverInitializer, positions: &Vec<Position>, split_num: usize) -> PartitionConfig {
    assert!(!positions.is_empty(), "positive number of positions");
    assert!(split_num >= 2, "split_num must be at least 2");

    let mut last_t = positions[0].t;
    let mut t_list: Vec<usize> = vec![last_t];
    for position in positions {
        assert!(position.t >= last_t, "t not monotonically increasing, vertex reordering must be performed before calling this");
        if position.t != last_t {
            t_list.push(position.t);
            last_t = position.t;
        }
    }

    // pick the t values to split at
    let mut t_split_vec: Vec<usize> = vec![0; split_num - 1];
    for i in 0..(split_num - 1) {
        let index: usize = t_list.len() / split_num * (i + 1);
        t_split_vec[i] = t_list[index];
    }

    // find the vertex indices for each split boundary
    let mut split_start_index_vec = vec![usize::MAX; split_num - 1];
    let mut split_end_index_vec = vec![usize::MAX; split_num - 1];
    let mut start_index = 0;
    let mut end_index = 0;
    for (vertex_index, position) in positions.iter().enumerate() {
        if start_index < split_num - 1 {
            if split_start_index_vec[start_index] == usize::MAX && position.t == t_split_vec[start_index] {
                split_start_index_vec[start_index] = vertex_index;
                if start_index != 0 {
                    end_index += 1;
                }
                start_index += 1;
            }
        }

        if end_index < split_num - 1 {
            if position.t == t_split_vec[end_index] {
                split_end_index_vec[end_index] = vertex_index + 1;
            }
        }
    }

    assert!(split_start_index_vec.iter().all(|&x| x != usize::MAX), "Some split boundaries were not found");

    // build partitions and fusions
    let mut partitions_vec = vec![];
    let mut fusions_vec = vec![];
    for i in 0..split_num {
        if i == 0 {
            partitions_vec.push(Partition::new(IndexRange::new(0, split_start_index_vec[0])));
        } else if i == split_num - 1 {
            partitions_vec.push(Partition::new(IndexRange::new(split_end_index_vec[i - 1], positions.len())));
        } else {
            partitions_vec.push(Partition::new(IndexRange::new(split_end_index_vec[i - 1], split_start_index_vec[i])));
        }

        if i < split_num - 1 {
            fusions_vec.push((i, i + 1));
        }
    }

    let mut partition_config = PartitionConfig::new(
        initializer.vertex_num,
        partitions_vec,
        fusions_vec,
        FastIterSet::new(),  // defect_vertices set later per decode call
    );

    // build the DAG for partition ordering
    let mut graph_nodes = vec![];
    for _i in 0..split_num {
        graph_nodes.push(partition_config.dag_partition_units.add_node(()));
    }
    for i in 0..(split_num - 1) {
        partition_config.dag_partition_units.add_edge(graph_nodes[i], graph_nodes[i + 1], false);
    }

    partition_config
}


#[cfg(test)]
mod tests {
    use super::super::code_builder::*;
    use super::super::types::ErrorType::*;
    use super::*;

    #[test]
    fn parallel_hyper_union_find_decoder_code_capacity() {
        // cargo test parallel_hyper_union_find_decoder_code_capacity -- --nocapture
        let d = 7;
        let noisy_measurements = 7;
        let p = 0.001;
        // build simulator
        let mut simulator = Simulator::new(CodeType::RotatedPlanarCode, CodeSize::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build noise model
        let mut noise_model = NoiseModel::new(&simulator);
        simulator.set_error_rates(&mut noise_model, p, p, p, 0.);
        simulator.compress_error_rates(&mut noise_model);
        noise_model_sanity_check(&simulator, &noise_model).unwrap();
        let noise_model = Arc::new(noise_model);
        // build decoder
        let mut hyper_union_find_decoder =
            ParallelHyperUnionFindDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        {
            // specific error pattern that previously failed
            simulator.clear_all_errors();
            simulator.set_error_check(&noise_model, &pos!(15, 8, 9), &Z);
            simulator.set_error_check(&noise_model, &pos!(23, 5, 6), &X);
            simulator.set_error_check(&noise_model, &pos!(27, 5, 5), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            println!("sparse measurement: {:?}", sparse_measurement);
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        {
            // random error generation test
            let mut logical_error_count = 0;
            let total_rounds = 100;
            for round in 0..total_rounds {
                simulator.clear_all_errors();
                simulator.generate_random_errors(&noise_model);
                let sparse_measurement = simulator.generate_sparse_measurement();
                let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
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
            // at p=0.001 with d=7, logical error rate should be very low
            assert!(logical_error_count <= 5, "too many logical errors: {}/{}", logical_error_count, total_rounds);
        }
    }
}
