//! build complete model graph from model graph
//! 

use std::collections::{BTreeMap};
use serde::{Serialize};
use super::simulator::*;
use super::model_graph::*;
use super::priority_queue::PriorityQueue;
use super::float_ord::FloatOrd;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
pub struct CompleteModelGraph {
    /// precomputed edges and active region helps to reduce the runtime complexity by caching complete graph
    /// , but need to be disabled when the probability of edges in model graph can change on the fly
    pub precompute_complete_model_graph: bool,
    /// each thread maintains a copy of this data structure to run Dijkstra's algorithm
    pub nodes: Vec::< Vec::< Vec::< Option< Box< CompleteModelGraphNode > > > > >,
    /// timestamp to invalidate all nodes without iterating them; only invalidating all nodes individually when active_timestamp is usize::MAX
    pub active_timestamp: usize,
    /// optimization flag to remove edge if sum of boundary weights is greater than the path weight
    pub optimize_weight_greater_than_sum_boundary: bool,
    /// the model graph to build this complete model graph
    pub model_graph: Arc<ModelGraph>,
}

/// precomputed data can help reduce runtime complexity, at the cost of more memory usage
#[derive(Debug, Serialize)]
pub struct CompleteModelGraphNode {
    /// flag to duplicate [`PrecomputedData`] inside [`Self::precomputed`]
    duplicate_on_clone: bool,
    /// precomputed data can help reduce runtime complexity, at the cost of more memory usage
    pub precomputed: Option<Arc<PrecomputedData>>,
    /// timestamp for Dijkstra's algorithm
    pub timestamp: usize,
    /// previous value, invalidated along timestamp
    pub previous: Option<Arc<Position>>,
}

