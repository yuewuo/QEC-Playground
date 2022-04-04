//! build complete model graph from model graph
//! 

use std::collections::{BTreeMap};
use serde::{Serialize};
use super::simulator::*;
use super::model_graph::*;
use super::priority_queue::PriorityQueue;
use super::float_ord::FloatOrd;
use std::sync::{Arc};

#[derive(Debug, Clone, Serialize)]
pub struct CompleteModelGraph {
    /// precomputed edges and active region helps to reduce the runtime complexity by caching complete graph
    /// , but need to be disabled when the probability of edges in model graph can change on the fly
    pub precompute_complete_model_graph: bool,
    /// each thread maintains a copy of this data structure to run Dijkstra's algorithm
    pub nodes: Vec::< Vec::< Vec::< Option< Box< CompleteModelGraphNode > > > > >,
    /// timestamp to invalidate all nodes without iterating them; only invalidating all nodes individually when active_timestamp is usize::MAX
    pub active_timestamp: usize,
}

/// precomputed data can help reduce runtime complexity, at the cost of more memory usage
#[derive(Debug, Clone, Serialize)]
pub struct CompleteModelGraphNode {
    /// precomputed data can help reduce runtime complexity, at the cost of more memory usage
    pub precomputed: Option<Arc<PrecomputedData>>,
    /// timestamp for Dijkstra's algorithm
    pub timestamp: usize,
    /// cache all edges currently needed to reconstruct path between interested pairs, will be cleared if timestamp is updated
    pub cache: BTreeMap<Position, CompleteModelGraphEdge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompleteModelGraphEdge {
    /// the next node to source back, it can also be itself, in which case this is the adjacent to boundary
    pub next: Position,
    /// the weight of this edge
    /// , note that we don't keep `possibility` here because it might overflow given small `p` and long path
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecomputedData {
    /// precomputed complete graph edges, if all edges are found and recorded, then no need to run Dijkstra's algorithm on the fly
    pub edges: BTreeMap<Position, CompleteModelGraphEdge>,
    /// precomputed complete graph edge to boundary
    pub boundary: Option<CompleteModelGraphEdge>,
}

impl CompleteModelGraph {
    pub fn new(simulator: &Simulator, model_graph: &ModelGraph) -> Self {
        assert!(simulator.volume() > 0, "cannot build graph out of zero-sized simulator");
        Self {
            precompute_complete_model_graph: false,
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if model_graph.is_node_exist(position) {
                            return Some(Box::new(CompleteModelGraphNode {
                                precomputed: None,
                                timestamp: 0,
                                cache: BTreeMap::new(),
                            }))
                        }
                        None
                    }).collect()
                }).collect()
            }).collect(),
            active_timestamp: 0,
        }
    }

    /// any valid position of the simulator is a valid position in model graph, but only some of these positions corresponds a valid node in model graph
    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<Box<CompleteModelGraphNode>> {
        &self.nodes[position.t][position.i][position.j]
    }

    /// check if a position contains model graph node
    pub fn is_node_exist(&self, position: &Position) -> bool {
        self.get_node(position).is_some()
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ CompleteModelGraphNode {
        self.get_node(position).as_ref().unwrap()
    }

    /// get mutable reference `self.nodes[t][i][j]` and unwrap
    pub fn get_node_mut_unwrap(&'_ mut self, position: &Position) -> &'_ mut CompleteModelGraphNode {
        self.nodes[position.t][position.i][position.j].as_mut().unwrap()
    }

    /// invalidate Dijkstra's algorithm state from previous call
    pub fn invalidate_previous_dijkstra(&mut self) -> usize {
        if self.active_timestamp == usize::MAX {  // rarely happens
            self.active_timestamp = 0;
            for array in self.nodes.iter_mut() {
                for array in array.iter_mut() {
                    for element in array.iter_mut() {
                        match element {
                            Some(ref mut node) => {
                                node.timestamp = 0;  // refresh all timestamps to avoid conflicts
                            }
                            None => { }
                        }
                    }
                }
            }
        }
        self.active_timestamp += 1;  // implicitly invalidate all nodes
        self.active_timestamp
    }

    /// compute the boundary sum given two positions
    pub fn get_boundary_sum(&self, position1: &Position, position2: &Position) -> Option<f64> {
        let node1 = self.get_node_unwrap(position1);
        if node1.precomputed.is_none() {
            return None;
        }
        if node1.precomputed.as_ref().unwrap().boundary.is_none() {
            return None;
        }
        let node2 = self.get_node_unwrap(position2);
        if node2.precomputed.is_none() {
            return None;
        }
        if node2.precomputed.as_ref().unwrap().boundary.is_none() {
            return None;
        }
        Some(node1.precomputed.as_ref().unwrap().boundary.as_ref().unwrap().weight + node2.precomputed.as_ref().unwrap().boundary.as_ref().unwrap().weight)
    }

    /// get edges in a batch manner to improve speed if need to run Dijkstra's algorithm on the fly;
    /// this function will clear any out-of-date cache
    pub fn get_edges(&mut self, position: &Position, targets: &Vec<Position>) -> (Vec<(usize, f64)>, Option<f64>) {
        if self.precompute_complete_model_graph {
            let mut edges = Vec::new();
            let node = self.get_node_unwrap(position);
            let precomputed = node.precomputed.as_ref().unwrap();

            for (index, target) in targets.iter().enumerate() {
                if let Some(edge) = precomputed.edges.get(target) {
                    edges.push((index, edge.weight));
                }
            }
            (edges, precomputed.boundary.as_ref().map(|boundary| boundary.weight))
        } else {
            unimplemented!();
        }
    }

    /// build correction with matching, requires [`Self::get_edges`] to be run before to cache the edges
    pub fn build_correction_matching(&self, source: &Position, target: &Position, model_graph: &ModelGraph) -> SparseCorrection {
        if self.precompute_complete_model_graph {
            let mut correction = SparseCorrection::new();
            let mut source = source.clone();
            while &source != target {
                let node = self.get_node_unwrap(&source);
                let precomputed = node.precomputed.as_ref().unwrap();
                let target_edge = precomputed.edges.get(target);
                let edge = target_edge.as_ref().unwrap();
                let next = &edge.next;
                let model_graph_node = model_graph.get_node_unwrap(&source);
                let next_edge = model_graph_node.edges.get(next);
                let next_correction = &next_edge.as_ref().unwrap().correction;
                correction.extend(next_correction);
                source = next.clone();
            }
            correction
        } else {
            // only read from cache, to improve efficiency
            unimplemented!();
        }
    }

    /// build correction with boundary, requires [`Self::get_edges`] to be run before to cache the edges
    pub fn build_correction_boundary(&self, position: &Position, model_graph: &ModelGraph) -> SparseCorrection {
        if self.precompute_complete_model_graph {
            let mut correction = SparseCorrection::new();
            let mut position = position.clone();
            loop {
                let node = self.get_node_unwrap(&position);
                let precomputed = node.precomputed.as_ref().unwrap();
                let boundary = precomputed.boundary.as_ref().unwrap();
                let next = &boundary.next;
                let model_graph_node = model_graph.get_node_unwrap(&position);
                if next == &position {
                    // this is the boundary
                    let boundary_correction = &model_graph_node.boundary.as_ref().unwrap().correction;
                    correction.extend(boundary_correction);
                    break
                } else {
                    let next_edge = model_graph_node.edges.get(next);
                    let next_correction = &next_edge.as_ref().unwrap().correction;
                    correction.extend(next_correction);
                    position = next.clone();
                }
            }
            correction
        } else {
            // only read from cache, to improve efficiency
            unimplemented!();
        }
    }

    /// run full Dijkstra's algorithm and identify the active region, running [`Self::find_shortest_boundary_paths`] required before this function
    pub fn precompute_dijkstra(&mut self, position: &Position, model_graph: &ModelGraph) {
        let active_timestamp = self.invalidate_previous_dijkstra();
        let mut pq = PriorityQueue::<Position, PriorityElement>::new();
        pq.push(position.clone(), PriorityElement::new(0., position.clone()));
        loop {  // until no more elements
            if pq.len() == 0 {
                break
            }
            let (target, PriorityElement { weight: FloatOrd(weight), mut next }) = pq.pop().unwrap();
            if &next == position {
                next = target.clone();  // this target is adjacent to itself, so previous set to this target
            }
            // eprintln!("target: {}, weight: {}, next: {}", target, weight, next);
            debug_assert!({
                let node = self.get_node_unwrap(position);
                !node.precomputed.as_ref().unwrap().edges.contains_key(&target)  // this entry shouldn't have been set
            });
            // update entry if size permits
            let node = self.get_node_mut_unwrap(&target);
            node.timestamp = active_timestamp;  // mark as visited
            if &target != position {
                let boundary_sum = self.get_boundary_sum(position, &target);
                if boundary_sum.is_none() || boundary_sum.unwrap() >= weight {
                    let node = self.get_node_mut_unwrap(position);
                    Arc::get_mut(node.precomputed.as_mut().unwrap()).unwrap().edges.insert(target.clone(), CompleteModelGraphEdge {
                        next: next.clone(),
                        weight: weight,
                    });
                }
            }
            // add its neighbors to priority queue
            let model_graph_node = model_graph.get_node_unwrap(&target);
            for (neighbor, edge) in model_graph_node.edges.iter() {
                let edge_weight = weight + edge.weight;
                if let Some(PriorityElement { weight: FloatOrd(existing_weight), next: existing_next }) = pq.get_priority(neighbor) {
                    // update the priority if weight is smaller or weight is equal but distance is smaller
                    // this is necessary if the graph has weight-0 edges, which could lead to cycles in the graph and cause deadlock
                    let mut update = &edge_weight < existing_weight;
                    if &edge_weight == existing_weight {
                        let distance = target.distance(&next);
                        let existing_distance = target.distance(&existing_next);
                        // prevent loop by enforcing strong non-descending
                        if distance < existing_distance || (distance == existing_distance && &next < existing_next) {
                            update = true;
                        }
                    }
                    if update {
                        pq.change_priority(neighbor, PriorityElement::new(edge_weight, next.clone()));
                    }
                } else {  // insert new entry only if neighbor has not been visited
                    let neighbor_node = self.get_node_unwrap(neighbor);
                    if neighbor_node.timestamp != active_timestamp {
                        pq.push(neighbor.clone(), PriorityElement::new(edge_weight, next.clone()));
                    }
                }
            }
        }
        // eprintln!("edges: {:?}", self.get_node_unwrap(position).precomputed.as_ref().unwrap().edges);
    }

    /// update shortest boundary path to so that edges finding can terminate early
    pub fn find_shortest_boundary_paths(&mut self, simulator: &Simulator, model_graph: &ModelGraph) {
        let mut pq = PriorityQueue::<Position, PriorityElement>::new();
        // create initial priority queue
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if self.is_node_exist(position) {
            let model_graph_node = model_graph.get_node_unwrap(position);
            if let Some(boundary) = &model_graph_node.boundary {
                pq.push(position.clone(), PriorityElement::new(boundary.weight, position.clone()));
            }
        });
        loop {  // until no more elements
            if pq.len() == 0 {
                break
            }
            let (position, PriorityElement { weight: FloatOrd(weight), next }) = pq.pop().unwrap();
            // eprintln!("position: {}, weight: {}, next: {}", position, weight, next);
            debug_assert!({
                let node = self.get_node_unwrap(&position);
                node.precomputed.as_ref().unwrap().boundary.is_none()  // this place shouldn't have been set
            });
            // update boundary
            let node = self.get_node_mut_unwrap(&position);
            Arc::get_mut(node.precomputed.as_mut().unwrap()).unwrap().boundary = Some(CompleteModelGraphEdge {
                next: next,
                weight: weight,
            });
            // add its neighbors to priority queue
            let model_graph_node = model_graph.get_node_unwrap(&position);
            for (neighbor, edge) in model_graph_node.edges.iter() {
                let edge_weight = weight + edge.weight;
                if let Some(PriorityElement { weight: FloatOrd(existing_weight), .. }) = pq.get_priority(neighbor) {
                    if &edge_weight < existing_weight {  // update the priority
                        pq.change_priority(neighbor, PriorityElement::new(edge_weight, position.clone()));
                    }
                } else {  // insert new entry only if neighbor has not been visited
                    let neighbor_node = self.get_node_unwrap(neighbor);
                    if neighbor_node.precomputed.as_ref().unwrap().boundary.is_none() {
                        pq.push(neighbor.clone(), PriorityElement::new(edge_weight, position.clone()));
                    }
                }
            }
        }
    }

    /// precompute complete model graph if `precompute_complete_model_graph` is set
    #[inline(never)]
    pub fn precompute(&mut self, simulator: &Simulator, model_graph: &ModelGraph, precompute_complete_model_graph: bool) {
        self.precompute_complete_model_graph = precompute_complete_model_graph;
        // clear existing state
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if self.is_node_exist(position) {
            let node = self.get_node_mut_unwrap(position);
            node.precomputed = Some(Arc::new(PrecomputedData {
                edges: BTreeMap::new(),
                boundary: None,
            }));
        });
        // find the shortest path to boundaries, this will help reduce the number of steps later
        self.find_shortest_boundary_paths(simulator, model_graph);
        if precompute_complete_model_graph {
            // iterate over each node to cache nearest nodes up to `precompute_complete_model_graph`
            simulator_iter!(simulator, position, if self.is_node_exist(position) {
                self.precompute_dijkstra(position, model_graph);
            });
        }
    }

    pub fn to_json(&self, simulator: &Simulator) -> serde_json::Value {
        json!({
            "code_type": simulator.code_type,
            "height": simulator.height,
            "vertical": simulator.vertical,
            "horizontal": simulator.horizontal,
            "precompute_complete_model_graph": self.precompute_complete_model_graph,
            "active_timestamp": self.active_timestamp,  // internal variable, export only when debug
            "nodes": (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if self.is_node_exist(position) {
                            let node = self.get_node_unwrap(position);
                            Some(json!({
                                "position": position,
                                "precomputed": node.precomputed,
                                "timestamp": node.timestamp,  // internal variable, export only when debug
                            }))
                        } else {
                            None
                        }
                    }).collect::<Vec<Option<serde_json::Value>>>()
                }).collect::<Vec<Vec<Option<serde_json::Value>>>>()
            }).collect::<Vec<Vec<Vec<Option<serde_json::Value>>>>>()
        })
    }
}

#[derive(Eq, Debug)]
pub struct PriorityElement {
    pub weight: FloatOrd<f64>,
    pub next: Position,
}

impl std::cmp::PartialEq for PriorityElement {
    #[inline]
    fn eq(&self, other: &PriorityElement) -> bool {
        self.weight == other.weight
    }
}

impl std::cmp::PartialOrd for PriorityElement {
    #[inline]
    fn partial_cmp(&self, other: &PriorityElement) -> Option<std::cmp::Ordering> {
        other.weight.partial_cmp(&self.weight)  // reverse `self` and `other` to prioritize smaller weight
    }
}

impl std::cmp::Ord for PriorityElement {
    #[inline]
    fn cmp(&self, other: &PriorityElement) -> std::cmp::Ordering {
        other.weight.cmp(&self.weight)  // reverse `self` and `other` to prioritize smaller weight
    }
}

impl PriorityElement {
    pub fn new(weight: f64, next: Position) -> Self {
        Self {
            weight: FloatOrd::<f64>(weight),
            next: next,
        }
    }
}
