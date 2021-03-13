//! Union Find Decoder
//!
//! (this is an implementation of https://arxiv.org/pdf/1709.06218.pdf)
//!
//! The Union Find algorithm borrows code from https://github.com/gifnksm/union-find-rs
//! with some modifications to store extra information of the set.
//!
//! A small improvement over the paper is that we allow integer cost of each edge, while the original paper fixed the cost of each edge to 2.
//! The allows a more accurate result, in the cost of longer execution time (proportional to the average cost of edge)
//!

use std::iter::FromIterator;
use std::mem;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use super::serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnionFindDecoder<U: std::fmt::Debug> {
    /// each node corresponds to a stabilizer
    pub nodes: Vec<DecoderNode<U>>,
    /// union find solver
    pub union_find: UnionFind,
    /// all odd clusters that need to update in each turn, clusters are named under the root
    pub odd_clusters: HashSet<usize>,
    /// record the boundary nodes as an optimization, see https://arxiv.org/pdf/1709.06218.pdf Section "Boundary representation".
    /// only odd clusters should be the key in HashMap, and only real boundary should be in the `HashSet` value
    pub cluster_boundaries: HashMap<usize, HashSet<usize>>,
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
    /// This attribute can be modified later, by calling `TODO` in `UnionFindDecoder`
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

impl<U: std::fmt::Debug> UnionFindDecoder<U> {
    pub fn new(nodes: Vec<InputNode<U>>, mut neighbors: Vec<NeighborEdge>) -> Self {
        let mut nodes: Vec<_> = nodes.into_iter().map(|node| {
            DecoderNode {
                node: node,
                neighbors: Vec::new(),
                neighbor_index: HashMap::new(),
                boundary_increased: 0,
            }
        }).collect();
        let odd_clusters: HashSet<_> = nodes.iter().enumerate().filter(|(_idx, node)| {
            node.node.is_error_syndrome
        }).map(|(idx, _node)| {
            idx
        }).collect();
        let cluster_boundaries: HashMap<_, _> = odd_clusters.iter().map(|&idx| {
            (idx, vec![idx].into_iter().collect::<HashSet<usize>>())
        }).collect();  // only roots of these odd clusters are boundaries in the initial state
        // union find solver
        let union_find = UnionFind::from_iter(nodes.iter().map(|node| {
            UnionNode {
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
            let a_idx = nodes[*a].neighbors.len();
            nodes[*a].neighbor_index.insert(*b, a_idx);
            nodes[*a].neighbors.push(NeighborPartialEdge {
                address: *b,
                increased: *increased,
                length: *length,
            });
            let b_idx = nodes[*b].neighbors.len();
            nodes[*b].neighbor_index.insert(*a, b_idx);
            nodes[*b].neighbors.push(NeighborPartialEdge {
                address: *a,
                increased: *increased,
                length: *length,
            });
        }
        Self {
            nodes: nodes,
            union_find: union_find,
            odd_clusters: odd_clusters,
            cluster_boundaries: cluster_boundaries,
        }
    }

    /// run a single turn
    pub fn run_single_iteration(&mut self) {
        // grow and update cluster boundaries
        let mut fusion_list = Vec::new();
        for &odd_cluster in self.odd_clusters.iter() {
            let boundaries = self.cluster_boundaries.get(&odd_cluster).unwrap();
            for &boundary in boundaries.iter() {
                // grow this boundary and check for grown edge at the same time
                let neighbor_len = self.nodes[boundary].neighbors.len();
                for i in 0..neighbor_len {
                    let partial_edge = &mut self.nodes[boundary].neighbors[i];
                    partial_edge.increased += 1;
                    let increased = partial_edge.increased;
                    let neighbor_addr = partial_edge.address;
                    let neighbor = &self.nodes[neighbor_addr];
                    let reverse_index = neighbor.neighbor_index[&boundary];
                    let neighbor_partial_edge = &neighbor.neighbors[reverse_index];
                    if neighbor_partial_edge.increased + increased >= neighbor_partial_edge.length {  // found grown edge
                        fusion_list.push((boundary, neighbor_addr))
                    }
                }
                // grow to the code boundary if it has
                match self.nodes[boundary].node.boundary_cost {
                    Some(boundary_cost) => {
                        let boundary_increased = &mut self.nodes[boundary].boundary_increased;
                        *boundary_increased += 1;
                        if *boundary_increased >= boundary_cost {
                            self.union_find.get_mut(boundary).is_touching_boundary = true;  // this set is touching the boundary
                        }
                    },
                    None => { }  // do nothing
                }
            }
        }
        // merge the clusters given `fusion_list` and also update the boundary list
        for &(a, b) in fusion_list.iter() {
            let real_merging = self.union_find.union(a, b);
            if real_merging {  // update the boundary list only when this is a real merging
                let to_be_appended = self.union_find.find(a);  // u should be either `a` or `b`
                let appending = if to_be_appended == a { b } else { a };  // the other one
                let appending_vec = self.cluster_boundaries.remove(&appending).unwrap();
                self.cluster_boundaries.get_mut(&to_be_appended).unwrap().extend(&appending_vec);  // append the boundary
            }
        }
        // replace `odd_clusters` by the root, so that querying `cluster_boundaries` will be valid
        let union_find = &mut self.union_find;
        self.odd_clusters = self.odd_clusters.iter().map(|&odd_cluster| {
            union_find.find(odd_cluster)
        }).collect();
        // update the boundary vertices
        for (&cluster, boundaries) in self.cluster_boundaries.iter_mut() {
            // `cluster_boundaries` should only contain root ones now
            assert_eq!(cluster, self.union_find.find(cluster), "non-root boundaries should already been removed");
            // first grow the boundary
            let mut grown_boundaries = HashSet::new();
            for &boundary in boundaries.iter() {
                let neighbor_len = self.nodes[boundary].neighbors.len();
                for i in 0..neighbor_len {
                    let partial_edge = &mut self.nodes[boundary].neighbors[i];
                    let increased = partial_edge.increased;
                    let neighbor_addr = partial_edge.address;
                    let neighbor = &self.nodes[neighbor_addr];
                    let reverse_index = neighbor.neighbor_index[&boundary];
                    let neighbor_partial_edge = &neighbor.neighbors[reverse_index];
                    if neighbor_partial_edge.increased + increased >= neighbor_partial_edge.length {  // this is grown edge
                        grown_boundaries.insert(neighbor_addr);
                    }
                }
            }
            // then shrink the boundary by checking if this is real boundary (neighbor are not all in the same set)
            let mut shrunk_boundaries = HashSet::new();
            for &boundary in grown_boundaries.iter() {
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
                if has_foreign {
                    shrunk_boundaries.insert(boundary);
                }
            }
            // replace the boundary list
            *boundaries = shrunk_boundaries;
        }
        // remove the even clusters (includes those already touched the code boundary) from `odd_clusters`
        let mut odd_clusters = HashSet::new();
        for &odd_cluster in self.odd_clusters.iter() {
            let union_node = self.union_find.get(odd_cluster);
            if union_node.cardinality % 2 == 1 && !union_node.is_touching_boundary {
                odd_clusters.insert(odd_cluster);
            }
        }
        self.odd_clusters = odd_clusters;
    }

    pub fn run_to_stable(&mut self) {
        while !self.odd_clusters.is_empty() {
            self.run_single_iteration()
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
            nodes.push(InputNode::new((i, j), false, if is_boundary { Some(1) } else { None }));
        }
    }
    let mut neighbors = Vec::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            for (di, dj) in [(2, 0), (0, 2)].iter() {
                let ni = i + di;
                let nj = j + dj;
                if ni <= 2*d-2 && nj <= 2*d-3 {
                    neighbors.push(NeighborEdge::new(position_to_index[&(i, j)], position_to_index[&(ni, nj)], 0, 2));
                }
            }
        }
    }
    (nodes, position_to_index, neighbors)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnionFind {
    /// tree structure, each node has a parent
    link_parent: Vec<usize>,
    /// the node information, has the same length as `link_parent`
    payload: Vec<Option<UnionNode>>,
}

#[derive(Copy, Debug, Serialize, Deserialize, Clone)]
pub struct UnionNode {
    set_size: usize,
    cardinality: usize,
    is_touching_boundary: bool,
}

#[derive(Copy, Debug, Serialize, Deserialize, Clone)]
pub enum UnionResult {
    Left(UnionNode),
    Right(UnionNode),
}

impl UnionNode {
    #[inline]
    pub fn union(left: Self, right: Self) -> UnionResult {
        let lsize = left.size();
        let rsize = right.size();
        let result = UnionNode {
            set_size: lsize + rsize,
            cardinality: left.cardinality + right.cardinality,
            is_touching_boundary: left.is_touching_boundary || right.is_touching_boundary,
        };
        if lsize >= rsize {
            UnionResult::Left(result)
        } else {
            UnionResult::Right(result)
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.set_size
    }

}

impl Default for UnionNode {
    #[inline]
    fn default() -> UnionNode {
        UnionNode {
            set_size: 1,
            cardinality: 0,  // by default the cardinality is 0, set to 1 if needed
            is_touching_boundary: false,  // is already touching the boundary
        }
    }
}

impl FromIterator<UnionNode> for UnionFind {
    #[inline]
    fn from_iter<T: IntoIterator<Item = UnionNode>>(iterator: T) -> UnionFind {
        let mut uf = UnionFind {
            link_parent: vec![],
            payload: vec![],
        };
        uf.extend(iterator);
        uf
    }
}

impl Extend<UnionNode> for UnionFind {
    #[inline]
    fn extend<T: IntoIterator<Item = UnionNode>>(&mut self, iterable: T) {
        let len = self.payload.len();
        let payload = iterable.into_iter().map(Some);
        self.payload.extend(payload);

        let new_len = self.payload.len();
        self.link_parent.extend(len..new_len);
    }
}

impl UnionFind {
    #[inline]
    pub fn new(len: usize) -> Self {
        Self::from_iter((0..len).map(|_| Default::default()))
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.payload.len()
    }

    #[inline]
    pub fn insert(&mut self, data: UnionNode) -> usize {
        let key = self.payload.len();
        self.link_parent.push(key);
        self.payload.push(Some(data));
        key
    }

    #[inline]
    pub fn union(&mut self, key0: usize, key1: usize) -> bool {
        let k0 = self.find(key0);
        let k1 = self.find(key1);
        if k0 == k1 {
            return false;
        }

        // Temporary replace with dummy to move out the elements of the vector.
        let v0 = mem::replace(&mut self.payload[k0], None).unwrap();
        let v1 = mem::replace(&mut self.payload[k1], None).unwrap();

        let (parent, child, val) = match UnionNode::union(v0, v1) {
            UnionResult::Left(val) => (k0, k1, val),
            UnionResult::Right(val) => (k1, k0, val),
        };
        self.payload[parent] = Some(val);
        self.link_parent[child] = parent;

        true
    }

    #[inline]
    pub fn find(&mut self, key: usize) -> usize {
        let mut k = key;
        let mut p = self.link_parent[k];
        while p != k {
            let pp = self.link_parent[p];
            self.link_parent[k] = pp;
            k = p;
            p = pp;
        }
        k
    }

    #[inline]
    pub fn get(&mut self, key: usize) -> &UnionNode {
        let root_key = self.find(key);
        self.payload[root_key].as_ref().unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, key: usize) -> &mut UnionNode {
        let root_key = self.find(key);
        self.payload[root_key].as_mut().unwrap()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    // use `cargo test union_find_decoder_test_case_1 -- --nocapture` to run specific test
    
    fn make_standard_planar_code_2d_nodes_only_x_stabilizers(d: usize) -> (Vec<InputNode<(usize, usize)>>, HashMap<(usize, usize), usize>, Vec<NeighborEdge>) {
        make_standard_planar_code_2d_nodes(d, true)
    }

    #[test]
    fn union_find_decoder_test_basic_algorithm() {
        let mut uf = UnionFind::new(100);
        // test from https://github.com/gifnksm/union-find-rs/blob/master/src/tests.rs
        assert_eq!(1, uf.get(0).size());
        assert_eq!(1, uf.get(1).size());
        assert!(uf.find(0) != uf.find(1));
        assert!(uf.find(1) != uf.find(2));
        assert!(uf.union(0, 1));
        assert!(uf.find(0) == uf.find(1));
        assert_eq!(2, uf.get(0).size());
        assert_eq!(2, uf.get(1).size());
        assert_eq!(1, uf.get(2).size());
        assert!(!uf.union(0, 1));
        assert_eq!(2, uf.get(0).size());
        assert_eq!(2, uf.get(1).size());
        assert_eq!(1, uf.get(2).size());
        assert!(uf.union(1, 2));
        assert_eq!(3, uf.get(0).size());
        assert_eq!(3, uf.get(1).size());
        assert_eq!(3, uf.get(2).size());
        assert!(uf.find(0) == uf.find(1));
        assert!(uf.find(2) == uf.find(1));
        let k100 = uf.insert(UnionNode::default());
        assert_eq!(k100, 100);
        let _ = uf.union(k100, 0);
        assert_eq!(4, uf.get(100).size());
        assert_eq!(101, uf.size());
    }
    
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
        UnionFindDecoder::new(nodes, neighbors);
    }
    
    #[test]
    fn union_find_decoder_sanity_check_2() {
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(3);
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        let decoder = UnionFindDecoder::new(nodes, neighbors);
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
        let mut decoder = UnionFindDecoder::new(nodes, neighbors);
        decoder.run_to_stable();
        println!("decoder.nodes: {:#?}", decoder.nodes);
    }

}
