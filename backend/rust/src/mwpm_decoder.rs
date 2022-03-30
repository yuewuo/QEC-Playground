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
    /// complete model graph each thread maintain its own precomputed data
    pub complete_model_graph: CompleteModelGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MWPMDecoderConfig {
    /// build complete model graph at first, but this will consume O(N^2) memory and increase initialization time,
    /// disable this when you're simulating large code
    #[serde(alias = "pcmg")]  // abbreviation
    #[serde(default = "mwpm_default_configs::precompute_complete_model_graph")]
    pub precompute_complete_model_graph: bool,
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
}

pub mod mwpm_default_configs {
    use super::*;
    pub fn precompute_complete_model_graph() -> bool { false }  // save for erasure error model and also large code distance
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
        // build complete model graph
        let mut complete_model_graph = CompleteModelGraph::new(&simulator, &model_graph);
        complete_model_graph.precompute(&simulator, &model_graph, config.precompute_complete_model_graph);
        Self {
            model_graph: Arc::new(model_graph),
            complete_model_graph: complete_model_graph,
        }
    }

    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        (SparseCorrection::new(), json!({}))
    }

}
