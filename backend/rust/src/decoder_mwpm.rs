//! minimum-weight perfect matching decoder
//! 

use serde::{Serialize, Deserialize};
use super::simulator::*;
use super::error_model::*;
use super::model_graph::*;
use super::complete_model_graph::*;
use super::serde_json;
use std::sync::{Arc};
use std::time::Instant;
use super::blossom_v;

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
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. error model
    #[serde(alias = "ucp")]  // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
}

pub mod mwpm_default_configs {
    use super::*;
    pub fn precompute_complete_model_graph() -> bool { false }  // save for erasure error model and also large code distance
    pub fn weight_function() -> WeightFunction { WeightFunction::AutotuneImproved }
    pub fn use_combined_probability() -> bool { true }  // default use combined probability for better accuracy
}

impl MWPMDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: Arc<ErrorModel>, decoder_configuration: &serde_json::Value, parallel: usize, use_brief_edge: bool) -> Self {
        // read attribute of decoder configuration
        let config: MWPMDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_graph = ModelGraph::new(&simulator);
        model_graph.build(&mut simulator, error_model, &config.weight_function, parallel, config.use_combined_probability, use_brief_edge);
        let model_graph = Arc::new(model_graph);
        // build complete model graph
        let mut complete_model_graph = CompleteModelGraph::new(&simulator, Arc::clone(&model_graph));
        complete_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel);
        Self {
            model_graph: model_graph,
            complete_model_graph: complete_model_graph,
        }
    }

    /// decode given measurement results
    #[allow(dead_code)]
    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        self.decode_with_erasure(sparse_measurement, &SparseDetectedErasures::new())
    }

    /// decode given measurement results and detected erasures
    pub fn decode_with_erasure(&mut self, sparse_measurement: &SparseMeasurement, sparse_detected_erasures: &SparseDetectedErasures) -> (SparseCorrection, serde_json::Value) {
        assert!(sparse_detected_erasures.len() == 0, "unimplemented");
        let mut correction = SparseCorrection::new();
        // list nontrivial measurements to be matched
        let to_be_matched = sparse_measurement.to_vec();
        let mut time_prepare_graph = 0.;
        let mut time_blossom_v = 0.;
        let mut time_build_correction = 0.;
        if to_be_matched.len() > 0 {
            // println!{"to_be_matched: {:?}", to_be_matched};
            let begin = Instant::now();
            // add the edges to the graph
            let m_len = to_be_matched.len();  // virtual boundary of `i` is `i + m_len`
            let node_num = m_len * 2;
            // Z (X) stabilizers are (fully) connected, boundaries are fully connected
            // stabilizer to boundary is one-to-one connected
            let mut weighted_edges = Vec::<(usize, usize, f64)>::new();
            // invalidate previous cache to save memory
            self.complete_model_graph.invalidate_previous_dijkstra();
            for i in 0..m_len {
                let position = &to_be_matched[i];
                let (edges, boundary) = self.complete_model_graph.get_edges(position, &to_be_matched);
                match boundary {
                    Some(weight) => {
                        weighted_edges.push((i, i + m_len, weight));
                    }, None => { }
                }
                for &(j, weight) in edges.iter() {
                    if i < j {  // remove duplicated edges
                        weighted_edges.push((i, j, weight));
                        // println!{"edge {} {} {} ", i, j, weight};
                    }
                }
                for j in (i+1)..m_len {
                    // virtual boundaries are always fully connected
                    weighted_edges.push((i + m_len, j + m_len, 0.));
                }
            }
            time_prepare_graph += begin.elapsed().as_secs_f64();
            // run the Blossom algorithm
            let begin = Instant::now();
            let matching = blossom_v::safe_minimum_weight_perfect_matching(node_num, weighted_edges);
            time_blossom_v += begin.elapsed().as_secs_f64();
            // build correction based on the matching
            let begin = Instant::now();
            for i in 0..m_len {
                let j = matching[i];
                let a = &to_be_matched[i];
                if j < i {  // only add correction if j < i, so that the same correction is not applied twice
                    // println!("match peer {:?} {:?}", to_be_matched[i], to_be_matched[j]);
                    let b = &to_be_matched[j];
                    let matching_correction = self.complete_model_graph.build_correction_matching(a, b);
                    correction.extend(&matching_correction);
                } else if j >= m_len {  // matched with boundary
                    // println!("match boundary {:?}", to_be_matched[i]);
                    let boundary_correction = self.complete_model_graph.build_correction_boundary(a);
                    correction.extend(&boundary_correction);
                }
            }
            time_build_correction += begin.elapsed().as_secs_f64();
        }
        (correction, json!({
            "to_be_matched": to_be_matched.len(),
            "time_prepare_graph": time_prepare_graph,
            "time_blossom_v": time_blossom_v,
            "time_build_correction": time_build_correction,
        }))
    }

}