/// clone works a little bit different to copy PrecomputedData accordingly
impl Clone for CompleteModelGraphNode {
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

impl PrecomputedData {
    /// clear existing data for edges, to save memory
    pub fn clear_edges(&mut self) {
        self.edges.clear();
    }
}

impl CompleteModelGraph {
    pub fn new(simulator: &Simulator, model_graph: Arc<ModelGraph>) -> Self {
        assert!(simulator.volume() > 0, "cannot build graph out of zero-sized simulator");
        Self {
            precompute_complete_model_graph: false,
            nodes: (0..simulator.height).map(|t| {
                (0..simulator.vertical).map(|i| {
                    (0..simulator.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if model_graph.is_node_exist(position) {
                            return Some(Box::new(CompleteModelGraphNode {
                                duplicate_on_clone: true,  // default behavior, just clone for safe
                                precomputed: None,
                                timestamp: 0,
                                previous: None,
                            }))
                        }
                        None
                    }).collect()
                }).collect()
            }).collect(),
            active_timestamp: 0,
            optimize_weight_greater_than_sum_boundary: false,  // Yue 2022.7.22: fusion algorithm sometimes fail because of this flag: remove it
            model_graph: model_graph,
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

    /// get mutable model graph, will copy the model graph if it has more than one strong reference to it; remember to call `model_graph_changed` if the model graph is changed
    pub fn get_model_graph_mut(&'_ mut self) -> &'_ mut ModelGraph {
        match Arc::get_mut(&mut self.model_graph) {
            Some(_) => { },  // no other references exist
            None => {
                // the existing reference doesn't allow mutable reference to it, so we have to copy it
                let model_graph: ModelGraph = { (*Arc::clone(&self.model_graph)).clone() };
                self.model_graph = Arc::new(model_graph);
            },
        }
        Arc::get_mut(&mut self.model_graph).expect("the new copied model graph should be ok to have a mutable reference")
    }

    /// need to be called every time the model graph is changed
    pub fn model_graph_changed(&mut self, simulator: &Simulator) {
        self.find_shortest_boundary_paths(simulator);
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
    pub fn get_edges(&mut self, position: &Position, targets: &Vec<Position>) -> (Vec<(usize, f64)>, Option<f64>) {
        if !self.precompute_complete_model_graph {
            self.precompute_dijkstra(position);
        }
        let (edges, boundary) = {
            let mut edges = Vec::new();
            let node = self.get_node_unwrap(position);
            let precomputed = node.precomputed.as_ref().unwrap();
            for (index, target) in targets.iter().enumerate() {
                if let Some(edge) = precomputed.edges.get(target) {
                    edges.push((index, edge.weight));
                    // eprintln!("{:?} {:?}: {}", position, target, edge.weight);
                }
            }
            (edges, precomputed.boundary.as_ref().map(|boundary| boundary.weight))
        };
        if !self.precompute_complete_model_graph {
            Arc::get_mut(self.get_node_mut_unwrap(position).precomputed.as_mut().unwrap()).unwrap().clear_edges();  // free memory immediately
        }
        (edges, boundary)
    }

    /// build correction with matching
    pub fn build_correction_matching(&mut self, source: &Position, target: &Position) -> SparseCorrection {
        let model_graph = Arc::clone(&self.model_graph);
        let mut correction = SparseCorrection::new();
        let mut source = source.clone();
        if self.precompute_complete_model_graph {
            while &source != target {
                let node = self.get_node_unwrap(&source);
                let precomputed = node.precomputed.as_ref().unwrap();
                let target_edge = precomputed.edges.get(target);
                if target_edge.is_none() {
                    println!("target_edge none: source: {source:?}, target: {target:?}");
                }
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
            self.precompute_dijkstra_with_end_position(target, &source);
            // logic is different from what's happening if `precompute_complete_model_graph` is set
            while &source != target {
                let node = self.get_node_unwrap(&source);
                assert_eq!(node.timestamp, self.active_timestamp, "after running `precompute_dijkstra`, this node must be visited");
                let next: Position = (**(node.previous.as_ref().expect("must exist a path"))).clone();
                let model_graph_node = model_graph.get_node_unwrap(&source);
                let next_edge = model_graph_node.edges.get(&next);
                let next_correction = &next_edge.as_ref().unwrap().correction;
                correction.extend(next_correction);
                source = next;
            }
            Arc::get_mut(self.get_node_mut_unwrap(target).precomputed.as_mut().unwrap()).unwrap().clear_edges();  // free memory immediately
            correction
        }
    }

    /// build correction with boundary
    pub fn build_correction_boundary(&mut self, position: &Position) -> SparseCorrection {
        let model_graph = Arc::clone(&self.model_graph);
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
    }

    /// run full Dijkstra's algorithm and identify the active region
    pub fn precompute_dijkstra(&mut self, position: &Position) {
        self.precompute_dijkstra_with_end_position(position, &pos!(usize::MAX, usize::MAX, usize::MAX))
    }

    /// run full Dijkstra's algorithm and identify the active region, running [`Self::find_shortest_boundary_paths`] required before this function;
    /// terminate early if `end_position` is found
    pub fn precompute_dijkstra_with_end_position(&mut self, position: &Position, end_position: &Position) {
        let model_graph = Arc::clone(&self.model_graph);
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
                let mut add_entry = true;
                if self.optimize_weight_greater_than_sum_boundary && self.precompute_complete_model_graph {
                    add_entry = boundary_sum.is_none() || boundary_sum.unwrap() >= weight;
                }
                if add_entry {
                    let node = self.get_node_mut_unwrap(position);
                    Arc::get_mut(node.precomputed.as_mut().unwrap()).unwrap().edges.insert(target.clone(), CompleteModelGraphEdge {
                        next: next.clone(),
                        weight: weight,
                    });
                    if &target == end_position {
                        return  // early terminate
                    }
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
                        if !self.precompute_complete_model_graph {  // need to record `previous`
                            self.get_node_mut_unwrap(neighbor).previous = Some(Arc::new(target.clone()));
                            // eprintln!("position:{}, neighbor: {}, target: {}", position, neighbor, target);
                        }
                        pq.change_priority(neighbor, PriorityElement::new(edge_weight, next.clone()));
                    }
                } else {  // insert new entry only if neighbor has not been visited
                    let neighbor_node = self.get_node_unwrap(neighbor);
                    if neighbor_node.timestamp != active_timestamp {
                        if !self.precompute_complete_model_graph {  // need to record `previous`
                            self.get_node_mut_unwrap(neighbor).previous = Some(Arc::new(target.clone()));
                            // eprintln!("position:{}, neighbor: {}, target: {}", position, neighbor, target);
                        }
                        pq.push(neighbor.clone(), PriorityElement::new(edge_weight, next.clone()));
                    }
                }
            }
        }
        // eprintln!("edges: {:?}", self.get_node_unwrap(position).precomputed.as_ref().unwrap().edges);
    }

    /// update shortest boundary path to so that edges finding can terminate early
    pub fn find_shortest_boundary_paths(&mut self, simulator: &Simulator) {
        let model_graph = Arc::clone(&self.model_graph);
        let mut pq = PriorityQueue::<Position, PriorityElement>::new();
        // create initial priority queue and clear existing state (this function might be called multiple times on the fly)
        simulator_iter!(simulator, position, delta_t => simulator.measurement_cycles, if self.is_node_exist(position) {
            Arc::get_mut(self.get_node_mut_unwrap(&position).precomputed.as_mut().unwrap()).unwrap().boundary = None;
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
    pub fn precompute(&mut self, simulator: &Simulator, precompute_complete_model_graph: bool, parallel: usize) {
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
        self.find_shortest_boundary_paths(simulator);
        if precompute_complete_model_graph {
            // iterate over each node to cache nearest nodes up to `precompute_complete_model_graph`
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
                    node.precomputed = instance_node.precomputed.clone();
                    counter += 1;
                });
            }
            // it's safe to disable copying all complete graph edges
            for array in self.nodes.iter_mut() {
                for array in array.iter_mut() {
                    for element in array.iter_mut() {
                        match element {
                            Some(ref mut node) => {
                                node.duplicate_on_clone = false;
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
