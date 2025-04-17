//! Hypergraph Minimum-Weight Parity Subgraph decoder (Hyperion)

use super::decoder_mwpm::*;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use crate::decoder_hyperion::HyperionDecoderConfig;
use crate::model_hypergraph::*;
use crate::mwpf::{mwpf_solver::*, util::*};
use mwpf::cli::graph_time_partition;
use mwpf::visualize::VisualizePosition;
use num_traits::One;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

pub struct ParallelHyperionDecoder {
    /// model hypergraph
    pub model_hypergraph: Arc<ModelHypergraph>,
    /// save configuration for later usage
    pub config: ParallelHyperionDecoderConfig,
    /// (approximate) minimum-weight parity factor solver
    pub solver: Box<dyn SolverTrait + Send>,
    /// the initializer of the solver, used for customized clone
    pub initializer: Arc<SolverInitializer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParallelHyperionDecoderConfig {
    pub hyperion_config: HyperionDecoderConfig,
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

impl Clone for ParallelHyperionDecoder {
    fn clone(&self) -> Self {
        // let (vertex_num, weighted_edges) = self.model_hypergraph.generate_mwpf_hypergraph();
        // // self.config.partition_config = Some(PartitionConfig::new(vertex_num));
        // let mut partition_config = PartitionConfig::new(vertex_num);
        // let mut partition_info = partition_config.info();
        // if 2 > 0 {
        //     partition_config = graph_time_partition(
        //         &self.initializer,
        //         &self
        //             .model_hypergraph
        //             .vertex_positions
        //             .iter()
        //             .map(|p| VisualizePosition {
        //                 i: p.i as f64,
        //                 j: p.j as f64,
        //                 t: p.t as f64,
        //             })
        //             .collect(),
        //         2,
        //     );
        //     partition_info = partition_config.info();
        // }

        // // let partition_config = PartitionConfig::new(vertex_num);
        // // let mut partition_info = partition_config.info();
        // // if 2 > 0 {
        // //     self.config.partition_config = Some(graph_time_partition(&self.initializer, &self.model_hypergraph.vertex_positions, 2));
        // //     partition_info = self.config.partition_config.clone().unwrap().info();
        // // }

        // // let partition_config = &self.config.partition_config;
        // // let partition_info = partition_config.clone().unwrap().info();
        // let solver = SolverParallelUnionFind::new(
        //     &self.initializer,
        //     &partition_info,
        //     serde_json::to_value(self.config.hyperion_config.clone()).unwrap(),
        // );
        // Self {
        //     model_hypergraph: self.model_hypergraph.clone(),
        //     config: self.config.clone(),
        //     solver: Box::new(solver) as Box<dyn SolverTrait + Send>,
        //     initializer: self.initializer.clone(),
        // }

        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver: Box::new(SolverParallelJointSingleHair::new(
                &self.initializer,
                &self.config.partition_config.as_ref().unwrap().info(),
                self.config.hyperion_config.hyperion_config.clone(),
            )) as Box<dyn SolverTrait + Send>,
            initializer: self.initializer.clone(),
        }
    }
}

impl ParallelHyperionDecoder {
    /// create a new MWPM decoder with decoder configuration
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
        if config.hyperion_config.substitute_with_simple_graph {
            let mut model_graph = ModelGraph::new(&simulator);
            model_graph.build(
                &mut simulator,
                noise_model,
                &config.hyperion_config.weight_function,
                parallel,
                config.hyperion_config.use_combined_probability,
                use_brief_edge,
            );
            model_hypergraph.load_from_model_graph(&model_graph);
        } else {
            model_hypergraph.build(
                &mut simulator,
                Arc::clone(&noise_model),
                &config.hyperion_config.weight_function,
                parallel,
                config.hyperion_config.use_combined_probability,
                use_brief_edge,
            );
        }
        let model_hypergraph = Arc::new(model_hypergraph);
        let (vertex_num, weighted_edges) = model_hypergraph.generate_mwpf_hypergraph();

        let mut initializer = SolverInitializer::new(vertex_num, weighted_edges);
        if config.hyperion_config.uniform_weights {
            initializer.uniform_weights(Weight::one());
        }
        let initializer = Arc::new(initializer);

        let partition_config = PartitionConfig::new(vertex_num);
        let mut partition_info = partition_config.info();
        if 2 > 0 {
            // if let Some(ref mut partition) = config.partition_config {
            //     *partition = graph_time_partition(&initializer, &model_hypergraph.vertex_positions, 2);
            //     partition_info = partition.info();
            // }
            config.partition_config = Some(graph_time_partition(
                &initializer,
                &model_hypergraph
                    .vertex_positions
                    .iter()
                    .map(|p| VisualizePosition {
                        i: p.i as f64,
                        j: p.j as f64,
                        t: p.t as f64,
                    })
                    .collect::<Vec<_>>(),
                2,
            ));
            partition_info = config.partition_config.clone().unwrap().info();
        }
        let solver = SolverParallelJointSingleHair::new(
            &initializer,
            &partition_info,
            config.hyperion_config.hyperion_config.clone(),
        );

        let mut solver: Box<dyn SolverTrait + Send> = Box::new(solver) as Box<dyn SolverTrait + Send>;

        if config.hyperion_config.use_bp {
            solver = match SolverBPWrapper::new(
                solver.solver_base(),
                config.hyperion_config.bp_iteration,
                config.hyperion_config.bp_application_ratio,
            )
            .solver
            .inner
            {
                SolverEnum::SolverSerialUnionFind(x) => Box::new(x) as Box<dyn SolverTrait + Send>,
                SolverEnum::SolverSerialSingleHair(x) => Box::new(x) as Box<dyn SolverTrait + Send>,
                SolverEnum::SolverSerialJointSingleHair(x) => Box::new(x) as Box<dyn SolverTrait + Send>,
                SolverEnum::SolverErrorPatternLogger(_) => panic!("not supported"),
                SolverEnum::SolverParallelUnionFind(x) => Box::new(x) as Box<dyn SolverTrait + Send>,
                SolverEnum::SolverParallelSingleHair(x) => Box::new(x) as Box<dyn SolverTrait + Send>,
                SolverEnum::SolverParallelJointSingleHair(x) => Box::new(x) as Box<dyn SolverTrait + Send>,
            };
        }

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
                let temp = *self
                    .model_hypergraph
                    .vertex_indices
                    .get(position)
                    .expect("measurement cannot happen at impossible position");
                temp
            })
            .collect();

        let syndrome_pattern = SyndromePattern::new_vertices(defect_vertices);

        if 2 > 0 {
            if let Some(ref mut temp_partition_config) = self.config.partition_config {
                // *temp_partition_config = graph_time_partition(&self.initializer, &self.model_hypergraph.vertex_positions, 2);
                temp_partition_config.defect_vertices = FastIterSet::from_iter(syndrome_pattern.defect_vertices.clone());
            }
            let partition_info = self.config.partition_config.clone().unwrap().info();
            self.solver = Box::new(SolverParallelJointSingleHair::new(
                &self.initializer,
                &partition_info,
                self.config.hyperion_config.hyperion_config.clone(),
            )) as Box<dyn SolverTrait + Send>;
        }

        let decoder_begin = Instant::now();

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
