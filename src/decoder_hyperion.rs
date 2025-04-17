//! Hypergraph Minimum-Weight Parity Subgraph decoder (Hyperion)

use super::decoder_mwpm::*;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use crate::model_hypergraph::*;
use crate::mwpf::{mwpf_solver::*, util::*};
use num_traits::One;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

pub struct HyperionDecoder {
    /// model hypergraph
    pub model_hypergraph: Arc<ModelHypergraph>,
    /// save configuration for later usage
    pub config: HyperionDecoderConfig,
    /// (approximate) minimum-weight parity factor solver
    pub solver: Box<dyn SolverTrait + Send>,
    /// the initializer of the solver, used for customized clone
    pub initializer: Arc<SolverInitializer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HyperionDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")] // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")] // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    #[serde(default = "hyperion_default_configs::uniform_weights")]
    pub uniform_weights: bool,
    #[serde(default = "hyperion_default_configs::default_hyperion_config")]
    pub hyperion_config: serde_json::Value,
    #[serde(default = "hyperion_default_configs::substitute_with_simple_graph")]
    pub substitute_with_simple_graph: bool,
    #[serde(default = "hyperion_default_configs::use_bp")]
    pub use_bp: bool,
    #[serde(default = "hyperion_default_configs::bp_iteration")]
    pub bp_iteration: usize,
    #[serde(default = "hyperion_default_configs::bp_application_ratio")]
    pub bp_application_ratio: f64,
}

pub mod hyperion_default_configs {
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
}

impl Clone for HyperionDecoder {
    fn clone(&self) -> Self {
        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver: Box::new(SolverSerialJointSingleHair::new(
                &self.initializer,
                self.config.hyperion_config.clone(),
            )) as Box<dyn SolverTrait + Send>,
            initializer: self.initializer.clone(),
        }
    }
}

impl HyperionDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(
        simulator: &Simulator,
        noise_model: Arc<NoiseModel>,
        decoder_configuration: &serde_json::Value,
        parallel: usize,
        use_brief_edge: bool,
    ) -> Self {
        // read attribute of decoder configuration
        let config: HyperionDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
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

        let mut initializer = SolverInitializer::new(vertex_num, weighted_edges);
        if config.uniform_weights {
            initializer.uniform_weights(Weight::one());
        }
        let initializer = Arc::new(initializer);
        let mut solver: Box<dyn SolverTrait + Send> =
            Box::new(SolverSerialJointSingleHair::new(&initializer, config.hyperion_config.clone()))
                as Box<dyn SolverTrait + Send>;
        if config.use_bp {
            solver = match SolverBPWrapper::new(solver.solver_base(), config.bp_iteration, config.bp_application_ratio)
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
