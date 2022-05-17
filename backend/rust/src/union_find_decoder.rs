//! Union Find Decoder
//!
//! (this is an implementation of <https://arxiv.org/pdf/1709.06218.pdf>)
//!
//! The Union Find algorithm borrows code from <https://github.com/gifnksm/union-find-rs>
//! with some modifications to store extra information of the set.
//!
//! A small improvement over the paper is that we allow integer cost of each edge, while the original paper fixed the cost of each edge to 2.
//! The allows a more accurate result, in the cost of longer execution time (proportional to the average cost of edge)
//!

use std::iter::FromIterator;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use super::serde::{Serialize, Deserialize};
use super::offer_decoder;
use super::ftqec;
use super::types::QubitType;
use super::petgraph;
use super::distributed_uf_decoder;
use super::serde_json;
use std::time::Instant;
use super::ftqec::{Measurement, DetectedErasures, Index, FastHashIndex};
use super::util::simple_hasher::SimpleHasher;
use super::union_find::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeprecatedUnionFindDecoder<U: std::fmt::Debug> {
    /// each node corresponds to a stabilizer
    pub nodes: Vec<DecoderNode<U>>,
    /// union find solver
    pub union_find: DefaultUnionFind,
    /// all odd clusters that need to update in each turn, clusters are named under the root
    pub odd_clusters: Vec<usize>,  // to reduce complexity of iterating over odd_clusters
    pub odd_clusters_set: HashSet<usize>,  // for ease of query
    /// record the boundary nodes as an optimization, see <https://arxiv.org/pdf/1709.06218.pdf> Section "Boundary representation".
    /// even clusters should not be key in HashMap, and only real boundary should be in the `HashSet` value
    /// those nodes without error syndrome also have entries in this HashMap, with the value of { itself }
    pub cluster_boundaries: HashMap<usize, (Vec<usize>, HashSet<usize>)>,
    /// original inputs
    pub input_neighbors: Vec<NeighborEdge>,
    // DEBUG: study the time consumption of each step
    pub time_uf_grow: f64,
    pub time_uf_merge: f64,
    pub time_uf_replace: f64,
    pub time_uf_update: f64,
    pub time_uf_remove: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DecoderNode<U: std::fmt::Debug> {
    /// the corresponding node in the input graph
    pub node: InputNode<U>,
    /// directly connected neighbors, (address, already increased length = 0, length = 0)
    pub neighbors: Vec<NeighborPartialEdge>,
    /// the mapping from node index to NeighborPartialEdge index
    pub neighbor_index: HashMap<usize, usize>,
    /// increased region towards boundary, only valid when `node.boundary_cost` is `Some(_)`
    pub boundary_increased: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputNode<U: std::fmt::Debug> {
    /// user defined data corresponds to each node, can be `()` if not used
    pub user_data: U,
    /// whether this stabilizer has detected a error
    pub is_error_syndrome: bool,
    /// if this node has a direct path to boundary, then set to `Some(cost)` given the integer cost of matching to boundary, otherwise `None`.
    /// This attribute can be modified later, by calling `TODO` in `DeprecatedUnionFindDecoder`
    pub boundary_cost: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeighborPartialEdge {
    /// the index of neighbor
    pub address: usize,
    /// already increased length, initialized as 0. erasure should initialize as `length` (or any value at least `length`/2)
    pub increased: usize,
    /// the total length of this edge. if the sum of the `increased` of two partial edges is no less than `length`, then two vertices are merged
    pub length: usize,
    /// performance optimization by caching whether it's already grown
    pub grown: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeighborEdge {
    /// address of node `a`
    pub a: usize,
    /// address of node `b`
    pub b: usize,
    /// current increased value, should be 0 in general, and can be `length` if there exists an erasure
    pub increased: usize,
    /// the total length of this edge
    pub length: usize,
}

impl<U: std::fmt::Debug> InputNode<U> {
    pub fn new(user_data: U, is_error_syndrome: bool, boundary_cost: Option<usize>) -> Self {
        Self {
            user_data: user_data,
            is_error_syndrome: is_error_syndrome,
            boundary_cost: boundary_cost,
        }
    }
}

impl Ord for NeighborEdge {
    fn cmp(&self, other: &Self) -> Ordering {
        let x_s = std::cmp::min(self.a, self.b);
        let x_l = std::cmp::max(self.a, self.b);
        let y_s = std::cmp::min(other.a, other.b);
        let y_l = std::cmp::max(other.a, other.b);
        match x_s.cmp(&y_s) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => x_l.cmp(&y_l),
        }
    }
}

impl PartialOrd for NeighborEdge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NeighborEdge {
    fn eq(&self, other: &Self) -> bool {
        let x_s = std::cmp::min(self.a, self.b);
        let x_l = std::cmp::max(self.a, self.b);
        let y_s = std::cmp::min(other.a, other.b);
        let y_l = std::cmp::max(other.a, other.b);
        x_s == y_s && x_l == y_l
    }
}

impl Eq for NeighborEdge {}

impl NeighborEdge {
    pub fn new(a: usize, b: usize, increased: usize, length: usize) -> Self {
        Self {
            a: a,
            b: b,
            increased: increased,
            length: length,
        }
    }
}

impl<U: std::fmt::Debug> DeprecatedUnionFindDecoder<U> {
    pub fn new(nodes: Vec<InputNode<U>>, mut neighbors: Vec<NeighborEdge>) -> Self {
        let mut nodes: Vec<_> = nodes.into_iter().map(|node| {
            DecoderNode {
                node: node,
                neighbors: Vec::new(),
                neighbor_index: HashMap::with_capacity(100),  // usually 4~12 neighbors, 10x larger capacity
                boundary_increased: 0,
            }
        }).collect();
        let odd_clusters: Vec<_> = nodes.iter().enumerate().filter(|(_idx, node)| {
            node.node.is_error_syndrome
        }).map(|(idx, _node)| {
            idx
        }).collect();
        let odd_clusters_set: HashSet<_> = odd_clusters.iter().map(|idx| *idx).collect();
        let cluster_boundaries: HashMap<_, _> = nodes.iter().enumerate().map(|(idx, _node)| {
            (idx, (vec![idx], vec![idx].into_iter().collect::<HashSet<usize>>()))
        }).collect();  // only roots of these odd clusters are boundaries in the initial state
        // union find solver
        let union_find = DefaultUnionFind::from_iter(nodes.iter().map(|node| {
            DefaultUnionNode {
                set_size: 1,
                cardinality: if node.node.is_error_syndrome { 1 } else { 0 },
                is_touching_boundary: false,  // the set is never touching the boundary at the beginning
            }
        }));
        // filter duplicated neighbor edges
        let nodes_length = nodes.len();
        let neighbors_length = neighbors.len();
        neighbors.retain(|edge| {
            edge.a != edge.b && edge.a < nodes_length && edge.b < nodes_length
        });  // remove invalid neighbor edges
        assert_eq!(neighbors_length, neighbors.len(), "`neighbors` contains invalid edges (either invalid address or edge connecting the same node)");
        neighbors.sort_unstable();
        neighbors.dedup();
        assert_eq!(neighbors_length, neighbors.len(), "`neighbors` contains duplicate elements (including the same ends)");
        for NeighborEdge {a, b, increased, length} in neighbors.iter() {
            let grown = 2 * increased >= *length;
            let a_idx = nodes[*a].neighbors.len();
            nodes[*a].neighbor_index.insert(*b, a_idx);
            nodes[*a].neighbors.push(NeighborPartialEdge {
                address: *b,
                increased: *increased,
                length: *length,
                grown: grown,
            });
            let b_idx = nodes[*b].neighbors.len();
            nodes[*b].neighbor_index.insert(*a, b_idx);
            nodes[*b].neighbors.push(NeighborPartialEdge {
                address: *a,
                increased: *increased,
                length: *length,
                grown: grown,
            });
        }
        Self {
            nodes: nodes,
            union_find: union_find,
            odd_clusters: odd_clusters,
            odd_clusters_set: odd_clusters_set,
            cluster_boundaries: cluster_boundaries,
            input_neighbors: neighbors,
            time_uf_grow: 0.,
            time_uf_merge: 0.,
            time_uf_replace: 0.,
            time_uf_update: 0.,
            time_uf_remove: 0.,
        }
    }

    /// need to call this after changing the grown state manually
    pub fn update_cluster_state(&mut self) {
        // update touching boundary
        for node_idx in 0..self.nodes.len() {
            let node = &self.nodes[node_idx];
            match node.node.boundary_cost {
                Some(boundary_cost) => {
                    if node.boundary_increased >= boundary_cost {
                        self.union_find.get_mut(node_idx).is_touching_boundary = true
                    }
                }, _ => { },
            }
        }
        self.run_single_iteration_optional_grow(true);
        // update time counter
        self.time_uf_grow = 0.;
        // self.time_uf_merge = 0.;  // keep merge time, to test union find operation
        self.time_uf_replace = 0.;
        self.time_uf_update = 0.;
        self.time_uf_remove = 0.;
    }
    
    pub fn run_single_iteration(&mut self) {
        self.run_single_iteration_optional_grow(false)
    }

    /// run a single turn
    fn run_single_iteration_optional_grow(&mut self, no_growing: bool) {
        // grow and update cluster boundaries
        let begin = Instant::now();
        let mut fusion_list = Vec::with_capacity(self.input_neighbors.len());
        if no_growing {  // iterate all clusters no matter it's odd or even
            self.odd_clusters = self.cluster_boundaries.keys().map(|idx| *idx).collect();
        }
        for &odd_cluster in self.odd_clusters.iter() {
            let (boundaries_vec, _boundaries) = self.cluster_boundaries.get(&odd_cluster).unwrap();
            for &boundary in boundaries_vec.iter() {
                // grow this boundary and check for grown edge at the same time
                let neighbor_len = self.nodes[boundary].neighbors.len();
                for i in 0..neighbor_len {
                    let partial_edge = &mut self.nodes[boundary].neighbors[i];
                    if partial_edge.grown && !no_growing {
                        continue  // already grown
                    }
                    if !no_growing {
                        partial_edge.increased += 1;
                    }
                    let increased = partial_edge.increased;
                    let neighbor_addr = partial_edge.address;
                    let neighbor = &mut self.nodes[neighbor_addr];
                    let reverse_index = neighbor.neighbor_index[&boundary];
                    let neighbor_partial_edge = &mut neighbor.neighbors[reverse_index];
                    if neighbor_partial_edge.increased + increased >= neighbor_partial_edge.length {  // found grown edge
                        fusion_list.push((boundary, neighbor_addr));
                        neighbor_partial_edge.grown = true;
                        let partial_edge = &mut self.nodes[boundary].neighbors[i];
                        partial_edge.grown = true;
                    }
                }
                // grow to the code boundary if it has
                match self.nodes[boundary].node.boundary_cost {
                    Some(boundary_cost) => {
                        let boundary_increased = &mut self.nodes[boundary].boundary_increased;
                        if *boundary_increased < boundary_cost || no_growing {
                            if !no_growing {
                                *boundary_increased += 1;
                            }
                            if *boundary_increased >= boundary_cost {
                                self.union_find.get_mut(boundary).is_touching_boundary = true;  // this set is touching the boundary
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
        //         println!("    {:?} {:?}", self.nodes[*a].node.user_data, self.nodes[*b].node.user_data);
        //     }
        // }
        self.time_uf_grow += begin.elapsed().as_secs_f64();
        // merge the clusters given `fusion_list` and also update the boundary list
        let begin = Instant::now();
        for &(a, b) in fusion_list.iter() {
            let a = self.union_find.find(a);  // update to its root
            let b = self.union_find.find(b);  // update to its root
            let real_merging = self.union_find.union(a, b);
            if real_merging {  // update the boundary list only when this is a real merging
                let to_be_appended = self.union_find.find(a);  // or self.union_find.find(r_b) equivalently
                assert!(to_be_appended == a || to_be_appended == b, "`to_be_appended` should be either `a` or `b`");
                let appending = if to_be_appended == a { b } else { a };  // the other one
                let (appending_boundaries_vec, appending_boundaries) = self.cluster_boundaries.remove(&appending).unwrap();
                let (to_be_appended_boundaries_vec, to_be_appended_boundaries) = self.cluster_boundaries.get_mut(&to_be_appended).unwrap();
                // append the boundary
                to_be_appended_boundaries_vec.extend(&appending_boundaries_vec);
                to_be_appended_boundaries.extend(&appending_boundaries);
            }
        }
        self.time_uf_merge += begin.elapsed().as_secs_f64();
        // replace `odd_clusters` by the root, so that querying `cluster_boundaries` will be valid
        let begin = Instant::now();
        let union_find = &mut self.union_find;
        self.odd_clusters = self.odd_clusters.iter().map(|&odd_cluster| {
            union_find.find(odd_cluster)
        }).collect();
        self.time_uf_replace += begin.elapsed().as_secs_f64();
        // update the boundary vertices
        let begin = Instant::now();
        for &cluster in self.odd_clusters.iter() {
            let (boundaries_vec, boundaries) = self.cluster_boundaries.get_mut(&cluster).unwrap();
            // `cluster_boundaries` should only contain root ones now
            // assert_eq!(cluster, self.union_find.find(cluster), "non-root boundaries should already been removed");
            // first grow the boundary
            // let mut grown_boundaries = HashSet::new();
            // for &boundary in boundaries.iter() {
            //     let neighbor_len = self.nodes[boundary].neighbors.len();
            //     for i in 0..neighbor_len {
            //         let partial_edge = &mut self.nodes[boundary].neighbors[i];
            //         let increased = partial_edge.increased;
            //         let neighbor_addr = partial_edge.address;
            //         let neighbor = &self.nodes[neighbor_addr];
            //         let reverse_index = neighbor.neighbor_index[&boundary];
            //         let neighbor_partial_edge = &neighbor.neighbors[reverse_index];
            //         if neighbor_partial_edge.increased + increased >= neighbor_partial_edge.length {  // this is grown edge
            //             grown_boundaries.insert(neighbor_addr);
            //         }
            //     }
            // }
            // then shrink the boundary by checking if this is real boundary (neighbor are not all in the same set)
            let mut shrunk_boundaries = HashSet::with_capacity(boundaries_vec.len() * 10);
            let mut shrunk_boundaries_vec = Vec::with_capacity(boundaries_vec.len());
            for &boundary in boundaries.iter() {
                let mut has_foreign = false;
                let neighbor_len = self.nodes[boundary].neighbors.len();
                for i in 0..neighbor_len {
                    let partial_edge = &mut self.nodes[boundary].neighbors[i];
                    let neighbor_addr = partial_edge.address;
                    if cluster != self.union_find.find(neighbor_addr) {
                        has_foreign = true;
                        break
                    }
                }
                let boundary_node = &self.nodes[boundary];
                match boundary_node.node.boundary_cost {
                    Some(boundary_cost) => {
                        if boundary_node.boundary_increased < boundary_cost {
                            has_foreign = true;
                        }
                    },
                    None => { },  // do nothing
                }
                if has_foreign {
                    let not_present = shrunk_boundaries.insert(boundary);
                    if not_present {
                        shrunk_boundaries_vec.push(boundary);
                    }
                }
            }
            // replace the boundary list
            *boundaries_vec = shrunk_boundaries_vec;
            *boundaries = shrunk_boundaries;
        }
        self.time_uf_update += begin.elapsed().as_secs_f64();
        // remove the even clusters (includes those already touched the code boundary) from `odd_clusters`
        let begin = Instant::now();
        let mut odd_clusters_set = HashSet::with_capacity(self.odd_clusters.len() * 10);
        let mut odd_clusters = Vec::with_capacity(self.odd_clusters.len());
        for &odd_cluster in self.odd_clusters.iter() {
            let union_node = self.union_find.get(odd_cluster);
            if union_node.cardinality % 2 == 1 && !union_node.is_touching_boundary {
                let not_present = odd_clusters_set.insert(odd_cluster);
                if not_present {
                    odd_clusters.push(odd_cluster);
                }
            }
        }
        self.odd_clusters = odd_clusters;
        self.odd_clusters_set = odd_clusters_set;
        self.time_uf_remove += begin.elapsed().as_secs_f64();
    }

    pub fn run_to_stable(&mut self) {
        while !self.odd_clusters.is_empty() {
            self.run_single_iteration()
        }
    }

    #[allow(dead_code)]
    pub fn detailed_print_run_to_stable(&mut self) {
        // let mut max_steps = 20usize;
        let mut max_steps = usize::MAX;
        while !self.odd_clusters.is_empty() && max_steps > 0 {
            if max_steps != usize::MAX { max_steps -= 1; }
            self.pretty_print_standard_planar_code();
            println!("cluster boundaries:");
            self.pretty_print_cluster_boundaries();
            self.run_single_iteration()
        }
        assert!(max_steps > 0, "run to stable terminated because of ");
        self.pretty_print_standard_planar_code();
        println!("cluster boundaries:");
        self.pretty_print_cluster_boundaries();
    }

    #[allow(dead_code)]
    pub fn pretty_print_standard_planar_code(&self) {
        let nodes_len = self.nodes.len();
        for i in 0..nodes_len {
            let root_user_data = &self.nodes[self.union_find.immutable_find(i)].node.user_data;
            let node = &self.nodes[i];
            let error_symbol = if node.node.is_error_syndrome { "x" } else { " " };
            let boundary_string = match node.node.boundary_cost {
                Some(boundary_cost) => {
                    format!("b({}/{})", node.boundary_increased, boundary_cost)
                },
                None => format!("      "),
            };
            let neighbors_len = node.neighbors.len();
            let mut neighbor_string = String::new();
            for j in 0..neighbors_len {
                let partial_edge = &self.nodes[i].neighbors[j];
                let increased = partial_edge.increased;
                let neighbor_addr = partial_edge.address;
                let neighbor = &self.nodes[neighbor_addr];
                let reverse_index = neighbor.neighbor_index[&i];
                let neighbor_partial_edge = &neighbor.neighbors[reverse_index];
                let neighbor_user_data = &neighbor.node.user_data;
                let string = format!("{:?}[{}/{}] ", neighbor_user_data, neighbor_partial_edge.increased + increased, neighbor_partial_edge.length);
                neighbor_string.push_str(string.as_str());
            }
            println!("{:?} ∈ {:?} {} {} n: {}", node.node.user_data, root_user_data, error_symbol, boundary_string, neighbor_string);
        }
    }

    /// only print those `cluster_boundaries` != vec!\[itself\]
    #[allow(dead_code)]
    pub fn pretty_print_cluster_boundaries(&self) {
        for (&cluster, (boundaries_vec, _boundaries)) in self.cluster_boundaries.iter() {
            if boundaries_vec.len() == 1 && self.odd_clusters_set.get(&cluster).is_none() {
                continue  // ignore printing this one
            }
            let mut user_data = Vec::new();
            for &idx in boundaries_vec.iter() {
                user_data.push(format!("{:?}", self.nodes[idx].node.user_data));
            }
            println!("{:?}: {}", self.nodes[cluster].node.user_data, user_data.join(" "));
        }
    }
}

/// create nodes for standard planar code (2d, perfect measurement condition). return only X stabilizers or only Z stabilizers.
/// return (nodes, position_to_index, neighbors)
pub fn make_standard_planar_code_2d_nodes(d: usize, is_x_stabilizers: bool) -> (Vec<InputNode<(usize, usize)>>, HashMap<(usize, usize), usize>, Vec<NeighborEdge>) {
    let mut nodes = Vec::new();
    let mut position_to_index = HashMap::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            position_to_index.insert((i, j), nodes.len());
            let is_boundary = if is_x_stabilizers { j == 1 || j == 2*d-3 } else { i == 1 || i == 2*d-3 };
            nodes.push(InputNode::new((i, j), false, if is_boundary { Some(2) } else { None }));
        }
    }
    let mut neighbors = Vec::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            for (di, dj) in [(2, 0), (0, 2)].iter() {
                let ni = i + di;
                let nj = j + dj;
                if ni <= 2*d-2 && nj <= 2*d-2 {
                    neighbors.push(NeighborEdge::new(position_to_index[&(i, j)], position_to_index[&(ni, nj)], 0, 2));
                }
            }
        }
    }
    (nodes, position_to_index, neighbors)
}

pub fn make_standard_planar_code_2d_nodes_only_x_stabilizers(d: usize) -> (Vec<InputNode<(usize, usize)>>, HashMap<(usize, usize), usize>, Vec<NeighborEdge>) {
    make_standard_planar_code_2d_nodes(d, true)
}

pub fn get_standard_planar_code_2d_left_boundary_cardinality(d: usize, position_to_index: &HashMap<(usize, usize), usize>
        , decoder: &DeprecatedUnionFindDecoder<(usize, usize)>, get_top_boundary_instead: bool, enable_toward_mwpm: bool) -> usize {
    let mut boundary_cardinality = 0;
    let mut peer_boundary_has = HashSet::new();
    if enable_toward_mwpm {
        for index in (0..=2*d-2).step_by(2) {
            let i = if get_top_boundary_instead { 2*d-3 } else { index };
            let j = if get_top_boundary_instead { index } else { 2*d-3 };
            let index = position_to_index[&(i, j)];
            let root = decoder.union_find.immutable_find(index);
            let node = &decoder.nodes[index];
            if node.boundary_increased >= node.node.boundary_cost.unwrap() {  // only when this node is bleeding into the boundary
                peer_boundary_has.insert(root);
            }
        }
    }
    let mut counted_sets = HashSet::new();
    for index in (0..=2*d-2).step_by(2) {
        let i = if get_top_boundary_instead { 1 } else { index };
        let j = if get_top_boundary_instead { index } else { 1 };
        let index = position_to_index[&(i, j)];
        let root = decoder.union_find.immutable_find(index);
        if !counted_sets.contains(&root) {  // every set should only be counted once
            let node = &decoder.nodes[index];
            if node.boundary_increased >= node.node.boundary_cost.unwrap() {  // only when this node is bleeding into the boundary
                counted_sets.insert(root);
                let root_uf_node = &decoder.union_find.immutable_get(root);
                if root_uf_node.cardinality % 2 == 1 {  // connect to boundary only if the cardinality is odd
                    if peer_boundary_has.contains(&root) {
                        // if don't consider this case, either by counting into left boundary or not into left boundary,
                        //    the results are almost the same (d=11, p=1e-1, x only, logical error rate = 16%)
                        // The main difference between original UF decoder and MWPM decoder lies here
                        if enable_toward_mwpm {

                        }
                    } else {
                        boundary_cardinality += 1;
                    }
                }
            }
        }
    }
    boundary_cardinality
}

pub fn get_standard_planar_code_3d_left_boundary_cardinality(d: usize, measurement_rounds: usize, position_to_index: &HashMap<(usize, usize, usize), usize>
        , decoder: &DeprecatedUnionFindDecoder<(usize, usize, usize)>, get_top_boundary_instead: bool, enable_toward_mwpm: bool) -> usize {
    let mut boundary_cardinality = 0;
    let mut peer_boundary_has = HashSet::new();
    if enable_toward_mwpm {
        for t in (18..=12+6*measurement_rounds).step_by(6) {
            for index in (0..=2*d-2).step_by(2) {
                let i = if get_top_boundary_instead { 2*d-3 } else { index };
                let j = if get_top_boundary_instead { index } else { 2*d-3 };
                let index = position_to_index[&(t, i, j)];
                let root = decoder.union_find.immutable_find(index);
                let node = &decoder.nodes[index];
                if node.boundary_increased >= node.node.boundary_cost.unwrap() {  // only when this node is bleeding into the boundary
                    peer_boundary_has.insert(root);
                }
            }
        }
    }
    let mut counted_sets = HashSet::new();
    for t in (18..=12+6*measurement_rounds).step_by(6) {
        for index in (0..=2*d-2).step_by(2) {
            let i = if get_top_boundary_instead { 1 } else { index };
            let j = if get_top_boundary_instead { index } else { 1 };
            let index = position_to_index[&(t, i, j)];
            let root = decoder.union_find.immutable_find(index);
            if !counted_sets.contains(&root) {  // every set should only be counted once
                let node = &decoder.nodes[index];
                // only when this node is bleeding into the boundary
                if node.node.boundary_cost.is_some() && node.boundary_increased >= node.node.boundary_cost.unwrap() {
                    counted_sets.insert(root);
                    let root_uf_node = &decoder.union_find.immutable_get(root);
                    if root_uf_node.cardinality % 2 == 1 {  // connect to boundary only if the cardinality is odd
                        if peer_boundary_has.contains(&root) {
                            // if don't consider this case, either by counting into left boundary or not into left boundary,
                            //    the results are almost the same (d=11, p=1e-1, x only, logical error rate = 16%)
                            // The main difference between original UF decoder and MWPM decoder lies here
                            if enable_toward_mwpm {

                            }
                        } else {
                            boundary_cardinality += 1;
                        }
                    }
                }
            }
        }
    }
    boundary_cardinality
}

/// return `(has_x_logical_error, has_z_logical_error)`
pub fn run_given_offer_decoder_instance(decoder: &mut offer_decoder::OfferDecoder, towards_mwpm: bool) -> (bool, bool) {
    let d = decoder.d;
    decoder.error_changed();
    // decode X errors
    let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes(d, true);
    for i in (0..=2*d-2).step_by(2) {
        for j in (1..=2*d-3).step_by(2) {
            if decoder.qubits[i][j].measurement {
                nodes[position_to_index[&(i, j)]].is_error_syndrome = true;
            }
        }
    }
    let mut uf_decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
    uf_decoder.run_to_stable();
    let left_boundary_cardinality = get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &uf_decoder, false, towards_mwpm)
        + decoder.origin_error_left_boundary_cardinality();
    let has_x_logical_error = left_boundary_cardinality % 2 == 1;
    // decode Z errors
    let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes(d, false);
    for i in (1..=2*d-3).step_by(2) {
        for j in (0..=2*d-2).step_by(2) {
            if decoder.qubits[i][j].measurement {
                nodes[position_to_index[&(i, j)]].is_error_syndrome = true;
            }
        }
    }
    let mut uf_decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
    uf_decoder.run_to_stable();
    let top_boundary_cardinality = get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &uf_decoder, true, towards_mwpm)
        + decoder.origin_error_top_boundary_cardinality();
    let has_z_logical_error = top_boundary_cardinality % 2 == 1;
    (has_x_logical_error, has_z_logical_error)
}

#[allow(dead_code)]
/// return (nodes, position_to_index, neighbors)
pub fn make_decoder_given_ftqec_model(model: &ftqec::PlanarCodeModel, stabilizer: QubitType) -> (Vec<InputNode<(usize, usize, usize)>>,
        HashMap<(usize, usize, usize), usize>, Vec<NeighborEdge>) {
    assert!(stabilizer != QubitType::Data, "cannot build decoder on data qubits");
    let mut nodes = Vec::new();
    let mut position_to_index = HashMap::new();
    model.iterate_measurement_stabilizers(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            position_to_index.insert((t, i, j), nodes.len());
            nodes.push(InputNode::new((t, i, j), false, if node.boundary.is_some() { Some(2) } else { None }));
        }
    });
    model.iterate_measurement_errors(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            nodes[position_to_index[&(t, i, j)]].is_error_syndrome = true;
        }
    });
    let mut neighbors = Vec::new();
    model.iterate_measurement_stabilizers(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            let idx = position_to_index[&(t, i, j)];
            for edge in node.edges.iter() {
                if edge.t > 12 {
                    let peer_idx = position_to_index[&(edge.t, edge.i, edge.j)];
                    if idx < peer_idx {  // remove duplicated neighbors
                        neighbors.push(NeighborEdge::new(idx, peer_idx, 0, 2));
                    }
                } else {
                    nodes[idx].boundary_cost = Some(2);  // viewing the bottom layer as boundary
                }
            }
        }
    });
    assert!(neighbors.len() > 0, "ftqec model may not have `build_graph` called, so the neighbor connections have not built yet");
    (nodes, position_to_index, neighbors)
}

