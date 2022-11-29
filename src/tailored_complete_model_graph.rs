//! build complete model graph from model graph
//! 

use std::collections::{BTreeMap};
use serde::{Serialize};
use super::simulator::*;
use super::tailored_model_graph::*;
use super::complete_model_graph::*;
use super::priority_queue::PriorityQueue;
use super::float_ord::FloatOrd;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
pub struct TailoredCompleteModelGraph {
    /// precomputed edges and active region helps to reduce the runtime complexity by caching complete graph
    /// , but need to be disabled when the probability of edges in model graph can change on the fly
    pub precompute_complete_model_graph: bool,
    /// each thread maintains a copy of this data structure to run Dijkstra's algorithm
    pub nodes: Vec::< Vec::< Vec::< Option< Box< TripleCompleteTailoredModelGraphNode > > > > >,
    /// timestamp to invalidate all nodes without iterating them; only invalidating all nodes individually when active_timestamp is usize::MAX
    pub active_timestamp: usize,
    /// the tailored model graph to build this complete tailored model graph
    pub tailored_model_graph: Arc<TailoredModelGraph>,
}

/// precomputed data can help reduce runtime complexity, at the cost of more memory usage
#[derive(Debug, Serialize)]
pub struct CompleteTailoredModelGraphNode {
    /// flag to duplicate [`PrecomputedData`] inside [`Self::precomputed`]
    duplicate_on_clone: bool,
    /// precomputed data can help reduce runtime complexity, at the cost of more memory usage
    pub precomputed: Option<Arc<PrecomputedData>>,
    /// timestamp for Dijkstra's algorithm
    pub timestamp: usize,
    /// previous value, invalidated along timestamp
    pub previous: Option<Arc<Position>>,
}

impl Clone for CompleteTailoredModelGraphNode {
    fn clone(&self) -> Self {
        let mut result = Self {
            duplicate_on_clone: self.duplicate_on_clone,
            precomputed: self.precomputed.clone(),
            timestamp: self.timestamp,
            previous: None,
        };
        if self.duplicate_on_clone {
            if self.precomputed.is_some() {
                // allocate new memory to copy the precomputed data
                result.precomputed = Some(Arc::new((**self.precomputed.as_ref().unwrap()).clone()));
            }
        }
        result
    }
}

pub type TripleCompleteTailoredModelGraphNode = [CompleteTailoredModelGraphNode; 3];

