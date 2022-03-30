//! minimum-weight perfect matching decoder
//! 

use serde::{Serialize, Deserialize};
use super::simulator::*;
use super::error_model::*;
use super::model_graph::*;
use super::complete_model_graph::*;
use super::serde_json;
use std::sync::{Arc};

/// MWPM decoder, initialized and cloned for multiple threads
#[derive(Debug, Clone, Serialize)]
pub struct MWPMDecoder {
    /// model graph is immutably shared
    pub model_graph: Arc<ModelGraph>,
    /// complete model graph share precomputed data but each thread maintains variables
    pub complete_model_graph: CompleteModelGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MWPMDecoderConfig {
    /// default 0: no precomputed complete graph; set to positive integer to have finite entries for each graph vertex;
    /// set to any negative number to have infinite entries for each graph vertex (may consume tons of time for large graph)
    #[serde(alias = "pcmgms")]  // abbreviation
    #[serde(default = "mwpm_default_configs::precompute_complete_model_graph_max_size")]
    pub precompute_complete_model_graph_max_size: i64,
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
}

pub mod mwpm_default_configs {
    use super::*;
    pub fn precompute_complete_model_graph_max_size() -> i64 { 0 }  // default to disable precomputed complete graph
    pub fn weight_function() -> WeightFunction { WeightFunction::AutotuneImproved }
}

impl MWPMDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: &ErrorModel, decoder_configuration: &serde_json::Value) -> Self {
        // read attribute of decoder configuration
        let config: MWPMDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_graph = ModelGraph::new(&simulator);
        model_graph.build(&mut simulator, &error_model, &config.weight_function);
        let mut complete_model_graph = CompleteModelGraph::new(&simulator, &model_graph);
        let precompute_complete_model_graph_max_size = if config.precompute_complete_model_graph_max_size >= 0 {
            config.precompute_complete_model_graph_max_size as usize
        } else { usize::MAX };
        // compute complete model graph
        complete_model_graph.precompute(&simulator, &model_graph, precompute_complete_model_graph_max_size);
        Self {
            model_graph: Arc::new(model_graph),
            complete_model_graph: complete_model_graph,
        }
    }

    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        (SparseCorrection::new(), json!({}))
    }

}
