//! Hypergraph Minimum-Weight Parity Subgraph decoder (Hyperion)

use super::decoder_mwpm::*;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use crate::model_hypergraph::*;
use crate::mwpf::mwpf_solver::*;
use crate::mwpf::util::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

pub struct HyperionDecoder {
    /// model hypergraph
    pub model_hypergraph: Arc<ModelHypergraph>,
    /// save configuration for later usage
    pub config: HyperionDecoderConfig,
    /// (approximate) minimum-weight parity subgraph solver
    pub solver: SolverSerialJointSingleHair,
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
    /// the maximum integer weight after scaling
    #[serde(alias = "mhw")] // abbreviation
    #[serde(default = "hyper_union_find_default_configs::max_weight")]
    pub max_weight: usize,
}

pub mod hyper_union_find_default_configs {
    pub fn max_weight() -> usize {
        1000000
    }
}

impl Clone for HyperionDecoder {
    fn clone(&self) -> Self {
        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver: SolverSerialJointSingleHair::new(&self.initializer),
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
        model_hypergraph.build(
            &mut simulator,
            Arc::clone(&noise_model),
            &config.weight_function,
            parallel,
            config.use_combined_probability,
            use_brief_edge,
        );
        let model_hypergraph = Arc::new(model_hypergraph);
        let (vertex_num, weighted_edges) = model_hypergraph.generate_mwpf_hypergraph(config.max_weight);
        let initializer = Arc::new(SolverInitializer::new(vertex_num, weighted_edges));
        let solver = SolverSerialJointSingleHair::new(&initializer);
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
                *self
                    .model_hypergraph
                    .vertex_indices
                    .get(position)
                    .expect("measurement cannot happen at impossible position")
            })
            .collect();
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
        (
            correction,
            json!({
                "time_decode": time_decode,
                "time_build_correction": time_build_correction,
            }),
        )
    }
}