#[allow(dead_code)]
/// return (nodes, position_to_index, neighbors)
pub fn make_decoder_given_ftqec_model_weighted(model: &ftqec::PlanarCodeModel, stabilizer: QubitType, max_half_weight: usize)
        -> (Vec<InputNode<(usize, usize, usize)>>, HashMap<(usize, usize, usize), usize>, Vec<NeighborEdge>) {
    assert!(stabilizer != QubitType::Data, "cannot build decoder on data qubits");
    // first find the maximum weight
    let mut maximum_weight = f64::MIN;
    model.iterate_measurement_stabilizers(|t, _i, _j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            for edge in node.edges.iter() {
                if edge.p > 0. && edge.weight > maximum_weight {
                    maximum_weight = edge.weight;
                }
            }
            match &node.boundary {
                Some(boundary) => {
                    if boundary.p > 0. && boundary.weight > maximum_weight {
                        maximum_weight = boundary.weight;
                    }
                },
                None => { }
            }
        }
    });
    // the minimum probability has weight `max_half_weight`, all other probability will scale accordingly
    let scale_weight = |weight: f64| -> usize {
        let mut half_weight = ((max_half_weight as f64) * weight / maximum_weight).round() as usize;
        if half_weight > max_half_weight {
            half_weight = max_half_weight;
        }
        if half_weight < 1 {
            half_weight = 1;
        }
        // println!("half_weight = {}, minimum_probability = {}, probability = {}", half_weight, minimum_probability, probability);
        2 * half_weight
    };
    // build graph
    let mut nodes = Vec::new();
    let mut position_to_index = HashMap::new();
    model.iterate_measurement_stabilizers(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            position_to_index.insert((t, i, j), nodes.len());
            nodes.push(InputNode::new((t, i, j), false, match &node.boundary {
                Some(boundary) => Some(scale_weight(boundary.weight)),
                None => None,
            }));
        }
    });
    model.iterate_measurement_errors(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            nodes[position_to_index[&(t, i, j)]].is_error_syndrome = true;
        }
    });
    let mut neighbors = Vec::new();
    model.iterate_measurement_stabilizers(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            let idx = position_to_index[&(t, i, j)];
            for edge in node.edges.iter() {
                if edge.t > 12 {
                    let peer_idx = position_to_index[&(edge.t, edge.i, edge.j)];
                    if idx < peer_idx {  // remove duplicated neighbors
                        neighbors.push(NeighborEdge::new(idx, peer_idx, 0, scale_weight(edge.weight)));
                    }
                } else {
                    let new_boundary_cost = match nodes[idx].boundary_cost {
                        Some(cost) => std::cmp::min(cost, scale_weight(edge.weight)),
                        None => scale_weight(edge.weight),
                    };
                    nodes[idx].boundary_cost = Some(new_boundary_cost);  // viewing the bottom layer as boundary
                }
            }
        }
    });
    assert!(neighbors.len() > 0, "ftqec model may not have `build_graph` called, so the neighbor connections have not built yet");
    (nodes, position_to_index, neighbors)
}

