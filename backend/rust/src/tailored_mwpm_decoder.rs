//! tailored surface code MWPM decoder
//! 

use serde::{Serialize, Deserialize};
use super::simulator::*;
use super::error_model::*;
use super::model_graph::*;
use super::mwpm_decoder::*;
use super::tailored_model_graph::*;
use super::tailored_complete_model_graph::*;
use super::serde_json;
use std::sync::{Arc};
use std::time::Instant;
use super::blossom_v;
use super::union_find_decoder::UnionFind;
use super::types::*;

/// MWPM decoder, initialized and cloned for multiple threads
#[derive(Debug, Clone, Serialize)]
pub struct TailoredMWPMDecoder {
    /// model graph is immutably shared
    pub tailored_model_graph: Arc<TailoredModelGraph>,
    /// complete model graph each thread maintain its own precomputed data
    pub tailored_complete_model_graph: TailoredCompleteModelGraph,
    /// normal MWPM decoder to handle residual decoding
    pub mwpm_decoder: MWPMDecoder,
    /// virtual nodes for correction
    pub virtual_nodes: Arc<Vec<Position>>,
    /// base simulator, which is immutable but can be used to check code information
    #[serde(skip)]
    pub simulator: Arc<Simulator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TailoredMWPMDecoderConfig {
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

impl TailoredMWPMDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: &ErrorModel, decoder_configuration: &serde_json::Value) -> Self {
        // read attribute of decoder configuration
        let config: MWPMDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
        tailored_model_graph.build(&mut simulator, &error_model, &config.weight_function);
        // build complete model graph
        let mut tailored_complete_model_graph = TailoredCompleteModelGraph::new(&simulator, &tailored_model_graph);
        tailored_complete_model_graph.precompute(&simulator, &tailored_model_graph, config.precompute_complete_model_graph);
        // build virtual nodes for decoding use
        let mut virtual_nodes = Vec::new();
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if tailored_model_graph.is_node_exist(position) {
            let node = simulator.get_node_unwrap(position);
            if node.is_virtual {
                virtual_nodes.push(position.clone());
            }
        });
        // build MWPM decoder
        let mwpm_decoder = MWPMDecoder::new(&simulator, error_model, decoder_configuration);
        Self {
            tailored_model_graph: Arc::new(tailored_model_graph),
            tailored_complete_model_graph: tailored_complete_model_graph,
            mwpm_decoder: mwpm_decoder,
            virtual_nodes: Arc::new(virtual_nodes),
            simulator: Arc::new(simulator),
        }
    }

    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        let mut correction = SparseCorrection::new();
        // list nontrivial measurements to be matched
        let to_be_matched = sparse_measurement.to_vec();
        let mut time_tailored_prepare_graph = 0.;
        let mut time_tailored_blossom_v = 0.;
        let mut time_tailored_union = 0.;
        let mut time_neutral_prepare_graph = 0.;
        let mut time_neutral_blossom_v = 0.;
        let mut time_residual_decoding = 0.;
        let mut time_build_correction = 0.;
        if to_be_matched.len() > 0 {
            let begin = Instant::now();
            // vertices layout: [positive real nodes] [positive virtual nodes] [negative real nodes] [negative virtual nodes]
            // since positive and negative nodes have the same position, only ([positive real nodes] [positive virtual nodes]) is saved in `to_be_matched`
            let real_len = to_be_matched.len();
            let virtual_len = self.virtual_nodes.len();
            // append virtual nodes behind real ones
            let mut tailored_to_be_matched = to_be_matched.clone();
            for i in 0..virtual_len {
                tailored_to_be_matched.push(self.virtual_nodes[i].clone());
            }
            let tailored_to_be_matched = tailored_to_be_matched;  // change to immutable
            let tailored_len = tailored_to_be_matched.len();
            debug_assert!(tailored_len == real_len + virtual_len);
            // invalidate previous cache to save memory
            self.tailored_complete_model_graph.invalidate_previous_dijkstra();
            // construct edges
            let mut tailored_weighted_edges = Vec::<(usize, usize, f64)>::new();
            for i in 0..tailored_len {
                let position = &tailored_to_be_matched[i];
                let [positive_edges, negative_edges] = self.tailored_complete_model_graph.get_tailored_matching_edges(position, &tailored_to_be_matched);
                for &(j, weight) in positive_edges.iter() {
                    tailored_weighted_edges.push((i, j, weight));
                    // println!{"positive edge {} {} {} ", tailored_to_be_matched[i], tailored_to_be_matched[j], weight};
                }
                for &(j, weight) in negative_edges.iter() {
                    tailored_weighted_edges.push((tailored_len + i, tailored_len + j, weight));
                    // println!{"negative edge {} {} {} ", tailored_to_be_matched[i], tailored_to_be_matched[j], weight};
                }
                // virtual nodes are connected with 0 weight
                if i >= real_len {
                    tailored_weighted_edges.push((i, tailored_len + i, 0.));
                }
            }
            time_tailored_prepare_graph += begin.elapsed().as_secs_f64();
            // match tailored graph
            let begin = Instant::now();
            debug_assert!({  // sanity check: edges are valid
                let mut all_edges_valid = true;
                for &(i, j, weight) in tailored_weighted_edges.iter() {
                    if i >= tailored_len * 2 || j >= tailored_len * 2 {
                        eprintln!("[error] invalid edge {} {} weight = {}", tailored_to_be_matched[i % tailored_len], tailored_to_be_matched[j % tailored_len], weight);
                        all_edges_valid = false;
                    }
                }
                all_edges_valid
            });
            let tailored_matching = blossom_v::safe_minimum_weight_perfect_matching(tailored_len * 2, tailored_weighted_edges);
            time_tailored_blossom_v += begin.elapsed().as_secs_f64();
            // union-find tailored clusters
            let begin = Instant::now();
            let mut tailored_clusters = UnionFind::new(tailored_len);
            for i in 0..tailored_len {  // set `cardinality` to 1 if the position is a StabY
                let position = &tailored_to_be_matched[i];
                let node = self.simulator.get_node_unwrap(position);
                if node.qubit_type == QubitType::StabY {
                    tailored_clusters.payload[i].as_mut().unwrap().cardinality = 1;
                }
            }
            for i in 0..2*tailored_len {
                let j = tailored_matching[i];
                let base_i = i % tailored_len;
                let base_j = j % tailored_len;
                if base_i < base_j {  // no need to union if base_i == base_j; also no need to union base_i > base_j
                    // println!("    union {} {}", tailored_to_be_matched[base_i], tailored_to_be_matched[base_j]);
                    tailored_clusters.union(base_i, base_j);
                }
            }
            time_tailored_union += begin.elapsed().as_secs_f64();
            // do neutral decoding, only consider neutral clusters
            let begin = Instant::now();
            let mut neutral_to_be_matched = Vec::new();
            let mut neutral_to_tailored_mapping = Vec::new();
            let mut residual_to_be_matched = Vec::new();
            for i in 0..tailored_len {
                // filtering out positions matched with itself
                if tailored_clusters.get(i).set_size > 1 {
                    // only care about neutral clusters
                    // eprintln!("cluster {}: cardinality: {}", i, tailored_clusters.get(i).cardinality);
                    if tailored_clusters.get(i).cardinality % 2 == 0 {
                        neutral_to_tailored_mapping.push(i);
                        neutral_to_be_matched.push(tailored_to_be_matched[i].clone());
                    } else {
                        // residual must be real node
                        let position = tailored_to_be_matched[i].clone();
                        let node = self.simulator.get_node_unwrap(&position);
                        if !node.is_virtual {
                            residual_to_be_matched.push(position);
                        }
                    }
                }
            }
            let neutral_len = neutral_to_be_matched.len();
            // println!("neutral_to_be_matched: {:?}", neutral_to_be_matched);
            // construct edges
            let mut neutral_weighted_edges = Vec::<(usize, usize, f64)>::new();
            for i in 0..neutral_len {
                let edges = self.tailored_complete_model_graph.get_neutral_matching_edges(i, &neutral_to_be_matched, &neutral_to_tailored_mapping, &mut tailored_clusters);
                for &(j, weight) in edges.iter() {
                    neutral_weighted_edges.push((i, j, weight));
                    // println!{"neutral edge {} {} {} ", neutral_to_be_matched[i], neutral_to_be_matched[j], weight};
                }
            }
            time_neutral_prepare_graph += begin.elapsed().as_secs_f64();
            // match neutral graph
            let begin = Instant::now();
            debug_assert!({  // sanity check: edges are valid
                let mut all_edges_valid = true;
                for &(i, j, weight) in neutral_weighted_edges.iter() {
                    if i >= neutral_len || j >= neutral_len {
                        eprintln!("[error] invalid edge {} {} weight = {}", neutral_to_be_matched[i], neutral_to_be_matched[j], weight);
                        all_edges_valid = false;
                    }
                }
                all_edges_valid
            });
            debug_assert!({  // sanity check: each vertex has at least one edge
                let mut edges_count: Vec<usize> = (0..neutral_len).map(|_| 0).collect();
                for &(i, j, _weight) in neutral_weighted_edges.iter() {
                    edges_count[i] += 1;
                    edges_count[j] += 1;
                }
                let mut all_vertices_have_edge = true;
                for i in 0..neutral_len {
                    if edges_count[i] == 0 {
                        eprintln!("[error] vertex {} has no edge", neutral_to_be_matched[i]);
                        all_vertices_have_edge = false;
                    }
                }
                all_vertices_have_edge
            });
            let neutral_matching = blossom_v::safe_minimum_weight_perfect_matching(neutral_len, neutral_weighted_edges);
            time_neutral_blossom_v += begin.elapsed().as_secs_f64();
            // do residual decoding, instead of using the confusing method in the paper, I just match them together using normal graph
            let begin = Instant::now();
            let residual_correction = if residual_to_be_matched.len() > 0 {
                let (correction, _) = self.mwpm_decoder.decode(&SparseMeasurement::from_vec(&residual_to_be_matched));
                correction
            } else {
                SparseCorrection::new()
            };
            time_residual_decoding += begin.elapsed().as_secs_f64();
            // build correction based on the residual matching
            let begin = Instant::now();
            for i in 0..neutral_len {
                let j = neutral_matching[i];
                let a = &neutral_to_be_matched[i];
                if j < i {  // only add correction if j < i, so that the same correction is not applied twice
                    // println!("neutral match peer {:?} {:?}", neutral_to_be_matched[i], neutral_to_be_matched[j]);
                    let b = &neutral_to_be_matched[j];
                    let matching_correction = self.tailored_complete_model_graph.build_correction_neutral_matching(a, b, &self.tailored_model_graph);
                    correction.extend(&matching_correction);
                }
            }
            correction.extend(&residual_correction);
            time_build_correction += begin.elapsed().as_secs_f64();
        }
        (correction, json!({
            "to_be_matched": to_be_matched.len(),
            "time_tailored_prepare_graph": time_tailored_prepare_graph,
            "time_tailored_blossom_v": time_tailored_blossom_v,
            "time_tailored_union": time_tailored_union,
            "time_neutral_prepare_graph": time_neutral_prepare_graph,
            "time_neutral_blossom_v": time_neutral_blossom_v,
            "time_residual_decoding": time_residual_decoding,
            "time_build_correction": time_build_correction,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::code_builder::*;
    use super::super::types::ErrorType::*;

    #[test]
    fn tailored_mwpm_decoder_code_capacity() {  // cargo test tailored_mwpm_decoder_code_capacity -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.005;
        let bias_eta = 1e6;
        // build simulator
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode{ noisy_measurements, dp: d, dn: d });
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), &error_model, &decoder_config);
        {  // debug 5: no edges in residual graph
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 1, 5)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 2, 4)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        {  // debug 4
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 4)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 7, 3)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 4)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        {  // debug 3
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 1, 5)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 2, 6)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 8)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        {  // debug 2.5
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 7, 7)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        {  // debug 2
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 8)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 6)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            assert_eq!(sparse_measurement.to_vec(), vec![pos!(6, 5, 6), pos!(6, 5, 8), pos!(6, 6, 5), pos!(6, 7, 8), pos!(6, 8, 5), pos!(6, 9, 6)]);
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        {  // debug 1
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 3)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            assert_eq!(sparse_measurement.to_vec(), vec![pos!(6, 3, 4), pos!(6, 4, 5), pos!(6, 5, 2), pos!(6, 6, 3)]);
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
    }

}
