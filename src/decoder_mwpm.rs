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
use super::erasure_graph::*;


/// MWPM decoder, initialized and cloned for multiple threads
#[derive(Debug, Clone, Serialize)]
pub struct MWPMDecoder {
    /// model graph is immutably shared
    pub model_graph: Arc<ModelGraph>,
    /// erasure graph is immutably shared
    pub erasure_graph: Arc<ErasureGraph>,
    /// complete model graph each thread maintain its own precomputed data; the internal model_graph might be copied and modified if erasure error exists
    pub complete_model_graph: CompleteModelGraph,
    /// save configuration for later usage
    pub config: MWPMDecoderConfig,
    /// an immutably shared simulator that is used to change model graph on the fly for correcting erasure errors
    pub simulator: Arc<Simulator>,
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
        model_graph.build(&mut simulator, Arc::clone(&error_model), &config.weight_function, parallel, config.use_combined_probability, use_brief_edge);
        let model_graph = Arc::new(model_graph);
        // build erasure graph
        let mut erasure_graph = ErasureGraph::new(&simulator);
        erasure_graph.build(&mut simulator, Arc::clone(&error_model), parallel);
        let erasure_graph = Arc::new(erasure_graph);
        // build complete model graph
        let mut complete_model_graph = CompleteModelGraph::new(&simulator, Arc::clone(&model_graph));
        complete_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel);
        Self {
            model_graph: model_graph,
            erasure_graph: erasure_graph,
            complete_model_graph: complete_model_graph,
            config: config,
            simulator: Arc::new(simulator),
        }
    }

    /// decode given measurement results
    #[allow(dead_code)]
    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        self.decode_with_erasure(sparse_measurement, &SparseDetectedErasures::new())
    }

    /// decode given measurement results and detected erasures
    pub fn decode_with_erasure(&mut self, sparse_measurement: &SparseMeasurement, sparse_detected_erasures: &SparseDetectedErasures) -> (SparseCorrection, serde_json::Value) {
        if sparse_detected_erasures.len() > 0 {
            assert!(self.config.precompute_complete_model_graph == false, "if erasure happens, the precomputed complete graph is invalid; please disable `precompute_complete_model_graph` or `pcmg` in the decoder configuration");
        }
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
            // update model graph weights to consider erasure information
            let mut erasure_graph_modifier = ErasureGraphModifier::<f64>::new();
            if sparse_detected_erasures.len() > 0 {  // if erasure exists, the model graph will be duplicated on demand
                let erasure_edges = sparse_detected_erasures.get_erasure_edges(&self.erasure_graph);
                let model_graph_mut = self.complete_model_graph.get_model_graph_mut();
                for erasure_edge in erasure_edges.iter() {
                    match erasure_edge {
                        ErasureEdge::Connection(position1, position2) => {
                            let node1 = model_graph_mut.get_node_mut_unwrap(position1);
                            let edge12 = node1.edges.get_mut(position2).expect("neighbor must exist");
                            let original_weight12 = edge12.weight;
                            edge12.weight = 0.;  // set to 0 because of erasure
                            let node2 = model_graph_mut.get_node_mut_unwrap(position2);
                            let edge21 = node2.edges.get_mut(position1).expect("neighbor must exist");
                            assert_eq!(original_weight12, edge21.weight, "model graph edge must be symmetric");
                            edge21.weight = 0.;  // set to 0 because of erasure
                            erasure_graph_modifier.push_modified_edge(ErasureEdge::Connection(position1.clone(), position2.clone()), original_weight12);
                        },
                        ErasureEdge::Boundary(position) => {
                            let node = model_graph_mut.get_node_mut_unwrap(position);
                            let boundary = node.boundary.as_mut().expect("boundary must exist").as_mut();
                            let original_weight = boundary.weight;
                            boundary.weight = 0.;
                            erasure_graph_modifier.push_modified_edge(ErasureEdge::Boundary(position.clone()), original_weight);
                        },
                    }
                }
                self.complete_model_graph.model_graph_changed(&self.simulator);
            }
            // invalidate previous cache to save memory
            self.complete_model_graph.invalidate_previous_dijkstra();
            for i in 0..m_len {
                let position = &to_be_matched[i];
                let (edges, boundary) = self.complete_model_graph.get_edges(position, &to_be_matched);
                match boundary {
                    Some(weight) => {
                        // eprintln!{"boundary {} {} ", i, weight};
                        weighted_edges.push((i, i + m_len, weight));
                    }, None => { }
                }
                for &(j, weight) in edges.iter() {
                    if i < j {  // remove duplicated edges
                        // eprintln!{"edge {} {} {} ", i, j, weight};
                        weighted_edges.push((i, j, weight));
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
            // recover the modified edges
            if sparse_detected_erasures.len() > 0 {
                let model_graph_mut = self.complete_model_graph.get_model_graph_mut();
                while erasure_graph_modifier.has_modified_edges() {
                    let (erasure_edge, weight) = erasure_graph_modifier.pop_modified_edge();
                    match erasure_edge {
                        ErasureEdge::Connection(position1, position2) => {
                            let node1 = model_graph_mut.get_node_mut_unwrap(&position1);
                            let edge12 = node1.edges.get_mut(&position2).expect("neighbor must exist");
                            assert_eq!(edge12.weight, 0., "why a non-zero edge needs to be recovered");
                            edge12.weight = weight;  // recover the weight
                            let node2 = model_graph_mut.get_node_mut_unwrap(&position2);
                            let edge21 = node2.edges.get_mut(&position1).expect("neighbor must exist");
                            assert_eq!(edge21.weight, 0., "why a non-zero edge needs to be recovered");
                            edge21.weight = weight;  // recover the weight
                        },
                        ErasureEdge::Boundary(position) => {
                            let node = model_graph_mut.get_node_mut_unwrap(&position);
                            let boundary = node.boundary.as_mut().expect("boundary must exist").as_mut();
                            assert_eq!(boundary.weight, 0., "why a non-zero edge needs to be recovered");
                            boundary.weight = weight;
                        },
                    }
                }
                // need to call here because if next round there are no erasure errors, the complete mode graph must still be in a consistent state
                self.complete_model_graph.model_graph_changed(&self.simulator);
            }
        }
        (correction, json!({
            "to_be_matched": to_be_matched.len(),
            "time_prepare_graph": time_prepare_graph,
            "time_blossom_v": time_blossom_v,
            "time_build_correction": time_build_correction,
        }))
    }

}


#[cfg(feature = "blossom_v")]
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::code_builder::*;
    use super::super::error_model_builder::*;
    
    // 2022.6.16: mwpm decoder should correct this pattern because UF decoder does
    // {"[0][1][5]":"Z","[0][2][6]":"Z","[0][4][4]":"X","[0][5][7]":"X","[0][9][7]":"Y"}, {"erasures":["[0][1][3]","[0][1][5]","[0][2][6]","[0][4][4]","[0][5][7]","[0][6][6]","[0][9][7]"]}
    // cargo run --release -- tool benchmark [5] [0] [0] --pes [0.1] --max_repeats 0 --min_failed_cases 10 --time_budget 60 --decoder mwpm --code_type StandardPlanarCode --error_model erasure-only-phenomenological -p0 --debug_print failed-error-pattern
    #[test]
    fn mwpm_decoder_debug_1() {  // cargo test mwpm_decoder_debug_1 -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.;
        let pe = 0.1;
        // build simulator
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let error_model_builder = ErrorModelBuilder::ErasureOnlyPhenomenological;
        error_model_builder.apply(&mut simulator, &mut error_model, &json!({}), p, 1., pe);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({});
        let mut mwpm_decoder = MWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        // load errors onto the simulator
        let sparse_error_pattern: SparseErrorPattern = serde_json::from_value(json!({"[0][1][5]":"Z","[0][2][6]":"Z","[0][4][4]":"X","[0][5][7]":"X","[0][9][7]":"Y"})).unwrap();
        let sparse_detected_erasures: SparseDetectedErasures = serde_json::from_value(json!({"erasures":["[0][1][3]","[0][1][5]","[0][2][6]","[0][4][4]","[0][5][7]","[0][6][6]","[0][9][7]"]})).unwrap();
        simulator.load_sparse_error_pattern(&sparse_error_pattern).expect("success");
        simulator.load_sparse_detected_erasures(&sparse_detected_erasures).expect("success");
        simulator.propagate_errors();
        let sparse_measurement = simulator.generate_sparse_measurement();
        println!("sparse_measurement: {:?}", sparse_measurement);
        let sparse_detected_erasures = simulator.generate_sparse_detected_erasures();
        let (correction, _runtime_statistics) = mwpm_decoder.decode_with_erasure(&sparse_measurement, &sparse_detected_erasures);
        code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        let (logical_i, logical_j) = simulator.validate_correction(&correction);
        assert!(!logical_i && !logical_j);
    }

}