/// return `(has_x_logical_error, has_z_logical_error)`
pub fn run_given_mwpm_decoder_instance_weighted(model: &ftqec::PlanarCodeModel, towards_mwpm: bool, max_half_weight: usize, use_xzzx_code: bool) -> (bool, bool) {
    assert_eq!(model.di, model.dj, "currently square code supported");
    let d = model.di;
    let measurement_rounds = model.MeasurementRounds;
    let default_correction = model.generate_default_correction();
    let (x_error_count, z_error_count) = model.get_boundary_cardinality(&default_correction);
    let x_error_qubit_type = if use_xzzx_code { QubitType::StabXZZXLogicalX } else { QubitType::StabZ };
    let z_error_qubit_type = if use_xzzx_code { QubitType::StabXZZXLogicalZ } else { QubitType::StabX };
    // decode X errors
    let (nodes, position_to_index, neighbors) = make_decoder_given_ftqec_model_weighted(&model, x_error_qubit_type, max_half_weight);
    let mut uf_decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
    uf_decoder.run_to_stable();
    let left_boundary_cardinality = get_standard_planar_code_3d_left_boundary_cardinality(d, measurement_rounds, &position_to_index, &uf_decoder
        , use_xzzx_code, towards_mwpm) + x_error_count;
    let has_x_logical_error = left_boundary_cardinality % 2 == 1;
    // decode Z errors
    let (nodes, position_to_index, neighbors) = make_decoder_given_ftqec_model_weighted(&model, z_error_qubit_type, max_half_weight);
    let mut uf_decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
    uf_decoder.run_to_stable();
    let top_boundary_cardinality = get_standard_planar_code_3d_left_boundary_cardinality(d, measurement_rounds, &position_to_index, &uf_decoder
        , !use_xzzx_code, towards_mwpm) + z_error_count;
    let has_z_logical_error = top_boundary_cardinality % 2 == 1;
    (has_x_logical_error, has_z_logical_error)
}

