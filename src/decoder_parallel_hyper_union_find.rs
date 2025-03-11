//! Parallel Hypergraph Minimum-Weight Parity Subgraph decoder (Hyperion)
//! 

use mwpf::{bp::bp::*, mwpf_solver::*, util::*};
use super::decoder_mwpm::*;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use crate::model_hypergraph::*;
use crate::mwpf::util::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;
use mwpf::util::PartitionConfig;
use std::usize::MAX;



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
        let (vertex_num, weighted_edges) = self.model_hypergraph.generate_mwpf_hypergraph();
        // self.config.partition_config = Some(PartitionConfig::new(vertex_num));
        let mut partition_config = PartitionConfig::new(vertex_num);
        let mut partition_info = partition_config.info();
        if 2 > 0 {
            partition_config = graph_time_partition(&self.initializer, &self.model_hypergraph.vertex_positions, 2);
            partition_info = partition_config.info();
        }

        // let partition_config = PartitionConfig::new(vertex_num);
        // let mut partition_info = partition_config.info();
        // if 2 > 0 {
        //     self.config.partition_config = Some(graph_time_partition(&self.initializer, &self.model_hypergraph.vertex_positions, 2));
        //     partition_info = self.config.partition_config.clone().unwrap().info();
        // }

        // let partition_config = &self.config.partition_config;
        // let partition_info = partition_config.clone().unwrap().info();
        let solver = SolverParallelUnionFind::new(&self.initializer, &partition_info, self.config.hyperion_config.clone());

        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver: solver,
            initializer: self.initializer.clone(),
        }
    }
}

// impl std::fmt::Debug for ParallelHyperUnionFindDecoder {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         f.debug_struct("ParallelHyperUnionFindDecoder")
//             .field("model_hypergraph", &self.model_hypergraph)
//             .field("config", &self.config)
//             .field("solver", &self.solver)
//             .field("initializer", &self.initializer)
//             .finish()
//     }
// }


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
        let partition_config = PartitionConfig::new(vertex_num);
        let mut partition_info = partition_config.info();
        if 2 > 0 {
            // if let Some(ref mut partition) = config.partition_config {
            //     *partition = graph_time_partition(&initializer, &model_hypergraph.vertex_positions, 2);
            //     partition_info = partition.info();
            // }
            config.partition_config = Some(graph_time_partition(&initializer, &model_hypergraph.vertex_positions, 2));
            partition_info = config.partition_config.clone().unwrap().info();
        }
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
                *self
                    .model_hypergraph
                    .vertex_indices
                    .get(position)
                    .expect("measurement cannot happen at impossible position")
            })
            .collect();
        let syndrome_pattern = SyndromePattern::new(defect_vertices, vec![]);

        if 2 > 0 {
            if let Some(ref mut temp_partition_config) = self.config.partition_config {
                // *temp_partition_config = graph_time_partition(&self.initializer, &self.model_hypergraph.vertex_positions, 2);
                temp_partition_config.defect_vertices = BTreeSet::from_iter(syndrome_pattern.defect_vertices.clone());
            }
            let partition_info = self.config.partition_config.clone().unwrap().info();
            self.solver = SolverParallelUnionFind::new(&self.initializer, &partition_info, self.config.hyperion_config.clone());
        }
        
        self.solver.solve(syndrome_pattern);
        let subgraph = self.solver.subgraph().subgraph;
        // println!("subgraph in decode with erasure: {:?}", subgraph);
        self.solver.clear();

        // println!("after solver clear subgraph in decode with erasure: {:?}", subgraph);
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


/// test for time partition
#[allow(clippy::unnecessary_cast)]
pub fn graph_time_partition(initializer: &SolverInitializer, positions: &Vec<Position>, split_num: usize) -> PartitionConfig  {
    assert!(positions.len() > 0, "positive number of positions");
    let mut partition_config = PartitionConfig::new(initializer.vertex_num);
    let mut last_t = positions[0].t;
    let mut t_list: Vec<usize> = vec![];
    t_list.push(last_t);
    for position in positions {
        assert!(position.t >= last_t, "t not monotonically increasing, vertex reordering must be performed before calling this");
        if position.t != last_t {
            t_list.push(position.t);
        }
        last_t = position.t;
    }

    // pick the t value in the middle to split it
    let mut t_split_vec: Vec<usize> = vec![0; split_num - 1];
    for i in 0..(split_num - 1) {
        let index: usize = t_list.len()/split_num * (i + 1);
        t_split_vec[i] = t_list[index];
    }
    // find the vertices indices
    let mut split_start_index_vec = vec![MAX; split_num - 1];
    let mut split_end_index_vec = vec![MAX; split_num - 1];
    let mut start_index = 0;
    let mut end_index = 0;
    for (vertex_index, position) in positions.iter().enumerate() {
        if start_index < split_num - 1 {
            if split_start_index_vec[start_index] == MAX && position.t == t_split_vec[start_index] {
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
                // end_index += 1;
            }
        }
    }

    assert!(split_start_index_vec.iter().all(|&x| x != MAX), "Some elements in split_start_index_vec are equal to MAX");
    
    // partitions are found
    let mut graph_nodes = vec![];
    let mut partitions_vec = vec![];
    for i in 0..split_num  {
        if i == 0 {
            partitions_vec.push(VertexRange::new(0, split_start_index_vec[0]));
        } else if i == split_num - 1 {
            partitions_vec.push(VertexRange::new(split_end_index_vec[i - 1], positions.len()));
        } else {
            partitions_vec.push(VertexRange::new(split_end_index_vec[i - 1], split_start_index_vec[i]));
        }

        if i < split_num - 1 {
            partition_config.fusions.push((i, i+1));
        }
        
        let a = partition_config.dag_partition_units.add_node(());
        graph_nodes.push(a.clone());
    }
    partition_config.partitions = partitions_vec;

    for i in 0..split_num {
        if i < split_num - 1 {
            partition_config.dag_partition_units.add_edge(graph_nodes[i], graph_nodes[i+1], false);
        }
    }
    // partition_config.defect_vertices = BTreeSet::from_iter(defect_vertices.clone()); // this can be set outside of this function

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
        let noisy_measurements = 7; // perfect measurement
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
        let enable_all = false;
        let mut hyper_union_find_decoder =
            ParallelHyperUnionFindDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&noise_model), &json!({}), 1, false);
        if enable_all {
            // debug 5
            simulator.clear_all_errors();
            // {"[0][4][6]":"Z","[0][5][9]":"Z","[0][7][1]":"Z","[0][9][1]":"Z"}
            simulator.set_error_check(&noise_model, &pos!(0, 4, 6), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 5, 9), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 7, 1), &Z);
            simulator.set_error_check(&noise_model, &pos!(0, 9, 1), &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            println!("sparse measurement debug 5: {:?}", sparse_measurement);
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if enable_all {
            // debug 4, should fail
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
        if enable_all {
            // debug 3
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
        if enable_all {
            // debug 2
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
        if enable_all {
            // debug 1
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
        if true {
            // errors that failed 
            simulator.clear_all_errors();
            simulator.set_error_check(&noise_model, &pos!(15, 8, 9), &Z);
            simulator.set_error_check(&noise_model, &pos!(23, 5, 6), &X);
            simulator.set_error_check(&noise_model, &pos!(27, 5, 5), &Z);
            println!("before propagate errors");
            simulator.propagate_errors();
            println!("before generate sparse measurement");
            let sparse_measurement = simulator.generate_sparse_measurement();
            println!("sparse measurement: {:?}", sparse_measurement);
            let (correction, _runtime_statistics) = hyper_union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
    }
}
