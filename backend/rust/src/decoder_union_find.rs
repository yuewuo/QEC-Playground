//! union-find decoder (weighted)
//! 

use serde::{Serialize, Deserialize};
use super::simulator::*;
use super::error_model::*;
use super::model_graph::*;
use super::complete_model_graph::*;
use super::serde_json;
use super::decoder_mwpm::*;
use super::union_find::*;
use std::sync::{Arc};
use std::time::Instant;
use std::collections::{HashMap, BTreeMap};
use super::either::Either;

/// MWPM decoder, initialized and cloned for multiple threads
#[derive(Debug, Clone, Serialize)]
pub struct UnionFindDecoder {
    /// model graph is immutably shared, just in case need to print out real weights instead of scaled and truncated weights
    pub model_graph: Arc<ModelGraph>,
    /// complete model graph each thread maintain its own precomputed data
    pub complete_model_graph: CompleteModelGraph,
    /// index to position mapping (immutable shared), index is the one used in the union-find algorithm
    pub index_to_position: Arc<Vec<Position>>,
    /// position to index mapping (immutable shared)
    pub position_to_index: Arc<HashMap<Position, usize>>,
    /// decoder nodes, each corresponds to a node in the model graph; each instance needs to modify node information and thus not shared
    pub nodes: Vec<UnionFindDecoderNode>,
    /// union-find algorithm
    pub union_find: UnionFind,
    /// recording the list of odd clusters to reduce iteration complexity
    pub odd_clusters: Vec<usize>,
    /// another odd_cluster to avoid memory allocation
    idle_odd_clusters: Vec<usize>,
    /// record the boundary nodes as an optimization, see <https://arxiv.org/pdf/1709.06218.pdf> Section "Boundary representation".
    /// even clusters should not be key in HashMap, and only real boundary should be in the `HashSet` value;
    /// those nodes without error syndrome also have entries in this HashMap, with the value of { itself }
    pub cluster_boundaries: Vec<Vec<usize>>,
    /// another cluster_boundaries to avoid memory allocation
    idle_cluster_boundaries: Vec<Vec<usize>>,
    /// trace: study the time consumption of each step
    pub time_uf_grow_step: f64,
    pub time_uf_grow: f64,
    pub count_uf_grow: usize,
    pub time_uf_merge: f64,
    pub time_uf_update: f64,
    pub time_uf_remove: f64,
    pub count_node_visited: usize,
    /// save configuration for later usage
    pub config: UnionFindDecoderConfig,
    /// internal cache used by iteration
    fusion_list: Vec<(usize, usize)>,
    /// internal variable that works like `odd_clusters_set: BTreeSet<usize>` but with constant performance
    odd_clusters_set_active_timestamp: usize,
    /// internal variable that works like `shrunk_boundaries: BTreeSet<usize>` but with constant performance
    shrunk_boundaries_active_timestamp: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnionFindDecoderNode {
    /// the index used in union-find algorithm, can be used to query position using [`UnionFindDecoder::index_to_position`]
    pub index: usize,
    /// whether this stabilizer has detected a error
    pub is_error_syndrome: bool,
    /// directly connected neighbors, (address, already increased length = 0, length = 0)
    pub neighbors: Vec<NeighborPartialEdge>,
    /// if this node has a direct path to boundary, then set to `Some(length)` given the integer length of matching to boundary, otherwise `None`.
    pub boundary_length: Option<usize>,
    /// increased region towards boundary, only valid when `node.boundary_length` is `Some(_)`
    pub boundary_increased: usize,
    /// whether visited ever in the algorithm, recorded as statistics
    pub node_visited: bool,
    /// internal variable that works like `odd_clusters_set: BTreeSet<usize>` but with constant performance
    odd_clusters_set_timestamp: usize,
    /// internal variable that works like `shrunk_boundaries: BTreeSet<usize>` but with constant performance
    shrunk_boundaries_timestamp: usize,
}

impl UnionFindDecoderNode {
    // the number of neighbors should be small
    fn index_to_neighbor(&self, neighbor: usize) -> Option<usize> {
        let mut result = None;
        for (index, partial_edge) in self.neighbors.iter().enumerate() {
            if partial_edge.neighbor == neighbor {
                result = Some(index);
                // return Some(index)  // comment out to ensure same runtime for each run
            }
        }
        result
    }
}

/// each edge consists of two partial edges, one from each vertex; the sum of their [`NeighborPartialEdge::increased`] is the growing progress of this edge
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeighborPartialEdge {
    /// the index of neighbor
    pub neighbor: usize,
    /// already increased length, initialized as 0. erasure should initialize as `length` (or any value at least `length`/2)
    pub increased: usize,
    /// the total length of this edge. if the sum of the `increased` of two partial edges is no less than `length`, then two vertices are merged
    pub length: usize,
    /// performance optimization by caching whether it's already grown
    pub grown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnionFindDecoderConfig {
    /// build complete model graph at first, but this will consume O(N^2) memory and increase initialization time,
    /// disable this when you're simulating large code
    #[serde(alias = "pcmg")]  // abbreviation
    #[serde(default = "mwpm_default_configs::precompute_complete_model_graph")]
    pub precompute_complete_model_graph: bool,
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// maximum weight will be 2 * max_half_weight, so that each time an edge can grow 1; by default is 1: unweighted union-find decoder
    #[serde(alias = "mhw")]  // abbreviation
    #[serde(default = "union_find_default_configs::max_half_weight")]
    pub max_half_weight: usize,
    /// real-weighted union-find decoder assuming weights are large integers, and try to handle them as real numbers (various growth step);
    /// by default is false: the original union-find decoder
    #[serde(alias = "urw")]  // abbreviation
    #[serde(default = "union_find_default_configs::use_real_weighted")]
    pub use_real_weighted: bool,
    /// skip building correction, only for benchmarking decoder speed; building correction can be very expensive in dense errors
    #[serde(alias = "bsbc")]  // abbreviation
    #[serde(default = "union_find_default_configs::benchmark_skip_building_correction")]
    pub benchmark_skip_building_correction: bool,
}

pub mod union_find_default_configs {
    pub fn max_half_weight() -> usize { 1 }
    pub fn use_real_weighted() -> bool { false }
    pub fn benchmark_skip_building_correction() -> bool { false }
}

impl UnionFindDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: &ErrorModel, decoder_configuration: &serde_json::Value, parallel: usize) -> Self {
        // read attribute of decoder configuration
        let config: UnionFindDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        if config.use_real_weighted {
            assert!(decoder_configuration.as_object().unwrap().contains_key("max_half_weight"), "`use_real_weighted` must come with `max_half_weight`; should be sufficiently large instead of the default 1");
        }
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_graph = ModelGraph::new(&simulator);
        model_graph.build(&mut simulator, &error_model, &config.weight_function);
        let model_graph = Arc::new(model_graph);
        // build complete model graph
        let mut complete_model_graph = CompleteModelGraph::new(&simulator, Arc::clone(&model_graph));
        complete_model_graph.optimize_weight_greater_than_sum_boundary = false;  // disable this optimization for any matching pair to exist
        complete_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel);
        // build union-find graph
        let mut index_to_position = Vec::<Position>::new();
        let mut position_to_index = HashMap::<Position, usize>::with_capacity(simulator.height * simulator.vertical * simulator.horizontal);
        let mut nodes = Vec::<UnionFindDecoderNode>::new();
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if model_graph.is_node_exist(position) {
            let index = nodes.len();
            let node = UnionFindDecoderNode {
                index: index,
                is_error_syndrome: false,
                neighbors: Vec::new(),  // updated later
                boundary_length: None,  // updated later
                boundary_increased: 0,
                node_visited: false,
                odd_clusters_set_timestamp: 0,
                shrunk_boundaries_timestamp: 0,
            };
            position_to_index.insert(position.clone(), index);
            index_to_position.push(position.clone());
            nodes.push(node);
        });
        // calculate scaling factor of edges
        let mut maximum_weight = 0.;
        for index in 0..nodes.len() {
            let position = index_to_position.get(index).unwrap();
            let model_graph_node = model_graph.get_node_unwrap(position);
            for (_peer_position, edge) in model_graph_node.edges.iter() {
                if edge.probability > 0. && edge.weight > maximum_weight {
                    maximum_weight = edge.weight;
                }
            }
            match &model_graph_node.boundary {
                Some(boundary) => {
                    if boundary.probability > 0. && boundary.weight > maximum_weight {
                        maximum_weight = boundary.weight;
                    }
                },
                None => { }
            }
        }
        let scale_weight = |weight: f64| -> usize {
            if maximum_weight == 0. {  // pure erasure channel could lead to this, all possible errors has weight = 0
                2 * config.max_half_weight
            } else {
                let mut half_weight = ((config.max_half_weight as f64) * weight / maximum_weight).round() as usize;
                if half_weight > config.max_half_weight {
                    half_weight = config.max_half_weight;
                }
                if half_weight < 1 {
                    half_weight = 1;
                }
                // println!("half_weight = {}, minimum_probability = {}, probability = {}", half_weight, minimum_probability, probability);
                2 * half_weight
            }
        };
        // fill in neighbors
        let mut cluster_boundaries = Vec::with_capacity(nodes.len());
        let mut idle_cluster_boundaries = Vec::with_capacity(nodes.len());
        for index in 0..nodes.len() {
            let position = index_to_position.get(index).unwrap();
            let node = nodes.get_mut(index).unwrap();
            let model_graph_node = model_graph.get_node_unwrap(position);
            for (peer_position, edge) in model_graph_node.edges.iter() {
                if edge.probability > 0. {
                    let peer_index = position_to_index[peer_position];
                    assert!(node.index_to_neighbor(peer_index).is_none(), "duplicate edge forbidden");
                    node.neighbors.push(NeighborPartialEdge {
                        neighbor: peer_index,
                        increased: 0,
                        length: scale_weight(edge.weight),
                        grown: false,
                    });
                }
            }
            match &model_graph_node.boundary {
                Some(boundary) => {
                    if boundary.probability > 0. {
                        node.boundary_length = Some(scale_weight(boundary.weight));
                    }
                },
                None => { }
            }
            cluster_boundaries.push(vec![index]);
            idle_cluster_boundaries.push(vec![]);
        }
        let union_find = UnionFind::new(nodes.len());
        Self {
            model_graph: Arc::clone(&model_graph),
            complete_model_graph: complete_model_graph,
            index_to_position: Arc::new(index_to_position),
            position_to_index: Arc::new(position_to_index),
            nodes: nodes,
            union_find: union_find,
            odd_clusters: Vec::new(),
            idle_odd_clusters: Vec::new(),
            cluster_boundaries: cluster_boundaries,  // Yue 2022.5.17: previously I use BTreeMap, but it has O(d^2.6) scaling rather than O(d^2)
            idle_cluster_boundaries: idle_cluster_boundaries,
            time_uf_grow_step: 0.,
            time_uf_grow: 0.,
            count_uf_grow: 0,
            time_uf_merge: 0.,
            time_uf_update: 0.,
            time_uf_remove: 0.,
            count_node_visited: 0,
            config: config,
            // internal caches
            fusion_list: Vec::new(),
            odd_clusters_set_active_timestamp: 0,
            shrunk_boundaries_active_timestamp: 0,
        }
    }

    /// clear odd_clusters_set
    pub fn clear_odd_clusters_set(&mut self) -> usize {
        if self.odd_clusters_set_active_timestamp == usize::MAX {  // rarely happens
            self.odd_clusters_set_active_timestamp = 0;
            for node in self.nodes.iter_mut() {
                node.odd_clusters_set_timestamp = 0;  // refresh all timestamps to avoid conflicts
            }
        }
        self.odd_clusters_set_active_timestamp += 1;  // implicitly invalidate all nodes
        self.odd_clusters_set_active_timestamp
    }

    /// has odd_clusters_set
    pub fn has_odd_clusters_set(&self, index: usize) -> bool {
        self.nodes[index].odd_clusters_set_timestamp == self.odd_clusters_set_active_timestamp
    }

    /// insert odd_clusters_set
    pub fn insert_odd_clusters_set(&mut self, index: usize) {
        self.nodes[index].odd_clusters_set_timestamp = self.odd_clusters_set_active_timestamp
    }

    pub fn clear_shrunk_boundaries_static(nodes: &mut Vec<UnionFindDecoderNode>, shrunk_boundaries_active_timestamp: &mut usize) -> usize {
        if *shrunk_boundaries_active_timestamp == usize::MAX {  // rarely happens
            *shrunk_boundaries_active_timestamp = 0;
            for node in nodes.iter_mut() {
                node.shrunk_boundaries_timestamp = 0;  // refresh all timestamps to avoid conflicts
            }
        }
        *shrunk_boundaries_active_timestamp += 1;  // implicitly invalidate all nodes
        *shrunk_boundaries_active_timestamp
    }

    /// clear shrunk_boundaries
    #[allow(dead_code)]
    pub fn clear_shrunk_boundaries(&mut self) -> usize {
        Self::clear_shrunk_boundaries_static(&mut self.nodes, &mut self.shrunk_boundaries_active_timestamp)
    }

    /// has shrunk_boundaries
    #[allow(dead_code)]
    pub fn has_shrunk_boundaries(&self, index: usize) -> bool {
        self.nodes[index].shrunk_boundaries_timestamp == self.shrunk_boundaries_active_timestamp
    }

    /// insert shrunk_boundaries
    #[allow(dead_code)]
    pub fn insert_shrunk_boundaries(&mut self, index: usize) {
        self.nodes[index].shrunk_boundaries_timestamp = self.shrunk_boundaries_active_timestamp
    }

    /// clear the state, must be called before trying to decode another syndrome
    pub fn clear(&mut self) {
        self.union_find.clear();
        for index in 0..self.nodes.len() {
            let node = self.nodes.get_mut(index).unwrap();
            node.is_error_syndrome = false;  // clean previous error syndrome
            for neighbor_partial_edge in node.neighbors.iter_mut() {
                neighbor_partial_edge.increased = 0;
                neighbor_partial_edge.grown = false;
            }
            node.boundary_increased = 0;
            node.node_visited = false;
            // overwrite the odd value
            self.cluster_boundaries[index].clear();
            self.cluster_boundaries[index].push(index);
            self.idle_cluster_boundaries[index].clear();
            // TODO: copy length and boundary_length as well, since erasure error decoding requires modifying them on the fly
        }
        self.odd_clusters.clear();
        self.idle_odd_clusters.clear();
        self.clear_odd_clusters_set();
        self.time_uf_grow = 0.;
        self.count_uf_grow = 0;
        self.time_uf_merge = 0.;
        self.time_uf_update = 0.;
        self.time_uf_remove = 0.;
        self.count_node_visited = 0;
    }

    /// decode given measurement results
    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        // clean the state and then read measurement result
        let begin = Instant::now();
        self.clear();
        for position in sparse_measurement.iter() {
            let index = self.position_to_index[position];
            self.odd_clusters.push(index);
            self.insert_odd_clusters_set(index);
            self.nodes[index].is_error_syndrome = true;
            self.union_find.payload[index].as_mut().unwrap().cardinality = 1;  // odd
            if !self.nodes[index].node_visited {
                self.nodes[index].node_visited = true;
                self.count_node_visited += 1;
            }
        }
        let time_prepare_decoders = begin.elapsed().as_secs_f64();
        // decode
        let begin = Instant::now();
        if true {  // set to false when debugging
            self.run_to_stable();
        } else {
            self.detailed_print_run_to_stable();
        }
        let time_run_to_stable = begin.elapsed().as_secs_f64();
        // build correction based on the matching
        let begin = Instant::now();
        let mut correction = SparseCorrection::new();
        if !self.config.benchmark_skip_building_correction {
            // invalidate previous cache to save memory
            self.complete_model_graph.invalidate_previous_dijkstra();
            // in order to build correction, first collect the nodes for each cluster
            let mut cluster_nodes = BTreeMap::<usize, Vec<usize>>::new();
            for position in sparse_measurement.iter() {
                let index = self.position_to_index[position];
                let root = self.union_find.find(index);
                if !cluster_nodes.contains_key(&root) {
                    cluster_nodes.insert(root, vec![]);
                }
                cluster_nodes.get_mut(&root).unwrap().push(index);
            }
            // then build correction based on each correction
            for (root, mut error_syndromes) in cluster_nodes.into_iter() {
                let root_node_cardinality = self.union_find.get(root).cardinality;
                let cluster_boundary_index = self.union_find.get(root).touching_boundary_index;
                debug_assert!(root_node_cardinality > 0, "each nontrivial measurement must be in a non-empty cluster");
                assert_eq!(error_syndromes.len(), root_node_cardinality);
                if root_node_cardinality % 2 == 1 {
                    assert!(cluster_boundary_index != usize::MAX, "boundary of odd cluster must exists");
                    // connect to a boundary and others internally
                    error_syndromes.push(cluster_boundary_index);  // let it match with others
                    let cluster_boundary_position = &self.index_to_position[cluster_boundary_index];
                    // println!("match boundary {:?}", cluster_boundary_position);
                    let boundary_correction = self.complete_model_graph.build_correction_boundary(cluster_boundary_position);
                    correction.extend(&boundary_correction);
                }
                assert_eq!(error_syndromes.len() % 2, 0);
                let half_len = error_syndromes.len() / 2;
                for i in 0..half_len{
                    let index1 = error_syndromes[i];
                    let index2 = error_syndromes[i + half_len];
                    if index1 != index2 {
                        let position1 = &self.index_to_position[index1];
                        let position2 = &self.index_to_position[index2];
                        // println!("match peer {:?} {:?}", position1, position2);
                        let matching_correction = self.complete_model_graph.build_correction_matching(position1, position2);
                        correction.extend(&matching_correction);
                    }
                }
            }
        }
        let time_build_correction = begin.elapsed().as_secs_f64();
        (correction, json!({
            "time_run_to_stable": time_run_to_stable,
            "time_prepare_decoders": time_prepare_decoders,
            "time_uf_grow_step": self.time_uf_grow_step,
            "time_uf_grow": self.time_uf_grow,
            "count_uf_grow": self.count_uf_grow,
            "time_uf_merge": self.time_uf_merge,
            "time_uf_update": self.time_uf_update,
            "time_uf_remove": self.time_uf_remove,
            "time_build_correction": time_build_correction,
            "count_node_visited": self.count_node_visited,
        }))
    }

    /// run single iterations until no non-terminating (odd and not yet touching boundary) clusters exist
    pub fn run_to_stable(&mut self) {
        while !self.odd_clusters.is_empty() {
            self.run_single_iteration()
        }
    }

    /// debug function where a limited iterations can be run
    #[allow(dead_code)]
    pub fn detailed_print_run_to_stable(&mut self) {
        // let mut max_steps = 20usize;
        let mut max_steps = usize::MAX;
        while !self.odd_clusters.is_empty() && max_steps > 0 {
            println!("[info] iteration begin");
            if max_steps != usize::MAX { max_steps -= 1; }
            self.debug_print_clusters();
            println!("cluster boundaries:");
            self.debug_print_cluster_boundaries();
            self.run_single_iteration()
        }
        println!("[info] reached stable state");
        assert!(max_steps > 0, "run to stable terminated because of ");
        self.debug_print_clusters();
        println!("cluster boundaries:");
        self.debug_print_cluster_boundaries();
    }

    /// debug print
    #[allow(dead_code)]
    pub fn debug_print_clusters(&self) {
        let nodes_len = self.nodes.len();
        for i in 0..nodes_len {
            let this_position = &self.index_to_position[i];
            let root_position = &self.index_to_position[self.union_find.immutable_find(i)];
            let node = &self.nodes[i];
            let error_symbol = if node.is_error_syndrome { "x" } else { " " };
            let boundary_string = match node.boundary_length {
                Some(boundary_length) => {
                    let color = if node.boundary_increased > 0 { "\x1b[93m" } else { "" };
                    format!("{}b({}/{})\x1b[0m", color, node.boundary_increased, boundary_length)
                },
                None => format!("      "),
            };
            let neighbors_len = node.neighbors.len();
            let mut neighbor_string = String::new();
            for j in 0..neighbors_len {
                let partial_edge = &self.nodes[i].neighbors[j];
                let increased = partial_edge.increased;
                let neighbor_index = partial_edge.neighbor;
                let neighbor = &self.nodes[neighbor_index];
                let reverse_index = neighbor.index_to_neighbor(i).unwrap();
                let neighbor_partial_edge = &neighbor.neighbors[reverse_index];
                let neighbor_position = &self.index_to_position[neighbor_index];
                let color = if neighbor_partial_edge.increased + increased > 0 { "\x1b[93m" } else { "" };
                let string = format!("{}{}({}/{})\x1b[0m ", color, neighbor_position, neighbor_partial_edge.increased + increased, neighbor_partial_edge.length);
                neighbor_string.push_str(string.as_str());
            }
            let color = if this_position != root_position { "\x1b[96m" } else { "" };
            println!("{}{} ∈ {}\x1b[0m {} {} n: {}", color, this_position, root_position, error_symbol, boundary_string, neighbor_string);
        }
    }

    /// only print those `cluster_boundaries` != vec!\[itself\]
    #[allow(dead_code)]
    pub fn debug_print_cluster_boundaries(&self) {
        for (cluster, boundaries_vec) in self.cluster_boundaries.iter().enumerate() {
            if boundaries_vec.len() == 1 && !self.has_odd_clusters_set(cluster) {
                continue  // ignore printing this one
            }
            let mut user_data = Vec::new();
            for &idx in boundaries_vec.iter() {
                let position = &self.index_to_position[idx];
                user_data.push(format!("{}", position));
            }
            let root_position = &self.index_to_position[cluster];
            println!("{}: {}", root_position, user_data.join(" "));
        }
    }

    #[inline(never)]
    fn run_single_iteration_get_grow_step(&mut self) -> usize {
        let begin = Instant::now();
        let grow_step = if !self.config.use_real_weighted {
            1
        } else {
            // compute the maximum safe length to growth
            let mut maximum_safe_length = usize::MAX;
            for &odd_cluster in self.odd_clusters.iter() {
                let boundaries_vec = &self.cluster_boundaries[odd_cluster];
                for &boundary in boundaries_vec.iter() {
                    let neighbor_len = self.nodes[boundary].neighbors.len();
                    for i in 0..neighbor_len {
                        if !self.nodes[boundary].neighbors[i].grown {
                            let partial_edge = &mut self.nodes[boundary].neighbors[i];
                            let increased = partial_edge.increased;
                            let neighbor_index = partial_edge.neighbor;
                            let neighbor = &mut self.nodes[neighbor_index];
                            let reverse_index = neighbor.index_to_neighbor(boundary).unwrap();
                            let neighbor_partial_edge = &mut neighbor.neighbors[reverse_index];
                            if neighbor_partial_edge.increased + increased < neighbor_partial_edge.length {  // not fully grown yet
                                let mut safe_length = neighbor_partial_edge.length - (neighbor_partial_edge.increased + increased);
                                // judge if peer needs to grow as well, if so, the safe length is halved
                                let neighbor_root = self.union_find.find(neighbor_index);
                                if self.has_odd_clusters_set(neighbor_root) {
                                    safe_length /= 2;
                                }
                                if safe_length < maximum_safe_length {
                                    maximum_safe_length = safe_length;
                                }
                            }
                        }
                    }
                    // grow to the code boundary if it has
                    match self.nodes[boundary].boundary_length {
                        Some(boundary_length) => {
                            let boundary_increased = &mut self.nodes[boundary].boundary_increased;
                            if *boundary_increased < boundary_length {
                                let safe_length = boundary_length - *boundary_increased;
                                if safe_length < maximum_safe_length {
                                    maximum_safe_length = safe_length;
                                }
                            }
                        },
                        None => { }  // do nothing
                    }
                }
            }
            // grow step cannot be 0
            assert_ne!(maximum_safe_length, usize::MAX, "should find at least one un-grown edge");
            if maximum_safe_length != 0 { maximum_safe_length } else { 1 }
        };
        self.time_uf_grow_step += begin.elapsed().as_secs_f64();
        grow_step
    }

    /// grow and update cluster boundaries
    #[inline(never)]
    fn run_single_iteration_uf_grow(&mut self, grow_step: usize) {
        let fusion_list = &mut self.fusion_list;
        fusion_list.clear();
        let begin = Instant::now();
        for &odd_cluster in self.odd_clusters.iter() {
            let boundaries_vec = &self.cluster_boundaries[odd_cluster];
            for &boundary in boundaries_vec.iter() {
                // grow this boundary and check for grown edge at the same time
                let neighbor_len = self.nodes[boundary].neighbors.len();
                for i in 0..neighbor_len {
                    let (neighbor_index, grown) = {
                        let partial_edge = &self.nodes[boundary].neighbors[i];
                        (partial_edge.neighbor, partial_edge.grown)
                    };
                    if !grown || self.union_find.find(boundary) != self.union_find.find(neighbor_index) {
                        self.count_uf_grow += 1;
                        let partial_edge = &mut self.nodes[boundary].neighbors[i];
                        partial_edge.increased += grow_step;  // may over-grown, but ok as long as weight is much smaller than usize::MAX
                        let neighbor_index = partial_edge.neighbor;
                        let increased = partial_edge.increased;
                        let neighbor = &mut self.nodes[neighbor_index];
                        let reverse_index = neighbor.index_to_neighbor(boundary).unwrap();
                        let neighbor_partial_edge = &mut neighbor.neighbors[reverse_index];
                        if neighbor_partial_edge.increased + increased >= neighbor_partial_edge.length {  // found grown edge
                            neighbor_partial_edge.grown = true;
                            let partial_edge = &mut self.nodes[boundary].neighbors[i];
                            partial_edge.grown = true;
                            fusion_list.push((boundary, neighbor_index));
                            if !self.nodes[neighbor_index].node_visited {
                                self.nodes[neighbor_index].node_visited = true;
                                self.count_node_visited += 1;
                            }
                        }
                    }
                }
                // grow to the code boundary if it has
                match self.nodes[boundary].boundary_length {
                    Some(boundary_length) => {
                        let boundary_increased = &mut self.nodes[boundary].boundary_increased;
                        if *boundary_increased < boundary_length {
                            self.count_uf_grow += 1;
                            *boundary_increased += grow_step;
                            if *boundary_increased >= boundary_length {
                                self.union_find.get_mut(boundary).is_touching_boundary = true;  // this set is touching the boundary
                                self.union_find.get_mut(boundary).touching_boundary_index = boundary;
                            }
                        }
                    },
                    None => { }  // do nothing
                }
            }
        }
        // {  // debug print `fusion_list`
        //     println!("fusion_list:");
        //     for (a, b) in fusion_list.iter() {
        //         println!("    {} {}", self.index_to_position[*a], self.index_to_position[*b]);
        //     }
        // }
        self.time_uf_grow += begin.elapsed().as_secs_f64();
    }

    /// merge the clusters given `fusion_list` and also update the boundary list
    #[inline(never)]
    fn run_single_iteration_uf_merge(&mut self) {
        let fusion_list = &mut self.fusion_list;
        let begin = Instant::now();
        for &(a, b) in fusion_list.iter() {
            let a = self.union_find.find(a);  // update to its root
            let b = self.union_find.find(b);  // update to its root
            let real_merging = self.union_find.union(a, b);
            if real_merging {  // update the boundary list only when this is a real merging
                let to_be_appended = self.union_find.find(a);  // or self.union_find.find(r_b) equivalently
                assert!(to_be_appended == a || to_be_appended == b, "`to_be_appended` should be either `a` or `b`");
                let appending = if to_be_appended == a { b } else { a };  // the other one
                let appending_boundaries_vec = self.cluster_boundaries[appending].clone();
                let to_be_appended_boundaries_vec = &mut self.cluster_boundaries[to_be_appended];
                // append the boundary
                to_be_appended_boundaries_vec.extend(appending_boundaries_vec.iter());
            }
        }
        self.time_uf_merge += begin.elapsed().as_secs_f64();
    }

    /// update the boundary vertices
    #[inline(never)]
    fn run_single_iteration_uf_update(&mut self) {
        let begin = Instant::now();
        self.clear_odd_clusters_set();  // used as `visited_cluster`
        for &cluster in self.odd_clusters.iter() {  // TODO: odd_clusters here might be duplicated, use odd_clusters_set instead
            // replace `odd_clusters` by the root, so that querying `cluster_boundaries` will be valid
            let cluster = self.union_find.find(cluster);
            if self.has_odd_clusters_set(cluster) {
                continue
            }
            {  // borrow checker workaround
                // self.insert_odd_clusters_set(cluster);  // to prevent the same cluster to calculate twice; this boundary updating is expensive
                self.nodes[cluster].odd_clusters_set_timestamp = self.odd_clusters_set_active_timestamp;
            }
            // `cluster_boundaries` should only contain root ones now
            // shrink the boundary by checking if this is real boundary (neighbor are not all in the same set)
            {  // borrow checker workaround
                // self.clear_shrunk_boundaries();
                Self::clear_shrunk_boundaries_static(&mut self.nodes, &mut self.shrunk_boundaries_active_timestamp);
            }
            self.idle_cluster_boundaries[cluster].clear();
            for &boundary in self.cluster_boundaries[cluster].iter() {
                let mut has_foreign = false;
                let neighbor_len = self.nodes[boundary].neighbors.len();
                for i in 0..neighbor_len {
                    let partial_edge = &mut self.nodes[boundary].neighbors[i];
                    let neighbor_index = partial_edge.neighbor;
                    if cluster != self.union_find.find(neighbor_index) {
                        has_foreign = true;
                        break
                    }
                }
                let boundary_node = &self.nodes[boundary];
                match boundary_node.boundary_length {
                    Some(boundary_length) => {
                        if boundary_node.boundary_increased < boundary_length {
                            has_foreign = true;
                        }
                    },
                    None => { },  // do nothing
                }
                if has_foreign {
                    let not_present = {  // borrow checker workaround
                        // !self.has_shrunk_boundaries(boundary);
                        self.nodes[boundary].shrunk_boundaries_timestamp != self.shrunk_boundaries_active_timestamp
                    };
                    {  // borrow checker workaround
                        // self.insert_shrunk_boundaries(boundary);
                        self.nodes[boundary].shrunk_boundaries_timestamp = self.shrunk_boundaries_active_timestamp;
                    }
                    if not_present {
                        self.idle_cluster_boundaries[cluster].push(boundary);
                    }
                }
            }
            std::mem::swap(&mut self.idle_cluster_boundaries[cluster], &mut self.cluster_boundaries[cluster]);
        }
        self.time_uf_update += begin.elapsed().as_secs_f64();
    }

    /// remove the even clusters (includes those already touched the code boundary) from `odd_clusters`
    #[inline(never)]
    fn run_single_iteration_uf_remove(&mut self) {
        let begin = Instant::now();
        self.clear_odd_clusters_set();
        self.idle_odd_clusters.clear();
        for &odd_cluster in self.odd_clusters.iter() {
            let odd_cluster = self.union_find.find(odd_cluster);
            let union_node = self.union_find.get(odd_cluster);
            if union_node.cardinality % 2 == 1 && !union_node.is_touching_boundary {
                let not_present = !self.has_odd_clusters_set(odd_cluster);
                {  // borrow checker workaround
                    // self.insert_odd_clusters_set(odd_cluster);
                    self.nodes[odd_cluster].odd_clusters_set_timestamp = self.odd_clusters_set_active_timestamp
                }
                if not_present {
                    self.idle_odd_clusters.push(odd_cluster);
                }
            }
        }
        std::mem::swap(&mut self.idle_odd_clusters, &mut self.odd_clusters);
        self.time_uf_remove += begin.elapsed().as_secs_f64();
    }

    /// run a single iteration
    #[inline(never)]
    fn run_single_iteration(&mut self) {
        let grow_step = self.run_single_iteration_get_grow_step();
        self.run_single_iteration_uf_grow(grow_step);
        self.run_single_iteration_uf_merge();
        self.run_single_iteration_uf_update();
        self.run_single_iteration_uf_remove();
    }

}