pub fn suboptimal_matching_by_union_find_given_measurement_build_decoders(model: &ftqec::PlanarCodeModel, measurement: &Measurement
        , detected_erasures: &DetectedErasures, max_half_weight: usize)
        -> Vec<DeprecatedUnionFindDecoder<(usize, usize, usize)>> {
    let di = model.di;
    let dj = model.dj;
    let fhi = |index: Index| -> FastHashIndex {
        FastHashIndex::with_di_dj(&index, di, dj)
    };
    // first find individual connected regions (so that the decoder weight is optimal for every region)
    let mut g = petgraph::graph::UnGraph::<(usize, usize, usize), ()>::default();
    let mut tij_to_index = HashMap::new();
    // add nodes
    model.iterate_measurement_stabilizers(|t, i, j, _node| {
        let index = g.add_node((t, i, j));
        tij_to_index.insert((t, i, j), index);
    });
    // add edges from Pauli errors
    model.iterate_measurement_stabilizers(|t, i, j, node| {
        for edge in node.edges.iter().filter(|edge| edge.p > 0.) {
            let index1 = tij_to_index[&(t, i, j)];
            let index2 = tij_to_index[&(edge.t, edge.i, edge.j)];
            g.update_edge(index1, index2, ());  // use `update_edge` instead of `add_edge` is to avoid duplicate edges
        }
    });
    // add edges from erasure errors
    if detected_erasures.has_erasures() {
        for (idx1, idx2) in detected_erasures.connected.iter() {
            let index1 = tij_to_index[&(idx1.index.t, idx1.index.i, idx1.index.j)];
            let index2 = tij_to_index[&(idx2.index.t, idx2.index.i, idx2.index.j)];
            g.update_edge(index1, index2, ());  // use `update_edge` instead of `add_edge` is to avoid duplicate edges
        }
    }
    // get connected regions, each connected region corresponds to an instance of union find decoder
    let mut already_visited = HashSet::new();
    let mut connected_regions = Vec::new();
    for current_node_index in g.node_indices() {
        let mut current_region = Vec::new();
        if already_visited.contains(&current_node_index) {
            continue;
        }
        let mut dfs = petgraph::visit::Dfs::new(&g, current_node_index);
        while let Some(nx) = dfs.next(&g) {
            current_region.push(nx);
            already_visited.insert(nx);
        }
        current_region.sort_unstable();
        current_region.dedup();  // remove duplicate ones
        connected_regions.push(current_region);
    }
    // { // debug print connected regions
    //     for region in connected_regions.iter() {
    //         print!("region:");
    //         for idx in region.iter() {
    //             let (t, i, j) = g.node_weight(*idx).expect("exists");
    //             print!(" ({}, {}, {})", t, i, j);
    //         }
    //         println!("");
    //     }
    // }
    let mut decoders = Vec::new();
    for region in connected_regions.iter() {
        let region_set = {
            let mut region_set = HashSet::<FastHashIndex, std::hash::BuildHasherDefault::<SimpleHasher> >::default();
            if detected_erasures.has_erasures() {
                region_set.extend(region.iter().map(|idx| {
                    let &(t, i, j) = g.node_weight(*idx).expect("exists");
                    fhi(Index::new(t, i, j))
                }));
            }
            region_set
        };
        // first find the maximum weight
        let mut maximum_weight = f64::MIN;
        for idx in region.iter() {
            let &(t, i, j) = g.node_weight(*idx).expect("exists");
            let node = model.snapshot[t][i][j].as_ref().expect("exist");
            for edge in node.edges.iter().filter(|edge| edge.p > 0.) {
                if edge.weight > maximum_weight {
                    maximum_weight = edge.weight;
                }
            }
            match &node.boundary {
                Some(boundary) => {
                    if boundary.p > 0. && boundary.weight > maximum_weight {
                        maximum_weight = boundary.weight;
                    }
                },
                None => { }
            }
        }
        // weight will scale accordingly
        let scale_weight = |weight: f64| -> usize {
            let mut half_weight = ((max_half_weight as f64) * weight / maximum_weight).round() as usize;
            if half_weight > max_half_weight {
                half_weight = max_half_weight;
            }
            if half_weight < 1 {
                half_weight = 1;
            }
            // println!("half_weight = {}, minimum_probability = {}, probability = {}", half_weight, minimum_probability, probability);
            2 * half_weight
        };
        // build graph
        let mut nodes = Vec::new();
        let mut position_to_index = HashMap::new();
        for idx in region.iter() {
            let &(t, i, j) = g.node_weight(*idx).expect("exists");
            let node = model.snapshot[t][i][j].as_ref().expect("exist");
            position_to_index.insert((t, i, j), nodes.len());
            nodes.push(InputNode::new((t, i, j), false, node.boundary.as_ref().and_then(|boundary| {
                if boundary.p > 0. {  Some(scale_weight(boundary.weight)) } else { None }
            }).or(detected_erasures.boundaries.get(&fhi(Index::new(t, i, j))).and(Some(0)))));
        }
        model.iterate_measurement_stabilizers(|t, i, j, _node| {
            let (mt, mi, mj) = ftqec::Index::new(t, i, j).to_measurement_idx();
            if position_to_index.contains_key(&(t, i, j)) && measurement[[mt, mi, mj]] {
                nodes[position_to_index[&(t, i, j)]].is_error_syndrome = true;
            }
        });
        let mut neighbors = Vec::new();
        for g_idx in region.iter() {
            let &(t, i, j) = g.node_weight(*g_idx).expect("exists");
            let node = model.snapshot[t][i][j].as_ref().expect("exist");
            let idx = position_to_index[&(t, i, j)];
            let mut added_edges_idx = HashSet::new();  // to avoid duplicate edges when considering edges caused by pure erasure errors
            for edge in node.edges.iter().filter(|edge| edge.p > 0.) {
                let peer_idx = position_to_index[&(edge.t, edge.i, edge.j)];
                if idx < peer_idx {  // remove duplicated neighbors
                    let mut edge_increased = 0;
                    let edge_length = scale_weight(edge.weight);
                    if detected_erasures.has_erasures() {  // check for erasure errors
                        added_edges_idx.insert(fhi(Index::new(edge.t, edge.i, edge.j)));
                        if detected_erasures.connected_contains(&Index::new(t, i, j), &Index::new(edge.t, edge.i, edge.j)) {
                            edge_increased = edge_length;  // set as already grown (actually edge_length/2 suffices)
                        }
                    }
                    neighbors.push(NeighborEdge::new(idx, peer_idx, edge_increased, edge_length));
                }
            }
            if detected_erasures.has_erasures() {
                match detected_erasures.connected_edges.get(&fhi(Index::new(t, i, j))) {
                    Some(edges) => {
                        for target in edges.iter() {
                            // target must in this region, and also cannot be already added as grown edge
                            if region_set.get(target).is_some() && added_edges_idx.get(target).is_none() {
                                let peer_idx = position_to_index[&(target.index.t, target.index.i, target.index.j)];
                                if idx < peer_idx {  // remove duplicated neighbors
                                    neighbors.push(NeighborEdge::new(idx, peer_idx, 0, 0));  // weight 0 edges is specific for erasure errors
                                }
                            }
                        }
                    }, _ => { }
                }
            }
        }
        // build union find decoder
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        // set boundary edge already grown for erasure errors
        if detected_erasures.has_erasures() {
            for node in decoder.nodes.iter_mut() {
                let (t, i, j) = node.node.user_data;
                if detected_erasures.boundaries.get(&fhi(Index::new(t, i, j))).is_some() {
                    node.boundary_increased = node.node.boundary_cost.expect("exists");
                }
            }
            decoder.update_cluster_state();
        }
        decoders.push(decoder);
    }
    decoders
}

