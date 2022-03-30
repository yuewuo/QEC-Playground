//! build complete model graph from model graph
//! 

use std::collections::{BTreeMap};
use std::sync::{Arc};
use serde::{Serialize};
use super::simulator::*;
use super::model_graph::*;

#[derive(Debug, Clone, Serialize)]
pub struct CompleteModelGraph {
    /// precomputed edges and active region helps to reduce the computation by stop searching unnecessarily unlikely edges
    /// , but need to be disabled when the probability of edges in model graph can change on the fly
    pub enable_precomputed: bool,
    /// each thread maintains a copy of this data structure to run Dijkstra's algorithm
    pub nodes: Vec::< Vec::< Vec::< Option< Box< CompleteModelGraphNode > > > > >,
    /// timestamp to invalidate all nodes without iterating them; only invalidating all nodes individually when active_timestamp is usize::MAX
    pub active_timestamp: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompleteModelGraphNode {
    /// precomputed data can help reduce runtime complexity, at the cost of more memory usage
    pub precomputed: Option<Arc<PrecomputedData>>,
    /// timestamp for Dijkstra's algorithm
    pub timestamp: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompleteModelGraphEdge {
    /// the weight of this edge
    /// , note that we don't keep `possibility` here because it might overflow given small `p` and long path
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrecomputedData {
    /// precomputed complete graph edges, if all edges are found, then no need to run Dijkstra's algorithm on the fly
    pub edges: BTreeMap<Position, CompleteModelGraphEdge>,
    /// precomputed active region, all connected
    pub active_region: ActiveRegion,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveRegion {
    pub t_min: usize,
    pub t_max: usize,
    pub i_min: usize,
    pub i_max: usize,
    pub j_min: usize,
    pub j_max: usize,
}

impl CompleteModelGraph {
    pub fn new(simulator: &Simulator, model_graph: &ModelGraph) -> Self {
        assert!(simulator.volume() > 0, "cannot build graph out of zero-sized simulator");
        Self {
            enable_precomputed: false,  // by default no precomputed data, disable precomputed fields
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if model_graph.is_node_exist(position) {
                            return Some(Box::new(CompleteModelGraphNode {
                                precomputed: None,  // by default no precomputed data
                                timestamp: 0,
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
    pub fn invalidate_previous_dijkstra(&mut self, simulator: &Simulator) {
        if self.active_timestamp == usize::MAX {  // rarely happens
            self.active_timestamp = 0;
            simulator_iter!(simulator, position, if self.is_node_exist(position) {
                let node = self.get_node_mut_unwrap(position);
                node.timestamp = 0;  // refresh all timestamps to avoid conflicts
            });
        }
        self.active_timestamp += 1;  // implicitly invalidate all nodes
    }

    pub fn run_dijkstra(&mut self, position: &Position, simulator: &Simulator, model_graph: &ModelGraph) {
        self.invalidate_previous_dijkstra(simulator);
        
    }

    pub fn precompute(&mut self, simulator: &Simulator, model_graph: &ModelGraph, precompute_complete_model_graph_max_size: usize) {
        if precompute_complete_model_graph_max_size == 0 {
            return  // no need to precompute
        }
        println!("precompute_complete_model_graph_max_size: {}", precompute_complete_model_graph_max_size);
        // iterate over each node to cache nearest nodes up to `precompute_complete_model_graph_max_size`
        simulator_iter!(simulator, position, if self.is_node_exist(position) {
            self.run_dijkstra(position, simulator, model_graph);
        });
    }
}