pub type UnionFind = UnionFindGeneric<UnionNode>;

#[derive(Copy, Debug, Serialize, Deserialize, Clone)]
pub struct UnionNode {
    pub set_size: usize,
    pub cardinality: usize,
    pub is_touching_boundary: bool,
    pub touching_boundary_index: usize,
}

impl UnionNodeTrait for UnionNode {

    #[inline]
    fn union(left: &Self, right: &Self) -> Either<Self, Self> {
        let lsize = left.set_size;
        let rsize = right.set_size;
        let result = Self {
            set_size: lsize + rsize,
            cardinality: left.cardinality + right.cardinality,
            is_touching_boundary: left.is_touching_boundary || right.is_touching_boundary,
            touching_boundary_index: std::cmp::min(left.touching_boundary_index, right.touching_boundary_index),
        };
        if lsize >= rsize {
            Either::Left(result)
        } else {
            Either::Right(result)
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.set_size = 1;
        self.cardinality = 0;
        self.is_touching_boundary = false;
        self.touching_boundary_index = usize::MAX;
    }

}

impl Default for UnionNode {
    #[inline]
    fn default() -> Self {
        Self {
            set_size: 1,
            cardinality: 0,  // by default the cardinality is 0, set to 1 if needed
            is_touching_boundary: false,  // is already touching the boundary
            touching_boundary_index: usize::MAX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::code_builder::*;
    use super::super::types::ErrorType::*;

    #[test]
    fn union_find_decoder_code_capacity() {  // cargo test union_find_decoder_code_capacity -- --nocapture
        let d = 5;
        let noisy_measurements = 0;  // perfect measurement
        let p = 0.001;
        // build simulator
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode{ noisy_measurements, di: d, dj: d });
        code_builder_sanity_check(&simulator).unwrap();
        // build error model
        let mut error_model = ErrorModel::new(&simulator);
        simulator.set_error_rates(&mut error_model, p, p, p, 0.);
        simulator.compress_error_rates(&mut error_model);
        error_model_sanity_check(&simulator, &error_model).unwrap();
        // build decoder
        let decoder_config = json!({
            "precompute_complete_model_graph": true,
        });
        let enable_all = true;
        let mut union_find_decoder = UnionFindDecoder::new(&Arc::new(simulator.clone()), &error_model, &decoder_config, 1);
        if true || enable_all {  // debug 5
            simulator.clear_all_errors();
            // {"[0][4][6]":"Z","[0][5][8]":"Z","[0][5][9]":"Z","[0][7][1]":"Z","[0][9][1]":"Z"}
            simulator.get_node_mut_unwrap(&pos!(0, 4, 6)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 8)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 9)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 7, 1)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 9, 1)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 4, should fail
            simulator.clear_all_errors();
            // {"[0][1][2]":"Z","[0][1][5]":"Z","[0][5][3]":"Z","[0][5][7]":"Z","[0][7][2]":"Z","[0][7][7]":"Z"}
            simulator.get_node_mut_unwrap(&pos!(0, 1, 2)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 1, 5)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 3)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 7)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 7, 2)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 7, 7)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
        }
        if false || enable_all {  // debug 3
            simulator.clear_all_errors();
            // {"[0][1][8]":"Z","[0][6][5]":"Z","[0][6][6]":"Z","[0][8][2]":"Z","[0][8][4]":"Z"}
            simulator.get_node_mut_unwrap(&pos!(0, 1, 8)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 5)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 2)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 4)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 2
            simulator.clear_all_errors();
            // {"[0][3][2]":"Z","[0][3][9]":"Z","[0][8][8]":"Z","[0][9][6]":"Z"}
            simulator.get_node_mut_unwrap(&pos!(0, 3, 2)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 3, 9)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 8, 8)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 9, 6)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
        if false || enable_all {  // debug 1
            simulator.clear_all_errors();
            simulator.get_node_mut_unwrap(&pos!(0, 6, 4)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 6, 6)).set_error(&error_model, &Z);
            simulator.get_node_mut_unwrap(&pos!(0, 5, 7)).set_error(&error_model, &Z);
            simulator.propagate_errors();
            let sparse_measurement = simulator.generate_sparse_measurement();
            let (correction, _runtime_statistics) = union_find_decoder.decode(&sparse_measurement);
            // println!("{:?}", correction);
            code_builder_sanity_check_correction(&mut simulator, &correction).unwrap();
            let (logical_i, logical_j) = simulator.validate_correction(&correction);
            assert!(!logical_i && !logical_j);
        }
    }
    
}