pub fn suboptimal_matching_by_union_find_given_measurement_generate_suboptimal_matching(decoder: &DeprecatedUnionFindDecoder<(usize, usize, usize)>)
        -> (Vec<((usize, usize, usize), (usize, usize, usize))>, Vec<(usize, usize, usize)>) {
    let mut edge_matchings = Vec::new();
    let mut boundary_matchings = Vec::new();
    let mut counted_sets = HashSet::new();
    for index in 0..decoder.nodes.len() {
        let root = decoder.union_find.immutable_find(index);
        if counted_sets.contains(&root) {  // every set should only be counted once
            continue;
        }
        let root_node = &decoder.union_find.immutable_get(root);
        if root_node.cardinality == 0 {  // ignore clusters without errors
            continue;
        }
        counted_sets.insert(root);
        // find all errors in this cluster
        let mut error_syndromes = Vec::new();
        let mut cluster_boundary = None;
        for index2 in 0..decoder.nodes.len() {
            let root2 = decoder.union_find.immutable_find(index2);
            if root2 == root {
                let node2 = &decoder.nodes[index2];
                if node2.node.boundary_cost.is_some() && node2.boundary_increased >= node2.node.boundary_cost.unwrap() {
                    cluster_boundary = Some(index2);
                }
                if node2.node.is_error_syndrome {
                    error_syndromes.push(index2)
                }
            }
        }
        assert_eq!(error_syndromes.len(), root_node.cardinality);
        if root_node.cardinality % 2 == 1 {
            // connect to a boundary and others internally
            let cluster_boundary_index = cluster_boundary.expect("odd cluster should at least have 1 boundary");
            error_syndromes.push(cluster_boundary_index);
            let node = &decoder.nodes[cluster_boundary_index];
            boundary_matchings.push(node.node.user_data);
        }
        assert_eq!(error_syndromes.len() % 2, 0);
        let half_len = error_syndromes.len() / 2;
        for i in 0..half_len{
            let node1 = &decoder.nodes[error_syndromes[i]];
            let node2 = &decoder.nodes[error_syndromes[i + half_len]];
            if node1.node.user_data != node2.node.user_data {
                edge_matchings.push((node1.node.user_data, node2.node.user_data));
            }
        }
    }
    (edge_matchings, boundary_matchings)
}
/// given an arbitrary ftqec::PlanarCodeModel, this function returns a suboptimal matching given by union find decoder
pub fn suboptimal_matching_by_union_find_given_measurement(model: &ftqec::PlanarCodeModel, measurement: &Measurement
        , detected_erasures: &DetectedErasures, max_half_weight: usize
        , use_distributed: bool, detailed_runtime_statistics: bool)
        -> (Vec<((usize, usize, usize), (usize, usize, usize))>, Vec<(usize, usize, usize)>, serde_json::Value) {
    let mut edge_matchings = Vec::new();
    let mut boundary_matchings = Vec::new();
    let mut time_run_to_stable = 0.;
    let mut time_build_decoders = 0.;
    let mut duf_clock_cycles = 0;
    let begin = Instant::now();
    let decoders = suboptimal_matching_by_union_find_given_measurement_build_decoders(model, measurement, detected_erasures, max_half_weight);
    time_build_decoders += begin.elapsed().as_secs_f64();
    let mut time_uf_grow = 0.;
    let mut time_uf_merge = 0.;
    let mut time_uf_replace = 0.;
    let mut time_uf_update = 0.;
    let mut time_uf_remove = 0.;
    for mut decoder in decoders.into_iter() {
        // run union find decoder
        if use_distributed {
            let begin = Instant::now();
            let (mut duf_decoder, _position_to_index) = distributed_uf_decoder::build_distributed_union_find_given_uf_decoder_3d(&decoder);
            time_build_decoders += begin.elapsed().as_secs_f64();
            let begin = Instant::now();
            duf_clock_cycles += duf_decoder.run_to_stable();
            time_run_to_stable += begin.elapsed().as_secs_f64();
            distributed_uf_decoder::copy_state_back_to_union_find_decoder(&mut decoder, &duf_decoder);
        } else {
            let begin = Instant::now();
            decoder.run_to_stable();
            // decoder.detailed_print_run_to_stable();
            time_run_to_stable += begin.elapsed().as_secs_f64();
            time_uf_grow += decoder.time_uf_grow;
            time_uf_merge += decoder.time_uf_merge;
            time_uf_replace += decoder.time_uf_replace;
            time_uf_update += decoder.time_uf_update;
            time_uf_remove += decoder.time_uf_remove;
        }
        // generate suboptimal matching
        let (mut local_edge_matchings, mut local_boundary_matchings) = suboptimal_matching_by_union_find_given_measurement_generate_suboptimal_matching(&decoder);
        // println!("local_edge_matchings: {:?}", local_edge_matchings);
        // println!("local_boundary_matchings: {:?}", local_boundary_matchings);
        edge_matchings.append(&mut local_edge_matchings);
        boundary_matchings.append(&mut local_boundary_matchings);
    }
    let mut runtime_statistics = json!({
        "time_run_to_stable": time_run_to_stable,
        "time_build_decoders": time_build_decoders,
    });
    if use_distributed {
        runtime_statistics["duf_clock_cycles"] = json!(duf_clock_cycles);
    } else {
        if detailed_runtime_statistics {
            runtime_statistics["time_uf_grow"] = json!(time_uf_grow);
            runtime_statistics["time_uf_merge"] = json!(time_uf_merge);
            runtime_statistics["time_uf_replace"] = json!(time_uf_replace);
            runtime_statistics["time_uf_update"] = json!(time_uf_update);
            runtime_statistics["time_uf_remove"] = json!(time_uf_remove);
        }
    }
    (edge_matchings, boundary_matchings, runtime_statistics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ErrorType;

    // use `cargo test union_find_decoder_sanity_check_1 -- --nocapture` to run specific test
    
    #[test]
    #[should_panic]
    fn union_find_decoder_sanity_check_1() {
        let (nodes, position_to_index, mut neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(3);
        println!("nodes: {:#?}", nodes);
        println!("position_to_index: {:?}", position_to_index);
        println!("neighbors: {:?}", neighbors);
        // add duplicate neighbor edge
        neighbors.push(NeighborEdge::new(position_to_index[&(0, 1)], position_to_index[&(0, 3)], 1000, 1000));
        // should then panic
        DeprecatedUnionFindDecoder::new(nodes, neighbors);
    }
    
    #[test]
    fn union_find_decoder_sanity_check_2() {
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(3);
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        let decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        println!("decoder.odd_clusters: {:?}", decoder.odd_clusters);
    }
    
    #[test]
    fn union_find_decoder_sanity_check_3() {
        let mut a = HashSet::<usize>::new();
        a.insert(1);
        a.insert(2);
        println!("a: {:?}", a);
        let a_ref = &mut a;
        *a_ref = HashSet::<usize>::new();
        println!("a: {:?}", a);
    }

    #[test]
    fn union_find_decoder_test_case_1() {
        let d = 3;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        assert_eq!(nodes.len(), 6, "d=3 should have 6 nodes");
        assert_eq!(neighbors.len(), 7, "d=3 should have 7 direct neighbor connections");
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        decoder.detailed_print_run_to_stable();
        // decoder.run_to_stable();
        // pretty_print_standard_planar_code(&decoder);
        assert_eq!(0, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn union_find_decoder_test_case_2() {
        let d = 5;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 7)]].is_error_syndrome = true;
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        decoder.detailed_print_run_to_stable();
        // decoder.run_to_stable();
        // pretty_print_standard_planar_code(&decoder);
        assert_eq!(0, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn union_find_decoder_test_case_3() {
        let d = 5;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(0, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(0, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(0, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 5)]].is_error_syndrome = true;
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        decoder.detailed_print_run_to_stable();
        // decoder.run_to_stable();
        // pretty_print_standard_planar_code(&decoder);
        assert_eq!(1, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn union_find_decoder_test_build_given_ftqec_model() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(measurement_rounds, d);
        model.set_phenomenological_error_with_perfect_initialization(p);
        model.build_graph(ftqec::weight_autotune);
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        model.add_error_at(el2t(0), 0, 2, &ErrorType::X).expect("error rate = 0 here");  // data qubit error (detected by next layer)
        model.add_error_at(el2t(1), 2, 3, &ErrorType::X).expect("error rate = 0 here");  // measurement error (detected by this and next layer)
        model.propagate_error();
        let (nodes, _position_to_index, neighbors) = make_decoder_given_ftqec_model(&model, QubitType::StabZ);
        assert_eq!(d * (d - 1) * measurement_rounds, nodes.len());
        assert_eq!((measurement_rounds * (d * (d - 1) * 2 - d - (d - 1)) + (measurement_rounds - 1) * d * (d - 1)), neighbors.len());
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        decoder.detailed_print_run_to_stable();
    }

    #[test]
    fn union_find_decoder_test_build_given_ftqec_model_weighted_1() {
        let p = 0.005;
        let bias_eta = 299.;
        let d = 3;
        let measurement_rounds = 1;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(measurement_rounds, d);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        model.set_individual_error_with_perfect_initialization(0., 0., 0.);
        // shallow_error_on_bottom
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t == 12 && node.qubit_type == QubitType::Data {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.add_error_at(12, 0, 2, &ErrorType::Z).expect("error rate = 0 here");  // data qubit error (detected by next layer)
        model.propagate_error();
        let (nodes, position_to_index, neighbors) = make_decoder_given_ftqec_model_weighted(&model, QubitType::StabXZZXLogicalZ, 4);
        assert_eq!(d * (d - 1) * measurement_rounds, nodes.len());
        assert_eq!((measurement_rounds * (d * (d - 1) * 2 - d - (d - 1)) + (measurement_rounds - 1) * d * (d - 1)), neighbors.len());
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        decoder.detailed_print_run_to_stable();
        assert_eq!(0, get_standard_planar_code_3d_left_boundary_cardinality(d, measurement_rounds, &position_to_index, &decoder, false, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn union_find_decoder_test_build_given_ftqec_model_weighted_2() {
        let p = 0.005;
        let bias_eta = 299.;
        let d = 3;
        let measurement_rounds = 1;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(measurement_rounds, d);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        model.set_individual_error_with_perfect_initialization(0., 0., 0.);
        // shallow_error_on_bottom
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t == 12 && node.qubit_type == QubitType::Data {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.add_error_at(12, 0, 2, &ErrorType::Z).expect("error rate = 0 here");  // data qubit error (detected by next layer)
        model.add_error_at(12, 0, 4, &ErrorType::Z).expect("error rate = 0 here");  // data qubit error (detected by next layer)
        model.propagate_error();
        let (nodes, position_to_index, neighbors) = make_decoder_given_ftqec_model_weighted(&model, QubitType::StabXZZXLogicalZ, 4);
        assert_eq!(d * (d - 1) * measurement_rounds, nodes.len());
        assert_eq!((measurement_rounds * (d * (d - 1) * 2 - d - (d - 1)) + (measurement_rounds - 1) * d * (d - 1)), neighbors.len());
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        decoder.detailed_print_run_to_stable();
        assert_eq!(1, get_standard_planar_code_3d_left_boundary_cardinality(d, measurement_rounds, &position_to_index, &decoder, false, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn union_find_decoder_test_build_given_ftqec_model_weighted_3() {
        let p = 0.005;
        let bias_eta = 299.;
        let d = 5;
        let measurement_rounds = 1;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(measurement_rounds, d);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        model.set_individual_error_with_perfect_initialization(0., 0., 0.);
        // shallow_error_on_bottom
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t == 12 && node.qubit_type == QubitType::Data {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.add_error_at(12, 2, 0, &ErrorType::Z).expect("error rate = 0 here");  // data qubit error (detected by next layer)
        model.propagate_error();
        let (nodes, position_to_index, neighbors) = make_decoder_given_ftqec_model_weighted(&model, QubitType::StabXZZXLogicalZ, 4);
        assert_eq!(d * (d - 1) * measurement_rounds, nodes.len());
        assert_eq!((measurement_rounds * (d * (d - 1) * 2 - d - (d - 1)) + (measurement_rounds - 1) * d * (d - 1)), neighbors.len());
        let mut decoder = DeprecatedUnionFindDecoder::new(nodes, neighbors);
        decoder.detailed_print_run_to_stable();
        assert_eq!(1, get_standard_planar_code_3d_left_boundary_cardinality(d, measurement_rounds, &position_to_index, &decoder, false, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }
    
    #[test]
    fn union_find_decoder_test_build_given_ftqec_model_weighted_4() {
        let p = 0.005;
        let bias_eta = 299.;
        let d = 5;
        let measurement_rounds = 1;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(measurement_rounds, d);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        model.set_individual_error_with_perfect_initialization(0., 0., 0.);
        // shallow_error_on_bottom
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t == 12 && node.qubit_type == QubitType::Data {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        let (has_x_logical_error, has_z_logical_error) = run_given_mwpm_decoder_instance_weighted(&mut model
            , false, 4, true);
        println!("has_x_logical_error: {}, has_z_logical_error: {}", has_x_logical_error, has_z_logical_error);
    }
    
    #[test]
    fn union_find_decoder_test_suboptimal_matching_by_union_find_1() {
        let p = 0.05;
        let bias_eta = 10.;
        let d = 7;
        let measurement_rounds = 1;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(measurement_rounds, d);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        model.set_individual_error_with_perfect_initialization(0., 0., 0.);
        // shallow_error_on_bottom
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t == 12 && node.qubit_type == QubitType::Data {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        // add errors
        model.add_error_at(12, 0, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.add_error_at(12, 0, 12, &ErrorType::Z).expect("error rate = 0 here");
        model.add_error_at(12, 9, 7, &ErrorType::Z).expect("error rate = 0 here");
        model.propagate_error();
        let measurement = model.generate_measurement();
        let (edge_matchings, boundary_matchings, _runtime_statistics) = suboptimal_matching_by_union_find_given_measurement(
            &model, &measurement, &ftqec::DetectedErasures::new(d, d), 4, false, false);
        println!("edge_matchings: {:?}", edge_matchings);
        println!("boundary_matchings: {:?}", boundary_matchings);
    }

    #[test]
    fn union_find_decoder_test_decode_erasure_error_1() {
        let d = 3;
        let pe = 0.1;  // erasure error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(0, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(0., 0., 0., 0.);
        model.iterate_snapshot_mut(|t, _i, _j, node| {  // shallow error on bottom
            if t == 6 && node.qubit_type == QubitType::Data {
                node.erasure_error_rate = pe;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.optimize_correction_pattern();
        model.build_exhausted_path();
        // add errors
        model.add_erasure_error_at(6, 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        model.propagate_error();
        // decode
        let measurement = model.generate_measurement();
        let detected_erasures = model.generate_detected_erasures();
        let (correction, _) = model.decode_UnionFind(&measurement, &detected_erasures, 4, false, false);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        assert!(validation_ret.is_ok());
    }

    #[test]
    fn union_find_decoder_test_decode_erasure_error_2() {
        let d = 3;
        let pe = 0.1;  // erasure error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(0, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(0., 0., 0., 0.);
        model.iterate_snapshot_mut(|t, _i, _j, node| {  // shallow error on bottom
            if t == 6 && node.qubit_type == QubitType::Data {
                node.erasure_error_rate = pe;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.optimize_correction_pattern();
        model.build_exhausted_path();
        // add errors
        model.add_erasure_error_at(6, 0, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.propagate_error();
        // decode
        let measurement = model.generate_measurement();
        let detected_erasures = model.generate_detected_erasures();
        let (correction, _) = model.decode_UnionFind(&measurement, &detected_erasures, 4, false, false);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        assert!(validation_ret.is_ok());
    }

    #[test]
    fn union_find_decoder_test_decode_erasure_error_3() {
        let d = 3;
        let pe = 0.1;  // erasure error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(0, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(0., 0., 0., 0.);
        model.iterate_snapshot_mut(|t, _i, _j, node| {  // shallow error on bottom
            if t == 6 && node.qubit_type == QubitType::Data {
                node.erasure_error_rate = pe;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.optimize_correction_pattern();
        model.build_exhausted_path();
        // add errors
        model.add_erasure_error_at(6, 0, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 2, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.propagate_error();
        // decode
        let measurement = model.generate_measurement();
        let detected_erasures = model.generate_detected_erasures();
        let (correction, _) = model.decode_UnionFind(&measurement, &detected_erasures, 4, false, false);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        assert!(validation_ret.is_ok());
    }

    #[test]
    fn union_find_decoder_test_decode_erasure_error_4() {
        let d = 3;
        let pe = 0.1;  // erasure error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(0, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(0., 0., 0., 0.);
        model.iterate_snapshot_mut(|t, _i, _j, node| {  // shallow error on bottom
            if t == 6 && node.qubit_type == QubitType::Data {
                node.erasure_error_rate = pe;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.optimize_correction_pattern();
        model.build_exhausted_path();
        // add errors
        model.add_erasure_error_at(6, 2, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 4, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.propagate_error();
        // decode
        let measurement = model.generate_measurement();
        let detected_erasures = model.generate_detected_erasures();
        let (correction, _) = model.decode_UnionFind(&measurement, &detected_erasures, 4, false, false);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        assert!(validation_ret.is_ok());
    }

    /*
     * debug case which never terminates
     * solved: it's because when updating the growing region, I didn't iterating over even clusters to find grown edges
     *        it's important to fuse even clusters!
     */
    #[test]
    fn union_find_decoder_test_decode_erasure_error_5() {
        let di = 3;
        let dj = 7;
        let pe = 0.2;  // erasure error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code_rectangle(0, di, dj);
        model.set_individual_error_with_perfect_initialization_with_erasure(0., 0., 0., 0.);
        model.iterate_snapshot_mut(|t, _i, _j, node| {  // shallow error on bottom
            if t == 6 && node.qubit_type == QubitType::Data {
                node.erasure_error_rate = pe;
            }
        });
        model.build_graph(ftqec::weight_autotune);
        model.optimize_correction_pattern();
        model.build_exhausted_path();
        // add errors
        model.add_erasure_error_at(6, 0, 0, &ErrorType::Y).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 1, 1, &ErrorType::Z).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 1, 5, &ErrorType::I).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 1, 9, &ErrorType::I).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 1, 11, &ErrorType::Y).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 2, 2, &ErrorType::Y).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 2, 6, &ErrorType::Z).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 2, 8, &ErrorType::I).expect("error rate = 0 here");
        model.add_erasure_error_at(6, 3, 7, &ErrorType::X).expect("error rate = 0 here");
        model.propagate_error();
        // decode
        // model.iterate_measurement_errors(|t, i, j, _node| {
        //     println!("measurement error at [{}][{}][{}]", t, i, j);
        // });
        let measurement = model.generate_measurement();
        let detected_erasures = model.generate_detected_erasures();
        let (correction, _) = model.decode_UnionFind(&measurement, &detected_erasures, 4, false, false);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        assert!(validation_ret.is_ok());
    }

}
