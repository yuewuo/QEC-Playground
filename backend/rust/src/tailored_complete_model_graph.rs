//! build complete model graph from model graph
//! 

use std::collections::{BTreeMap};
use serde::{Serialize};
use super::simulator::*;
use super::tailored_model_graph::*;
use super::complete_model_graph::*;
use super::priority_queue::PriorityQueue;
use super::float_ord::FloatOrd;
use std::sync::{Arc};

#[derive(Debug, Clone, Serialize)]
pub struct TailoredCompleteModelGraph {
    /// precomputed edges and active region helps to reduce the runtime complexity by caching complete graph
    /// , but need to be disabled when the probability of edges in model graph can change on the fly
    pub precompute_complete_model_graph: bool,
    /// each thread maintains a copy of this data structure to run Dijkstra's algorithm
    pub nodes: Vec::< Vec::< Vec::< Option< Box< TripleCompleteTailoredModelGraphNode > > > > >,
    /// timestamp to invalidate all nodes without iterating them; only invalidating all nodes individually when active_timestamp is usize::MAX
    pub active_timestamp: usize,
}

/// precomputed data can help reduce runtime complexity, at the cost of more memory usage
#[derive(Debug, Clone, Serialize)]
pub struct CompleteTailoredModelGraphNode {
    /// precomputed data can help reduce runtime complexity, at the cost of more memory usage
    pub precomputed: Option<Arc<PrecomputedData>>,
    /// timestamp for Dijkstra's algorithm
    pub timestamp: usize,
    /// cache all edges currently needed to reconstruct path between interested pairs, will be cleared if timestamp is updated
    pub cache: BTreeMap<Position, CompleteTailoredModelGraphEdge>,
}

pub type TripleCompleteTailoredModelGraphNode = [CompleteTailoredModelGraphNode; 3];

#[derive(Debug, Clone, Serialize)]
pub struct CompleteTailoredModelGraphEdge {
    /// the next node to source back, it can also be itself, in which case this is the adjacent to boundary
    pub next: Position,
    /// the weight of this edge
    /// , note that we don't keep `possibility` here because it might overflow given small `p` and long path
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecomputedData {
    /// precomputed complete graph edges, if all edges are found and recorded, then no need to run Dijkstra's algorithm on the fly
    pub edges: BTreeMap<Position, CompleteTailoredModelGraphEdge>,
    /// precomputed complete graph edge to boundary
    pub boundary: Option<CompleteTailoredModelGraphEdge>,
}

impl TailoredCompleteModelGraph {
    pub fn new(simulator: &Simulator, model_graph: &TailoredModelGraph) -> Self {
        assert!(simulator.volume() > 0, "cannot build graph out of zero-sized simulator");
        Self {
            precompute_complete_model_graph: false,
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if model_graph.is_node_exist(position) {
                            return Some(Box::new([CompleteTailoredModelGraphNode {
                                precomputed: None,
                                timestamp: 0,
                                cache: BTreeMap::new(),
                            }, CompleteTailoredModelGraphNode {
                                precomputed: None,
                                timestamp: 0,
                                cache: BTreeMap::new(),
                            }, CompleteTailoredModelGraphNode {
                                precomputed: None,
                                timestamp: 0,
                                cache: BTreeMap::new(),
                            }]))
                        }
                        None
                    }).collect()
                }).collect()
            }).collect(),
            active_timestamp: 0,
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
        if self.precompute_complete_model_graph {
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
            [positive_edges, negative_edges]
        } else {
            unimplemented!();
        }
    }

    /// build correction with neutral matching, requires [`Self::get_neutral_matching_edges`] to be run before to cache the edges
    pub fn build_correction_neutral_matching(&self, source: &Position, target: &Position, tailored_model_graph: &TailoredModelGraph) -> SparseCorrection {
        if self.precompute_complete_model_graph {
            let mut correction = SparseCorrection::new();
            let mut source = source.clone();
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
            // only read from cache, to improve efficiency
            unimplemented!();
        }
    }

    /// run full Dijkstra's algorithm and identify the active region
    pub fn precompute_dijkstra(&mut self, position: &Position, model_graph: &TailoredModelGraph) {
        let active_timestamp = self.invalidate_previous_dijkstra();
        for idx in 0..3 {
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
                let model_graph_node = &model_graph.get_node_unwrap(&target)[idx];
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
                        let neighbor_node = &self.get_node_unwrap(neighbor)[idx];
                        if neighbor_node.timestamp != active_timestamp {
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
    pub fn precompute(&mut self, simulator: &Simulator, model_graph: &TailoredModelGraph, precompute_complete_model_graph: bool) {
        self.precompute_complete_model_graph = precompute_complete_model_graph;
        // clear existing state
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if self.is_node_exist(position) {
            let double_node = self.get_node_mut_unwrap(position);
            for idx in 0..3 {
                double_node[idx].precomputed = Some(Arc::new(PrecomputedData {
                    edges: BTreeMap::new(),
                    boundary: None,
                }));
            }
        });
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
