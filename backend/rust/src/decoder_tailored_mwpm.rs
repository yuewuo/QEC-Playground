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
use super::union_find::DefaultUnionFind;
use super::types::*;
use std::collections::{BTreeSet, BTreeMap};

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
    /// corner virtual nodes, required for residual decoding
    pub corner_virtual_nodes: Arc<Vec<(Position, Position)>>,
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
    /// whether use naive residual decoding, it seems the complex residual decoding doesn't help much...
    #[serde(alias = "nrd")]  // abbreviation
    #[serde(default = "tailored_mwpm_default_configs::naive_residual_decoding")]
    pub naive_residual_decoding: bool,
    /// disable residual decoding to test correctness under infinite bias
    #[serde(default = "tailored_mwpm_default_configs::disable_residual_decoding")]
    pub disable_residual_decoding: bool,
    /// whether use the original residual decoding weighting in https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.124.130501
    #[serde(default = "tailored_mwpm_default_configs::original_residual_weighting")]
    pub original_residual_weighting: bool,
    /// whether use the original residual decoding weighting of corner clusters: use the Manhattan distance
    #[serde(default = "tailored_mwpm_default_configs::original_residual_corner_weights")]
    pub original_residual_corner_weights: bool,
}

pub mod tailored_mwpm_default_configs {
    pub fn naive_residual_decoding() -> bool { false }
    pub fn disable_residual_decoding() -> bool { false }
    pub fn original_residual_weighting() -> bool { false }
    pub fn original_residual_corner_weights() -> bool { false }
}

