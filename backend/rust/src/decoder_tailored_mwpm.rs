//! tailored surface code MWPM decoder
//! 

use serde::{Serialize, Deserialize};
use super::simulator::*;
use super::error_model::*;
use super::model_graph::*;
use super::decoder_mwpm::*;
use super::tailored_model_graph::*;
use super::tailored_complete_model_graph::*;
use super::serde_json;
use std::sync::{Arc};
use std::time::Instant;
use super::blossom_v;
use super::union_find::UnionFind;
use super::types::*;
use std::collections::{BTreeSet};

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
    /// save configuration for later usage
    pub config: TailoredMWPMDecoderConfig,
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

pub mod tailored_mwpm_default_configs {

}

impl TailoredMWPMDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: &ErrorModel, decoder_configuration: &serde_json::Value) -> Self {
        // read attribute of decoder configuration
        let config: TailoredMWPMDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
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
        let mwpm_decoder = MWPMDecoder::new(&simulator, error_model, &json!({
            "precompute_complete_model_graph": config.precompute_complete_model_graph,
            "weight_function": config.weight_function,
        }));
        Self {
            tailored_model_graph: Arc::new(tailored_model_graph),
            tailored_complete_model_graph: tailored_complete_model_graph,
            mwpm_decoder: mwpm_decoder,
            virtual_nodes: Arc::new(virtual_nodes),
            simulator: Arc::new(simulator),
            config: config,
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
                    if i < j {  // remove duplicate edges in undirected graph
                        tailored_weighted_edges.push((i, j, weight));
                        // println!{"positive edge {} {} {} ", tailored_to_be_matched[i], tailored_to_be_matched[j], weight};
                    }
                }
                for &(j, weight) in negative_edges.iter() {
                    if i < j {  // remove duplicate edges in undirected graph
                        tailored_weighted_edges.push((tailored_len + i, tailored_len + j, weight));
                        // println!{"negative edge {} {} {} ", tailored_to_be_matched[i], tailored_to_be_matched[j], weight};
                    }
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
            // create clusters
            let mut tailored_cluster_roots: BTreeSet<usize> = BTreeSet::new();
            for i in 0..tailored_len {
                // filtering out positions matched with itself
                if tailored_clusters.get(i).set_size > 1 {
                    let root_i = tailored_clusters.find(i);
                    tailored_cluster_roots.insert(root_i);
                }
            }
            // eprintln!("tailored_cluster_roots: {:?}", tailored_cluster_roots);
            // do neutral decoding, only consider neutral clusters
            for &root_i in tailored_cluster_roots.iter() {
                if tailored_clusters.get(root_i).cardinality % 2 == 0 {
                    let begin = Instant::now();
                    let mut neutral_cluster = Vec::new();  // neutral cluster is a cycle of even number of StabY and even number of StabX
                    let root_i_positive = root_i % tailored_len;
                    let mut negative_2 = root_i_positive;  // make sure this enters loop at least once
                    while negative_2 != root_i_positive + tailored_len {
                        let positive_1 = negative_2 % tailored_len;
                        let positive_2 = tailored_matching[positive_1];
                        let negative_1 = positive_2 + tailored_len;
                        negative_2 = tailored_matching[negative_1];
                        neutral_cluster.push(positive_1);
                        neutral_cluster.push(positive_2);
                        // eprintln!("{} {} {}", positive_1, positive_2, negative_2);
                    }
                    debug_assert!({  // sanity check: indeed even number of StabY and even number of StabX
                        let mut stab_y_count = 0;
                        let mut stab_x_count = 0;
                        for &i in neutral_cluster.iter() {
                            let position = &tailored_to_be_matched[i];
                            let node = self.simulator.get_node_unwrap(position);
                            if node.qubit_type == QubitType::StabY {
                                stab_y_count += 1;
                            }
                            if node.qubit_type == QubitType::StabX {
                                stab_x_count += 1;
                            }
                        }
                        stab_y_count % 2 == 0 && stab_x_count % 2 == 0
                    });
                    // eprintln!("neutral_cluster: {:?}", neutral_cluster);
                    time_neutral_prepare_graph += begin.elapsed().as_secs_f64();
                    // build correction directly
                    let begin = Instant::now();
                    let mut last_y = None;
                    let mut last_x = None;
                    for &i in neutral_cluster.iter() {
                        let position = &tailored_to_be_matched[i];
                        let node = self.simulator.get_node_unwrap(position);
                        if node.qubit_type == QubitType::StabY {
                            if last_y.is_none() {
                                last_y = Some(position.clone());
                            } else {
                                let matching_correction = self.tailored_complete_model_graph.build_correction_neutral_matching(last_y.as_ref().unwrap(), position, &self.tailored_model_graph);
                                correction.extend(&matching_correction);
                                last_y = None;
                            }
                        }
                        if node.qubit_type == QubitType::StabX {
                            if last_x.is_none() {
                                last_x = Some(position.clone());
                            } else {
                                let matching_correction = self.tailored_complete_model_graph.build_correction_neutral_matching(last_x.as_ref().unwrap(), position, &self.tailored_model_graph);
                                correction.extend(&matching_correction);
                                last_x = None;
                            }
                        }
                    }
                    time_build_correction += begin.elapsed().as_secs_f64();
                }
            }
            // do residual decoding, instead of using the confusing method in the paper, I just match them together using normal graph
            let begin = Instant::now();
            let mut residual_to_be_matched = Vec::new();
            for i in 0..tailored_len {
                // filtering out positions matched with itself
                if tailored_clusters.get(i).set_size > 1 {
                    // only care about neutral clusters
                    // eprintln!("cluster {}: cardinality: {}", i, tailored_clusters.get(i).cardinality);
                    if tailored_clusters.get(i).cardinality % 2 == 1 {
                        // residual must be real node
                        let position = tailored_to_be_matched[i].clone();
                        let node = self.simulator.get_node_unwrap(&position);
                        if !node.is_virtual {
                            residual_to_be_matched.push(position);
                        }
                    }
                }
            }
            // eprintln!("residual_to_be_matched: {:?}", residual_to_be_matched);
            let residual_correction = if residual_to_be_matched.len() > 0 {
                let (correction, _) = self.mwpm_decoder.decode(&SparseMeasurement::from_vec(&residual_to_be_matched));
                correction
            } else {
                SparseCorrection::new()
            };
            time_residual_decoding += begin.elapsed().as_secs_f64();
            let begin = Instant::now();
            correction.extend(&residual_correction);
            time_build_correction += begin.elapsed().as_secs_f64();
        }
        (correction, json!({
            "to_be_matched": to_be_matched.len(),
            "time_tailored_prepare_graph": time_tailored_prepare_graph,
            "time_tailored_blossom_v": time_tailored_blossom_v,
            "time_tailored_union": time_tailored_union,
            "time_neutral_prepare_graph": time_neutral_prepare_graph,
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
        if false {  // debug 5: no edges in residual graph
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
        if false {  // debug 4
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
        if false {  // debug 3
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
        if false {  // debug 2
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
        if false {  // debug 1
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

    #[test]
    fn tailored_mwpm_decoder_deadlock_1() {  // cargo test tailored_mwpm_decoder_deadlock_1 -- --nocapture
        let d = 11;
        let noisy_measurements = 0;  // perfect measurement
        let p = 1.99053585e-01;
        let bias_eta = 1e200;
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
        // let error_pattern: SparseErrorPattern = serde_json::from_str(r#"{"[0][10][13]":"Z","[0][10][7]":"Z","[0][10][8]":"Z","[0][11][11]":"Z","[0][11][1]":"Z","[0][11][5]":"Z","[0][11][7]":"Z","[0][11][9]":"Z","[0][12][12]":"Z","[0][12][14]":"Z","[0][12][5]":"Z","[0][13][20]":"Z","[0][14][11]":"Z","[0][14][12]":"Z","[0][14][14]":"Z","[0][14][17]":"Z","[0][15][10]":"Z","[0][15][14]":"Z","[0][15][15]":"Z","[0][15][7]":"Z","[0][16][16]":"Z","[0][16][5]":"Z","[0][17][11]":"Z","[0][17][14]":"Z","[0][17][15]":"Z","[0][18][11]":"Z","[0][18][8]":"Z","[0][19][10]":"Z","[0][19][12]":"Z","[0][4][8]":"Z","[0][5][12]":"Z","[0][5][13]":"Z","[0][5][14]":"Z","[0][6][13]":"Z","[0][6][14]":"Z","[0][6][6]":"Z","[0][6][8]":"Z","[0][6][9]":"Z","[0][7][11]":"Z","[0][7][15]":"Z","[0][8][15]":"Z","[0][8][17]":"Z","[0][8][6]":"Z","[0][8][7]":"Z","[0][9][12]":"Z","[0][9][15]":"Z","[0][9][16]":"Z","[0][9][17]":"Z","[0][9][18]":"Z","[0][9][2]":"Z","[0][9][3]":"Z","[0][9][5]":"Z","[0][9][6]":"Z"}"#).unwrap();
        let error_pattern: SparseErrorPattern = serde_json::from_str(r#"{"[0][10][17]":"Z","[0][10][9]":"Z","[0][11][13]":"Z","[0][11][15]":"Z","[0][11][16]":"Z","[0][11][3]":"Z","[0][12][12]":"Z","[0][12][16]":"Z","[0][12][18]":"Z","[0][12][9]":"Z","[0][13][10]":"Z","[0][13][11]":"Z","[0][13][20]":"Z","[0][13][7]":"Z","[0][13][9]":"Z","[0][14][10]":"Z","[0][14][14]":"Z","[0][14][18]":"Z","[0][14][4]":"Z","[0][14][6]":"Z","[0][14][7]":"Z","[0][15][10]":"Z","[0][15][11]":"Z","[0][15][15]":"Z","[0][15][18]":"Z","[0][15][6]":"Z","[0][16][10]":"Z","[0][16][14]":"Z","[0][16][15]":"Z","[0][17][11]":"Z","[0][17][14]":"Z","[0][17][8]":"Z","[0][17][9]":"Z","[0][19][10]":"Z","[0][1][10]":"Z","[0][1][11]":"Z","[0][20][12]":"Z","[0][21][11]":"Z","[0][21][12]":"Z","[0][4][10]":"Z","[0][5][12]":"Z","[0][6][12]":"Z","[0][6][15]":"Z","[0][7][15]":"Z","[0][7][4]":"Z","[0][7][9]":"Z","[0][8][11]":"Z","[0][8][12]":"Z","[0][8][15]":"Z","[0][9][10]":"Z","[0][9][14]":"Z","[0][9][15]":"Z","[0][9][16]":"Z"}"#).unwrap();
        // println!("{:?}", error_pattern);
        simulator.load_sparse_error_pattern(&error_pattern).unwrap();
        simulator.propagate_errors();
        let sparse_measurement = simulator.generate_sparse_measurement();
        let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
        // println!("{:?}", correction);
        code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        let (logical_i, logical_j) = simulator.validate_correction(&correction);
        assert!(!logical_i && !logical_j);
    }

    #[test]
    fn tailored_mwpm_decoder_periodic_code_capacity_d5() {  // cargo test tailored_mwpm_decoder_periodic_code_capacity_d5 -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.005;
        let bias_eta = 1e6;
        // build simulator
        let mut simulator = Simulator::new(CodeType::PeriodicRotatedTailoredCode{ noisy_measurements, dp: d+1, dn: d+1 });
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
        {
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 7, 7)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error(&error_model, &Z);
            // simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error(&error_model, &Z);
            // simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
    }

    #[test]
    fn tailored_mwpm_decoder_periodic_code_capacity_d7() {  // cargo test tailored_mwpm_decoder_periodic_code_capacity_d7 -- --nocapture
        let d = 7;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.001;
        let bias_eta = 1e200;
        // build simulator
        let mut simulator = Simulator::new(CodeType::PeriodicRotatedTailoredCode{ noisy_measurements, dp: d+1, dn: d+1 });
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
        {  // debug: why 2 Z errors can cause logical error?
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 0)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
    }

}