#[derive(Debug, Clone, Serialize)]
pub struct CompleteTailoredModelGraphEdge {
    /// the next node to source back
    pub next: Position,
    /// the weight of this edge
    /// , note that we don't keep `possibility` here because it might overflow given small `p` and long path
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecomputedData {
    /// precomputed complete graph edges, if all edges are found and recorded, then no need to run Dijkstra's algorithm on the fly
    pub edges: BTreeMap<Position, CompleteTailoredModelGraphEdge>,
}

impl PrecomputedData {
    /// clear existing data for edges, to save memory
    pub fn clear_edges(&mut self) {
        self.edges.clear();
    }
}

impl TailoredCompleteModelGraph {
    pub fn new(simulator: &Simulator, tailored_model_graph: Arc<TailoredModelGraph>) -> Self {
        assert!(simulator.volume() > 0, "cannot build graph out of zero-sized simulator");
        Self {
            precompute_complete_model_graph: false,
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if tailored_model_graph.is_node_exist(position) {
                            let node = CompleteTailoredModelGraphNode {
                                duplicate_on_clone: true,  // default behavior, just clone for safe
                                precomputed: None,
                                timestamp: 0,
                                previous: None,
                            };
                            return Some(Box::new([node.clone(), node.clone(), node.clone()]))
                        }
                        None
                    }).collect()
                }).collect()
            }).collect(),
            active_timestamp: 0,
            tailored_model_graph: tailored_model_graph,
        }
    }

    /// any valid position of the simulator is a valid position in model graph, but only some of these positions corresponds a valid node in model graph
    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<Box<TripleCompleteTailoredModelGraphNode>> {
        &self.nodes[position.t][position.i][position.j]
    }

    /// check if a position contains model graph node
    pub fn is_node_exist(&self, position: &Position) -> bool {
        self.get_node(position).is_some()
    }

    /// get reference `self.nodes[t][i][j]` and then unwrap
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ TripleCompleteTailoredModelGraphNode {
        self.get_node(position).as_ref().unwrap()
    }

    /// get mutable reference `self.nodes[t][i][j]` and unwrap
    pub fn get_node_mut_unwrap(&'_ mut self, position: &Position) -> &'_ mut TripleCompleteTailoredModelGraphNode {
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
                            Some(ref mut double_node) => {
                                // refresh all timestamps to avoid conflicts
                                double_node[0].timestamp = 0;
                                double_node[1].timestamp = 0;
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

    /// get tailored matching edges in a batch manner to improve speed if need to run Dijkstra's algorithm on the fly;
    pub fn get_tailored_matching_edges(&mut self, position: &Position, targets: &Vec<Position>) -> [Vec<(usize, f64)>; 2] {
        if !self.precompute_complete_model_graph {
            self.precompute_dijkstra_subset(position, &mut [0,1].into_iter());
        }
        let [positive_node, negative_node, _neutral_node] = self.get_node_unwrap(position);
        // compute positive edges
        let mut positive_edges = Vec::new();
        let positive_precomputed = positive_node.precomputed.as_ref().unwrap();
        for (index, target) in targets.iter().enumerate() {
            if let Some(edge) = positive_precomputed.edges.get(target) {
                positive_edges.push((index, edge.weight));
            }
        }
        // compute negative edges
        let mut negative_edges = Vec::new();
        let negative_precomputed = negative_node.precomputed.as_ref().unwrap();
        for (index, target) in targets.iter().enumerate() {
            if let Some(edge) = negative_precomputed.edges.get(target) {
                negative_edges.push((index, edge.weight));
            }
        }
        if !self.precompute_complete_model_graph {
            for node in self.get_node_mut_unwrap(position) {
                Arc::get_mut(node.precomputed.as_mut().unwrap()).unwrap().clear_edges();  // free memory immediately
            }
        }
        [positive_edges, negative_edges]
    }

    /// get neutral matching edges in a batch manner to improve speed if need to run Dijkstra's algorithm on the fly;
    /// note that this will also include zero weight edges
    pub fn get_neutral_matching_edges(&mut self, position: &Position, targets: &Vec<Position>) -> Vec<(usize, f64)> {
        if !self.precompute_complete_model_graph {
            self.precompute_dijkstra_subset(position, &mut [2].into_iter());
        }
        let [_positive_node, _negative_node, neutral_node] = self.get_node_unwrap(position);
        // compute neutral edges
        let mut neutral_edges = Vec::new();
        let positive_precomputed = neutral_node.precomputed.as_ref().unwrap();
        for (index, target) in targets.iter().enumerate() {
            if let Some(edge) = positive_precomputed.edges.get(target) {
                neutral_edges.push((index, edge.weight));
            } else {
                if target == position {
                    neutral_edges.push((index, 0.));
                }
            }
        }
        if !self.precompute_complete_model_graph {
            for node in self.get_node_mut_unwrap(position) {
                Arc::get_mut(node.precomputed.as_mut().unwrap()).unwrap().clear_edges();  // free memory immediately
            }
        }
        neutral_edges
    }

    /// build correction with neutral matching
    pub fn build_correction_neutral_matching(&mut self, source: &Position, target: &Position) -> SparseCorrection {
        let tailored_model_graph = Arc::clone(&self.tailored_model_graph);
        let mut correction = SparseCorrection::new();
        let mut source = source.clone();
        if self.precompute_complete_model_graph {
            while &source != target {
                let [_, _, node] = self.get_node_unwrap(&source);
                let precomputed = node.precomputed.as_ref().unwrap();
                let target_edge = precomputed.edges.get(target);
                let edge = target_edge.as_ref().unwrap();
                let next = &edge.next;
                let [_, _, model_graph_node] = tailored_model_graph.get_node_unwrap(&source);
                let next_edge = model_graph_node.edges.get(next);
                let next_correction = &next_edge.as_ref().unwrap().correction;
                correction.extend(next_correction);
                source = next.clone();
            }
            correction
        } else {
            self.precompute_dijkstra_subset(target, &mut [2].into_iter());
            // logic is different from what's happening if `precompute_complete_model_graph` is set
            while &source != target {
                let [_, _, node] = self.get_node_unwrap(&source);
                assert_eq!(node.timestamp, self.active_timestamp, "after running `precompute_dijkstra`, this node must be visited");
                let next: Position = (**(node.previous.as_ref().expect("must exist a path"))).clone();
                let [_, _, model_graph_node] = tailored_model_graph.get_node_unwrap(&source);
                let next_edge = model_graph_node.edges.get(&next);
                let next_correction = &next_edge.as_ref().unwrap().correction;
                correction.extend(next_correction);
                source = next;
            }
            Arc::get_mut(self.get_node_mut_unwrap(target)[2].precomputed.as_mut().unwrap()).unwrap().clear_edges();  // free memory immediately
            correction
        }
    }

    /// run full Dijkstra's algorithm and identify the active region
    pub fn precompute_dijkstra(&mut self, position: &Position) {
        self.precompute_dijkstra_subset(position, &mut [0,1,2].into_iter())
    }

    /// run full Dijkstra's algorithm and identify the active region
    pub fn precompute_dijkstra_subset(&mut self, position: &Position, indices: &mut dyn Iterator<Item = usize>) {
        let tailored_model_graph = Arc::clone(&self.tailored_model_graph);
        let active_timestamp = self.invalidate_previous_dijkstra();
        for idx in indices {
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
                    let node = &self.get_node_unwrap(position)[idx];
                    !node.precomputed.as_ref().unwrap().edges.contains_key(&target)  // this entry shouldn't have been set
                });
                // update entry if size permits
                let node = &mut self.get_node_mut_unwrap(&target)[idx];
                node.timestamp = active_timestamp;  // mark as visited
                if &target != position {
                    let node = &mut self.get_node_mut_unwrap(position)[idx];
                    Arc::get_mut(node.precomputed.as_mut().unwrap()).unwrap().edges.insert(target.clone(), CompleteTailoredModelGraphEdge {
                        next: next.clone(),
                        weight: weight,
                    });
                }
                // add its neighbors to priority queue
                let tailored_model_graph_node = &tailored_model_graph.get_node_unwrap(&target)[idx];
                for (neighbor, edge) in tailored_model_graph_node.edges.iter() {
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
                            if !self.precompute_complete_model_graph {  // need to record `previous`
                                let node = &mut self.get_node_mut_unwrap(&neighbor)[idx];
                                node.previous = Some(Arc::new(target.clone()));
                                // eprintln!("position:{}, neighbor: {}, target: {}", position, neighbor, target);
                            }
                            pq.change_priority(neighbor, PriorityElement::new(edge_weight, next.clone()));
                        }
                    } else {  // insert new entry only if neighbor has not been visited
                        let neighbor_node = &self.get_node_unwrap(neighbor)[idx];
                        if neighbor_node.timestamp != active_timestamp {
                            if !self.precompute_complete_model_graph {  // need to record `previous`
                                let node = &mut self.get_node_mut_unwrap(&neighbor)[idx];
                                node.previous = Some(Arc::new(target.clone()));
                                // eprintln!("position:{}, neighbor: {}, target: {}", position, neighbor, target);
                            }
                            pq.push(neighbor.clone(), PriorityElement::new(edge_weight, next.clone()));
                        }
                    }
                }
            }
            // eprintln!("edges: {:?}", self.get_node_unwrap(position)[idx].precomputed.as_ref().unwrap().edges);
        }
    }

    /// precompute complete model graph if `precompute_complete_model_graph` is set
    #[inline(never)]
    pub fn precompute(&mut self, simulator: &Simulator, precompute_complete_model_graph: bool, parallel: usize) {
        self.precompute_complete_model_graph = precompute_complete_model_graph;
        // clear existing state
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if self.is_node_exist(position) {
            let double_node = self.get_node_mut_unwrap(position);
            for idx in 0..3 {
                double_node[idx].precomputed = Some(Arc::new(PrecomputedData {
                    edges: BTreeMap::new(),
                }));
            }
        });
        if precompute_complete_model_graph {
            if parallel <= 1 {
                simulator_iter!(simulator, position, if self.is_node_exist(position) {
                    self.precompute_dijkstra(position);
                });
            } else {
                // spawn `parallel` threads to compute in parallel
                let mut handlers = Vec::new();
                let mut instances = Vec::new();
                let shared_simulator = Arc::new(simulator.clone());
                for parallel_idx in 0..parallel {
                    let instance = Arc::new(Mutex::new(self.clone()));
                    let simulator = Arc::clone(&shared_simulator);
                    let thread_idx = parallel_idx;
                    instances.push(Arc::clone(&instance));
                    handlers.push(std::thread::spawn(move || {
                        let mut counter = 0;
                        let mut instance = instance.lock().unwrap();
                        simulator_iter!(simulator, position, if instance.is_node_exist(position) {
                            if counter % parallel == thread_idx {  // only compute my part of share
                                instance.precompute_dijkstra(position);
                            }
                            counter += 1;
                        });
                    }));
                }
                for handler in handlers.drain(..) {
                    handler.join().unwrap();
                }
                // move the data from instances (without additional large memory allocation)
                let mut counter = 0;
                simulator_iter!(simulator, position, if self.is_node_exist(position) {
                    let instance = &instances[counter % parallel];
                    let mut instance = instance.lock().unwrap();
                    let node = self.get_node_mut_unwrap(&position);
                    let instance_node = instance.get_node_mut_unwrap(&position);
                    for idx in 0..3 {
                        node[idx].precomputed = instance_node[idx].precomputed.clone();
                    }
                    counter += 1;
                });
            }
            // it's safe to disable copying all complete graph edges
            for array in self.nodes.iter_mut() {
                for array in array.iter_mut() {
                    for element in array.iter_mut() {
                        match element {
                            Some(ref mut tri_nodes) => {
                                for idx in 0..3 {
                                    tri_nodes[idx].duplicate_on_clone = false;
                                }
                            }
                            None => { }
                        }
                    }
                }
            }
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
                            let [positive_node, negative_node, neutral_node] = self.get_node_unwrap(position);
                            Some(json!({
                                "position": position,
                                "positive_precomputed": positive_node.precomputed,
                                "positive_timestamp": positive_node.timestamp,  // internal variable, export only when debug
                                "negative_precomputed": negative_node.precomputed,
                                "negative_timestamp": negative_node.timestamp,  // internal variable, export only when debug
                                "neutral_precomputed": neutral_node.precomputed,
                                "neutral_timestamp": neutral_node.timestamp,  // internal variable, export only when debug
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