impl TailoredMWPMDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: Arc<ErrorModel>, decoder_configuration: &serde_json::Value, parallel: usize, use_brief_edge: bool) -> Self {
        // read attribute of decoder configuration
        let config: TailoredMWPMDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
        tailored_model_graph.build(&mut simulator, error_model.as_ref(), &config.weight_function);
        let tailored_model_graph = Arc::new(tailored_model_graph);
        // build complete model graph
        let mut tailored_complete_model_graph = TailoredCompleteModelGraph::new(&simulator, Arc::clone(&tailored_model_graph));
        tailored_complete_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel);
        // build virtual nodes for decoding use
        let mut virtual_nodes = Vec::new();
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if tailored_model_graph.is_node_exist(position) {
            let node = simulator.get_node_unwrap(position);
            if node.is_virtual {
                virtual_nodes.push(position.clone());
            }
        });
        // build corner virtual nodes for residual decoding, see https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.124.130501
        // corner virtual nodes, in my understanding, is those having connection with only one real node
        let mut corner_virtual_nodes = Vec::<(Position, Position)>::new();
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if tailored_model_graph.is_node_exist(position) {
            let node = simulator.get_node_unwrap(position);
            if node.is_virtual {
                if let Some(miscellaneous) = &node.miscellaneous {
                    if miscellaneous.get("is_corner") == Some(&json!(true)) {
                        let peer_corner = miscellaneous.get("peer_corner").expect("corner must appear in pair, either in standard or rotated tailored surface code");
                        let peer_position = serde_json::from_value(peer_corner.clone()).unwrap();
                        corner_virtual_nodes.push((position.clone(), peer_position));
                    }
                }
            }
        });
        // println!("corner_virtual_nodes: {:?}", corner_virtual_nodes);
        // build MWPM decoder
        let mwpm_decoder = MWPMDecoder::new(&simulator, error_model, &json!({
            "precompute_complete_model_graph": config.precompute_complete_model_graph,
            "weight_function": config.weight_function,
        }), parallel, use_brief_edge);
        Self {
            tailored_model_graph: tailored_model_graph,
            tailored_complete_model_graph: tailored_complete_model_graph,
            mwpm_decoder: mwpm_decoder,
            virtual_nodes: Arc::new(virtual_nodes),
            corner_virtual_nodes: Arc::new(corner_virtual_nodes),
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
            // eprintln!("tailored_to_be_matched: {:?}", tailored_to_be_matched);
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
            let mut tailored_clusters = DefaultUnionFind::new(tailored_len);
            for i in 0..tailored_len {  // set `cardinality` to 1 if the position is a StabY
                let position = &tailored_to_be_matched[i];
                let node = self.simulator.get_node_unwrap(position);
                if node.qubit_type == QubitType::StabY {
                    tailored_clusters.payload[i].cardinality = 1;
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
            let begin = Instant::now();
            let mut all_clusters = BTreeMap::<usize, Vec<usize>>::new();  // both neutral and charged clusters
            let mut charged_cluster_count = 0;
            for &root_i in tailored_cluster_roots.iter() {
                let mut cluster = Vec::new();
                let root_i_positive = root_i % tailored_len;
                let mut negative_2 = root_i_positive;  // make sure this enters loop at least once
                while negative_2 != root_i_positive + tailored_len {
                    let positive_1 = negative_2 % tailored_len;
                    let positive_2 = tailored_matching[positive_1];
                    let negative_1 = positive_2 + tailored_len;
                    negative_2 = tailored_matching[negative_1];
                    cluster.push(positive_1);
                    cluster.push(positive_2);
                    // eprintln!("{} {} {}", positive_1, positive_2, negative_2);
                }
                let is_neutral = tailored_clusters.get(root_i).cardinality % 2 == 0;
                // eprintln!("root_i: {}, cardinality: {}", root_i, tailored_clusters.get(root_i).cardinality);
                debug_assert!({  // sanity check: indeed even number of StabY and even number of StabX for neutral cluster, otherwise all odd
                    let mut stab_y_count = 0;
                    let mut stab_x_count = 0;
                    for &i in cluster.iter() {
                        let position = &tailored_to_be_matched[i];
                        let node = self.simulator.get_node_unwrap(position);
                        if node.qubit_type == QubitType::StabY {
                            stab_y_count += 1;
                        }
                        if node.qubit_type == QubitType::StabX {
                            stab_x_count += 1;
                        }
                    }
                    if is_neutral {
                        stab_y_count % 2 == 0 && stab_x_count % 2 == 0
                    } else {
                        stab_y_count % 2 == 1 && stab_x_count % 2 == 1
                    }
                });
                if is_neutral {
                    let neutral_cluster = cluster;
                    // eprintln!("neutral_cluster: {:?}", neutral_cluster);
                    all_clusters.insert(root_i, neutral_cluster);
                } else {
                    let charged_cluster = cluster;
                    // eprintln!("charged_cluster: {:?}", charged_cluster);
                    all_clusters.insert(root_i, charged_cluster);
                    charged_cluster_count += 1;
                }
            }
            time_neutral_prepare_graph += begin.elapsed().as_secs_f64();
            for &root_i in tailored_cluster_roots.iter() {
                if tailored_clusters.get(root_i).cardinality % 2 == 0 {
                    let neutral_cluster = &all_clusters[&root_i];
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
                                let matching_correction = self.tailored_complete_model_graph.build_correction_neutral_matching(last_y.as_ref().unwrap(), position);
                                correction.extend(&matching_correction);
                                last_y = None;
                            }
                        }
                        if node.qubit_type == QubitType::StabX {
                            if last_x.is_none() {
                                last_x = Some(position.clone());
                            } else {
                                let matching_correction = self.tailored_complete_model_graph.build_correction_neutral_matching(last_x.as_ref().unwrap(), position);
                                correction.extend(&matching_correction);
                                last_x = None;
                            }
                        }
                    }
                    time_build_correction += begin.elapsed().as_secs_f64();
                }
            }
            let begin = Instant::now();
            let residual_correction = if self.config.disable_residual_decoding {
                SparseCorrection::new()
            } else if self.config.naive_residual_decoding {
                // do naive residual decoding, instead of using the confusing method in the paper, I just match them together using normal graph
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
                if residual_to_be_matched.len() > 0 {
                    let (correction, _) = self.mwpm_decoder.decode(&SparseMeasurement::from_vec(&residual_to_be_matched));
                    correction
                } else {
                    SparseCorrection::new()
                }
            } else {  // reproduce the exact residual-decoding in https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.124.130501
                let mut correction = SparseCorrection::new();
                if charged_cluster_count > 0 {
                    let mut residual_to_be_matched_cluster_root = Vec::<usize>::new();
                    let mut residual_index_peer_neutral_copied = BTreeMap::<usize, usize>::new();
                    let mut residual_weighted_edges = Vec::<(usize, usize, f64)>::new();
                    let mut residual_roots = Vec::new();
                    for &root_i in tailored_cluster_roots.iter() {
                        let is_neutral = tailored_clusters.get(root_i).cardinality % 2 == 0;
                        if is_neutral {
                            let neutral_cluster = &all_clusters[&root_i];
                            let mut has_stab_y = false;
                            let mut has_stab_x = false;
                            for &i in neutral_cluster.iter() {
                                let position = &tailored_to_be_matched[i];
                                let node = self.simulator.get_node_unwrap(position);
                                if node.qubit_type == QubitType::StabY {
                                    has_stab_y = true;
                                }
                                if node.qubit_type == QubitType::StabX {
                                    has_stab_x = true;
                                }
                            }
                            // cluster is neutral and contains X-type and Y-type defects
                            if has_stab_y && has_stab_x {
                                // add two copies of cluster to graph connected by a zero weight edge
                                let first_index = residual_to_be_matched_cluster_root.len();
                                let second_index = first_index + 1;
                                residual_weighted_edges.push((first_index, second_index, 0.));
                                // for them to find peer quickly
                                residual_index_peer_neutral_copied.insert(first_index, second_index);
                                residual_index_peer_neutral_copied.insert(second_index, first_index);
                                residual_to_be_matched_cluster_root.push(root_i);
                                residual_to_be_matched_cluster_root.push(root_i);
                                residual_roots.push(root_i);
                            }
                        } else {  // charged, add cluster to graph
                            residual_to_be_matched_cluster_root.push(root_i);
                            residual_roots.push(root_i);
                        }
                    }
                    let residual_real_cluster_len = residual_to_be_matched_cluster_root.len();
                    // foreach corner vertex at each time step where no stabilizer is applied, add virtual cluster to graph
                    for ci in 0..self.corner_virtual_nodes.len() {
                        residual_to_be_matched_cluster_root.push(ci);
                    }
                    // foreach cluster pair in graph do
                    let get_cluster_positions = |index: usize| -> Vec<Position> {
                        if index < residual_real_cluster_len {  // a real cluster
                            let root_i = residual_to_be_matched_cluster_root[index];
                            let cluster = &all_clusters[&root_i];  // can be either neutral or charged, doesn't matter
                            cluster.iter().map(|i| tailored_to_be_matched[*i].clone()).collect()
                        } else {
                            let ci = residual_to_be_matched_cluster_root[index];
                            let (pos1, pos2) = self.corner_virtual_nodes[ci].clone();
                            vec![pos1, pos2]
                        }
                    };
                    for i in 0..residual_to_be_matched_cluster_root.len() {
                        if !self.config.original_residual_corner_weights && i >= residual_real_cluster_len {
                            continue
                        }
                        let cluster_positions_i = get_cluster_positions(i);
                        for j in i+1..residual_to_be_matched_cluster_root.len() {
                            if i < residual_real_cluster_len && j < residual_real_cluster_len && residual_to_be_matched_cluster_root[i] == residual_to_be_matched_cluster_root[j] {
                                // already added zero weight, skip
                                continue
                            }
                            let cluster_positions_j = get_cluster_positions(j);
                            // eprintln!("cluster_positions_[{}]: {:?}, cluster_positions_[{}]: {:?}", i, cluster_positions_i, j, cluster_positions_j);
                            // add edge to graph weighted by minimum Manhattan distance between any pairing of defects drawn one from each cluster
                            let mut stab_x_min_weight = f64::MAX;
                            let mut stab_y_min_weight = f64::MAX;
                            for pi in cluster_positions_i.iter() {
                                let is_stab_x = self.simulator.get_node_unwrap(pi).qubit_type == QubitType::StabX;
                                let neutral_matching_edges = self.tailored_complete_model_graph.get_neutral_matching_edges(pi, &cluster_positions_j);
                                for (_wi, weight) in neutral_matching_edges.iter() {
                                    // eprintln!("edge between {:?} and {:?}: weight = {}", pi, cluster_positions_j[*_wi], weight);
                                    if is_stab_x {
                                        if *weight < stab_x_min_weight {
                                            stab_x_min_weight = *weight;
                                        }
                                    } else {
                                        if *weight < stab_y_min_weight {
                                            stab_y_min_weight = *weight;
                                        }
                                    }
                                }
                            }
                            assert!(stab_x_min_weight != f64::MAX, "there should be at least one neutral edge between two clusters we're considering");
                            assert!(stab_y_min_weight != f64::MAX, "there should be at least one neutral edge between two clusters we're considering");
                            // take the bigger one as the final weight (this should benefit)
                            let min_weight = if self.config.original_residual_weighting {
                                // "weighted by minimum Manhattan distance between any pairing of defects drawn one from each cluster;
                                if stab_x_min_weight < stab_y_min_weight {
                                    stab_x_min_weight
                                } else {
                                    stab_y_min_weight
                                }
                            } else {
                                stab_x_min_weight + stab_y_min_weight
                            };
                            // let min_weight = if stab_x_min_weight < stab_y_min_weight { stab_x_min_weight } else { stab_y_min_weight };
                            // let min_weight = if stab_x_min_weight > stab_y_min_weight { stab_x_min_weight } else { stab_y_min_weight };
                            residual_weighted_edges.push((i, j, min_weight));
                        }
                    }
                    // those corner clusters should be connected with zero weight: this is not in the paper but otherwise they'll mess up the weights
                    if !self.config.original_residual_corner_weights {  // edges already added if enable original residual corner weights
                        for i in 0..self.corner_virtual_nodes.len() {
                            for j in i+1..self.corner_virtual_nodes.len() {
                                residual_weighted_edges.push((residual_real_cluster_len + i, residual_real_cluster_len + j, 0.));
                            }
                        }
                    }
                    // if odd number of charged clusters
                    if residual_to_be_matched_cluster_root.len() % 2 == 1 {
                        // add virtual cluster to graph with zero weight edge to each virtual cluster
                        let additional_virtual_index = residual_to_be_matched_cluster_root.len();
                        residual_to_be_matched_cluster_root.push(usize::MAX);
                        for ci in 0..self.corner_virtual_nodes.len() {
                            residual_weighted_edges.push((residual_real_cluster_len + ci, additional_virtual_index, 0.));
                        }
                    }
                    // eprintln!("residual_weighted_edges: {:?}", residual_weighted_edges);
                    let residual_matching = blossom_v::safe_minimum_weight_perfect_matching(residual_to_be_matched_cluster_root.len(), residual_weighted_edges);
                    // eprintln!("residual_matching: {:?}", residual_matching);
                    // foreach cluster pair in matching do
                    let mut neutralized_charged_cluster = BTreeSet::<usize>::new();  // index in `residual_matching`
                    let apply_correction_with_merged_cluster = |mut_self: &mut TailoredMWPMDecoder, correction: &mut SparseCorrection, cluster_1: Vec::<Position>, cluster_2: Vec::<Position>| {
                        let mut merged_to_be_matched = Vec::<Position>::with_capacity(cluster_1.len() + cluster_2.len());
                        merged_to_be_matched.extend(cluster_1.into_iter());
                        merged_to_be_matched.extend(cluster_2.into_iter());
                        let merged_to_be_matched = merged_to_be_matched;  // change to immutable
                        // eprintln!("merged_to_be_matched: {:?}", merged_to_be_matched);
                        // first split into stabX and stabY
                        let mut stab_x_positions = Vec::<Position>::new();
                        let mut stab_y_positions = Vec::<Position>::new();
                        for position in merged_to_be_matched.iter() {
                            let node = mut_self.simulator.get_node_unwrap(position);
                            if node.qubit_type == QubitType::StabX {
                                stab_x_positions.push(position.clone());
                            } else {
                                stab_y_positions.push(position.clone());
                            }
                        }
                        assert!(stab_x_positions.len() % 2 == 0, "merged cluster should be neutral");
                        assert!(stab_y_positions.len() % 2 == 0, "merged cluster should be neutral");
                        for positions in [stab_x_positions, stab_y_positions].iter() {
                            for i in (0..positions.len()).step_by(2) {
                                let position_1 = &positions[i];
                                let position_2 = &positions[i + 1];
                                let matching_correction = mut_self.tailored_complete_model_graph.build_correction_neutral_matching(position_1, position_2);
                                correction.extend(&matching_correction);
                            }
                        }
                    };
                    for i in 0..residual_real_cluster_len {  // remove matches where each cluster is virtual
                        let root_i = residual_to_be_matched_cluster_root[i];
                        let was_neutral_i = tailored_clusters.get(root_i).cardinality % 2 == 0;
                        if was_neutral_i {
                            continue  // do not start at neutral node
                        }
                        // note: charged + charged = neutral, neutral + neutral = neutral, they can be handled locally
                        //       charged + neutral = charged, which must be handled with extra effort; I'll neutralize them in a chain if this happens
                        // always pick up a charged cluster, and add it to `neutralized_charged_cluster`; chain the operation until all neutralized
                        if !neutralized_charged_cluster.contains(&i) {
                            neutralized_charged_cluster.insert(i);
                            let j = residual_matching[i];
                            let cluster_i = &all_clusters[&root_i];
                            if j < residual_real_cluster_len {
                                let root_j = residual_to_be_matched_cluster_root[j];
                                let was_neutral_j = tailored_clusters.get(root_j).cardinality % 2 == 0;
                                if !was_neutral_j {  // good, we can stop here
                                    assert!(!neutralized_charged_cluster.contains(&j), "peer cluster should not already been neutralized");
                                    neutralized_charged_cluster.insert(j);
                                    // eprintln!("residual match {} to charged cluster {}", i, j);
                                    let cluster_j = &all_clusters[&root_j];
                                    // merge them into a single one cluster
                                    apply_correction_with_merged_cluster(self, &mut correction, cluster_i.iter().map(|i| tailored_to_be_matched[*i].clone()).collect()
                                        , cluster_j.iter().map(|i| tailored_to_be_matched[*i].clone()).collect());
                                } else {  // have to do it in a chain... until peer becomes virtual or charged
                                    let mut charged_i: Vec<usize> = all_clusters[&root_i].clone();  // index of `tailored_to_be_matched`
                                    let mut j1 = j;
                                    loop {  // must enter with j1 as neutral
                                        let root_j1 = residual_to_be_matched_cluster_root[j1];
                                        let cluster_j1 = &all_clusters[&root_j1];
                                        // kick out one StabX and one StabY out of this neutral cluster
                                        let mut charged_j = Vec::<usize>::new();
                                        for delete_type in [QubitType::StabX, QubitType::StabY] {
                                            for idx in 0..cluster_j1.len() {
                                                let position_i = &tailored_to_be_matched[cluster_j1[idx]];
                                                let node_i = self.simulator.get_node_unwrap(position_i);
                                                if node_i.qubit_type == delete_type {
                                                    charged_j.push(cluster_j1[idx]);
                                                    break
                                                }
                                            }
                                        }
                                        assert!(charged_j.len() == 2, "neutral clusters here must contain at least one X and one Y");
                                        // add shortest path of Y (X) operators to recovery between selected X-type (Y-type) defects from each cluster in pair
                                        apply_correction_with_merged_cluster(self, &mut correction, charged_i.iter().map(|i| tailored_to_be_matched[*i].clone()).collect()
                                            , charged_j.iter().map(|i| tailored_to_be_matched[*i].clone()).collect());
                                        // break if k is no longer neutral cluster, otherwise keep loop
                                        let j2 = residual_index_peer_neutral_copied[&j1];
                                        let k = residual_matching[j2];
                                        if k >= residual_real_cluster_len {
                                            // match `charged_j` with this virtual cluster
                                            let ck = k - residual_real_cluster_len;
                                            let (pos1, pos2) = self.corner_virtual_nodes[ck].clone();
                                            apply_correction_with_merged_cluster(self, &mut correction, charged_j.iter().map(|i| tailored_to_be_matched[*i].clone()).collect()
                                                , vec![pos1, pos2]);
                                            break
                                        }
                                        let root_k = residual_to_be_matched_cluster_root[k];
                                        let was_neutral_k = tailored_clusters.get(root_k).cardinality % 2 == 0;
                                        if !was_neutral_k {
                                            assert!(!neutralized_charged_cluster.contains(&k), "peer cluster should not already been neutralized");
                                            neutralized_charged_cluster.insert(k);
                                            // match `charged_j` with the charged cluster `root_k`
                                            let cluster_k = &all_clusters[&root_k];
                                            apply_correction_with_merged_cluster(self, &mut correction, charged_j.iter().map(|i| tailored_to_be_matched[*i].clone()).collect()
                                                , cluster_k.iter().map(|i| tailored_to_be_matched[*i].clone()).collect());
                                            break
                                        }
                                        charged_i = charged_j;
                                        j1 = k;
                                    }
                                }
                            } else {  // good, match to virtual node
                                // eprintln!("residual match {} to virtual {}", i, j);
                                let cj = j - residual_real_cluster_len;
                                let (pos1, pos2) = self.corner_virtual_nodes[cj].clone();
                                apply_correction_with_merged_cluster(self, &mut correction, cluster_i.iter().map(|i| tailored_to_be_matched[*i].clone()).collect()
                                    , vec![pos1, pos2]);
                            }
                        }
                    }
                }
                correction
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


#[cfg(feature = "blossom_v")]
#[cfg(test)]
mod tests {
    use super::*;
    use super::super::code_builder::*;
    use super::super::types::ErrorType::*;

    #[test]
    fn tailored_mwpm_decoder_code_capacity_inf_bias_d_3() {  // cargo test tailored_mwpm_decoder_code_capacity_inf_bias_d_3 -- --nocapture
        let d = 3;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.02;
        let bias_eta = 1e200;
        // build simulator
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let enable_all = true;
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        if true || enable_all {  // debug 11: why cannot code distance 3 correct only 3 Z errors?
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 1, 3)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 3, 1)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 3, 5)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            // assert!(!logical_i && !logical_j);
            assert!(logical_i && logical_j);  // .... surprisingly, it's supposed to have a logical error on both axis
        }
    }

    #[test]
    fn tailored_mwpm_decoder_code_capacity() {  // cargo test tailored_mwpm_decoder_code_capacity -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.005;
        let bias_eta = 1e6;
        // build simulator
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let enable_all = true;
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        if false || enable_all {  // debug 7: residual decoding
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 7, 5)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error_check(&error_model, &Y);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 5: no edges in residual graph
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 1, 5)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 2, 4)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 4
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 4)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 7, 3)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 4)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 3
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 1, 5)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 2, 6)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 8)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 2.5
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 7, 7)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 2
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 8)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 6)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            assert_eq!(sparse_measurement.to_vec(), vec![pos!(6, 5, 6), pos!(6, 5, 8), pos!(6, 6, 5), pos!(6, 7, 8), pos!(6, 8, 5), pos!(6, 9, 6)]);
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 1
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 3)).set_error_check(&error_model, &Z);
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
    fn tailored_mwpm_decoder_code_capacity_residual() {  // cargo test tailored_mwpm_decoder_code_capacity_residual -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.01;
        let bias_eta = 10.;
        // build simulator
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        {  // debug 9: failing case: {"[0][7][5]":"Y","[0][8][4]":"Z"}
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 7, 5)).set_error_check(&error_model, &Y);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 4)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            println!("sparse_measurement: {:?}", sparse_measurement);
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false {  // debug 8: residual decoding with charged node
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 6, 4)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 4, 6)).set_error_check(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            println!("sparse_measurement: {:?}", sparse_measurement);
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
    }


    #[test]
    fn tailored_mwpm_decoder_code_capacity_residual_2() {  // cargo test tailored_mwpm_decoder_code_capacity_residual_2 -- --nocapture
        let d = 7;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.05;
        let bias_eta = 10.;
        // build simulator
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        {  // debug 10: infinite loop case
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 6, 4)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 8)).set_error_check(&error_model, &Y);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 6)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 10, 8)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 11, 9)).set_error_check(&error_model, &Y);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            println!("sparse_measurement: {:?}", sparse_measurement);
            let (correction, _runtime_statistics) = tailored_mwpm_decoder.decode(&sparse_measurement);
            println!("{:?}", correction);
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
        let mut simulator = Simulator::new(CodeType::RotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, d, d));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
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
        let mut simulator = Simulator::new(CodeType::PeriodicRotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, d+1, d+1));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        {
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 7, 7)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error_check(&error_model, &Z);
            // simulator.get_node_mut_unwrap(&pos!(0, 5, 5)).set_error_check(&error_model, &Z);
            // simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error_check(&error_model, &Z);
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
        let mut simulator = Simulator::new(CodeType::PeriodicRotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, d+1, d+1));
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        let error_model = Arc::new(error_model);
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let mut tailored_mwpm_decoder = TailoredMWPMDecoder::new(&Arc::new(simulator.clone()), Arc::clone(&error_model), &decoder_config, 1, false);
        {  // debug: why 2 Z errors can cause logical error?
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 4, 4)).set_error_check(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 0)).set_error_check(&error_model, &Z);
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
