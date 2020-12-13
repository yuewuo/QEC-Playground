//! # Fault Tolerant Quantum Error Correction Module
//!
//! (This module corresponds to `FaultTolerantView.vue` in frontend)
//!
//! ## Error Model
//!
//! It has some helper functions to build runnable error model and to generate random errors based on the error model. 
//! It supports both standard planar code and rotated planar code. 
//!
//! ## Decoder Implementation
//! 
//! In order to maximize decoder performance, we compute static information (like graph structure and weights) beforehand. 
//! The decoder accepts these auxiliary information, which can be generated using the functions in `Error Model`
//!


#![allow(non_snake_case)]

use super::ndarray;
use super::petgraph;
use std::collections::HashMap;

/// uniquely index a node
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Index {
    pub t: usize,
    pub i: usize,
    pub j: usize,
}

/// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
#[derive(Debug)]
pub struct Node {
    pub t: usize,
    pub i: usize,
    pub j: usize,
    pub connection: Option<Connection>,
    pub gate_type: GateType,
    pub qubit_type: QubitType,
    pub error: ErrorType,
    pub error_rate_x: f64,
    pub error_rate_z: f64,
    pub error_rate_y: f64,
    pub propagated: ErrorType,
    // graph information
    pub edges: Vec::<Edge>,
    pub boundary: Option<Boundary>,  // if connects to boundary in graph, this is the probability
    pub pet_node: Option<petgraph::graph::NodeIndex>,
    pub exhausted_map: HashMap<Index, ExhaustedElement>,
}

/// The structure of surface code, including how quantum gates are implemented
#[derive(Debug)]
pub struct PlanarCodeModel {
    /// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
    pub snapshot: Vec::< Vec::< Vec::< Option<Node> > > >,
    pub L: usize,
    pub T: usize,
    pub graph: Option<petgraph::graph::Graph<Index, PetGraphEdge>>,
}

impl PlanarCodeModel {
    pub fn new_standard_planar_code(T: usize, L: usize) -> Self {
        Self::new_planar_code(T, L, |_i, _j| true)
    }
    pub fn new_planar_code<F>(T: usize, L: usize, filter: F) -> Self
            where F: Fn(usize, usize) -> bool {
        let width = 2 * L - 1;
        let height = T * 6 + 1;
        let mut snapshot = Vec::with_capacity(height);
        for t in 0..height {
            let mut snapshot_row_0 = Vec::with_capacity(width);
            for i in 0..width {
                let mut snapshot_row_1 = Vec::with_capacity(width);
                for j in 0..width {
                    if filter(i, j) {
                        let stage = Stage::from(t);
                        let qubit_type = if (i + j) % 2 == 0 { QubitType::Data } else { if i % 2 == 0 { QubitType::StabZ } else { QubitType::StabX } };
                        let mut gate_type = GateType::None;
                        let mut connection = None;
                        match stage {
                            Stage::Initialization => {
                                if qubit_type != QubitType::Data {
                                    gate_type = GateType::Initialization;
                                }
                            },
                            Stage::CXGate1 => {
                                if qubit_type == QubitType::Data {
                                    if i+1 < width && filter(i+1, j) {
                                        gate_type = if j % 2 == 0 { GateType::Target } else { GateType::Control };
                                        connection = Some(Connection{ t: t, i: i+1, j: j });
                                    }
                                } else {
                                    if i >= 1 && filter(i-1, j) {
                                        gate_type = if j % 2 == 0 { GateType::Control } else { GateType::Target };
                                        connection = Some(Connection{ t: t, i: i-1, j: j });
                                    }
                                }
                            },
                            Stage::CXGate2 => {
                                if qubit_type == QubitType::Data {
                                    if j+1 < width && filter(i, j+1) {
                                        gate_type = if i % 2 == 0 { GateType::Control} else { GateType::Target};
                                        connection = Some(Connection{ t: t, i: i, j: j+1 });
                                    }
                                } else {
                                    if j >= 1 && filter(i, j-1) {
                                        gate_type = if i % 2 == 0 { GateType::Target} else { GateType::Control};
                                        connection = Some(Connection{ t: t, i: i, j: j-1 });
                                    }
                                }
                            },
                            Stage::CXGate3 => {
                                if qubit_type == QubitType::Data {
                                    if j >= 1 && filter(i, j-1) {
                                        gate_type = if i % 2 == 0 { GateType::Control} else { GateType::Target};
                                        connection = Some(Connection{ t: t, i: i, j: j-1 });
                                    }
                                } else {
                                    if j+1 < width && filter(i, j+1) {
                                        gate_type = if i % 2 == 0 { GateType::Target} else { GateType::Control};
                                        connection = Some(Connection{ t: t, i: i, j: j+1 });
                                    }
                                }
                            },
                            Stage::CXGate4 => {
                                if qubit_type == QubitType::Data {
                                    if i >= 1 && filter(i-1, j) {
                                        gate_type = if j % 2 == 0 { GateType::Target} else { GateType::Control};
                                        connection = Some(Connection{ t: t, i: i-1, j: j });
                                    }
                                } else {
                                    if i+1 < width && filter(i+1, j) {
                                        gate_type = if j % 2 == 0 { GateType::Control} else { GateType::Target};
                                        connection = Some(Connection{ t: t, i: i+1, j: j });
                                    }
                                }
                            },
                            Stage::Measurement => {
                                if qubit_type != QubitType::Data {
                                    gate_type = GateType::Measurement;
                                }
                            },
                        }
                        snapshot_row_1.push(Some(Node{
                            t: t, i: i, j: j,
                            connection: connection,
                            gate_type: gate_type,
                            qubit_type: qubit_type,
                            error: ErrorType::I,
                            error_rate_x: 0.25,  // by default error rate is the highest
                            error_rate_z: 0.25,
                            error_rate_y: 0.25,
                            propagated: ErrorType::I,
                            edges: Vec::new(),
                            boundary: None,
                            pet_node: None,
                            exhausted_map: HashMap::new(),
                        }))
                    } else {
                        snapshot_row_1.push(None);
                    }
                }
                snapshot_row_0.push(snapshot_row_1);
            }
            snapshot.push(snapshot_row_0);
        }
        Self {
            snapshot: snapshot,
            L: L,
            T: T,
            graph: None,
        }
    }
    pub fn iterate_snapshot_mut<F>(&mut self, mut func: F) where F: FnMut(usize, usize, usize, &mut Node) {
        for (t, array) in self.snapshot.iter_mut().enumerate() {
            for (i, array) in array.iter_mut().enumerate() {
                for (j, element) in array.iter_mut().enumerate() {
                    match element {
                        Some(ref mut e) => { func(t, i, j, e); }
                        None => { }
                    }
                }
            }
        }
    }
    pub fn iterate_snapshot<F>(&self, mut func: F) where F: FnMut(usize, usize, usize, &Node) {
        for (t, array) in self.snapshot.iter().enumerate() {
            for (i, array) in array.iter().enumerate() {
                for (j, element) in array.iter().enumerate() {
                    match element {
                        Some(ref e) => { func(t, i, j, e); }
                        None => { }
                    }
                }
            }
        }
    }
    pub fn set_depolarizing_error(&mut self, error_rate: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p(1-p)
        self.iterate_snapshot_mut(|_t, _i, _j, node| {
            node.error_rate_x = error_rate;
            node.error_rate_z = error_rate;
            node.error_rate_y = error_rate;
        })
    }
    pub fn clear_error(&mut self) {
        self.iterate_snapshot_mut(|_t, _i, _j, node| {
            node.error = ErrorType::I;
        })
    }
    pub fn count_nodes(&self) -> usize {
        let mut count = 0;
        self.iterate_snapshot(|_t, _i, _j, _node| {
            count += 1;
        });
        count
    }
    /// generate random error based on `error_rate` in each node.
    pub fn generate_random_errors<F>(&mut self, mut rng: F) -> usize where F: FnMut() -> f64 {
        let mut error_count = 0;
        self.iterate_snapshot_mut(|_t, _i, _j, node| {
            let random_number = rng();
            if random_number < node.error_rate_x {
                node.error = ErrorType::X;
                error_count += 1;
            } else if random_number < node.error_rate_x + node.error_rate_z {
                node.error = ErrorType::Z;
                error_count += 1;
            } else if random_number < node.error_rate_x + node.error_rate_z + node.error_rate_y {
                node.error = ErrorType::Y;
                error_count += 1;
            } else {
                node.error = ErrorType::I;
            }
        });
        error_count
    }
    pub fn count_error(&self) -> usize {
        let mut count = 0;
        self.iterate_snapshot(|_t, _i, _j, node| {
            if node.error != ErrorType::I {
                count += 1;
            }
        });
        count
    }
    pub fn add_error_at(&mut self, t: usize, i: usize, j: usize, error: &ErrorType) -> Option<ErrorType> {
        if let Some(array) = self.snapshot.get_mut(t) {
            if let Some(array) = array.get_mut(i) {
                if let Some(element) = array.get_mut(j) {
                    match element {
                        Some(ref mut node) => {
                            node.error = node.error.multiply(error);
                            Some(node.error.clone())
                        }
                        None => None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    /// update `propagated` of each error node,
    pub fn propagate_error(&mut self) {
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            if t != 0 {  // will not change the propagated error from the lowest layer
                node.propagated = ErrorType::I;
            }
        });
        for t in 0..self.snapshot.len() - 1 {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                        // error will definitely propagated to itself at t+1
                        let node_propagated = node.propagated.clone();
                        let node_connection = node.connection.clone();
                        let direct_error = node.error.multiply(&node_propagated);
                        let gate_type = node.gate_type.clone();
                        let next_node = self.snapshot[t+1][i][j].as_mut().expect("exist");
                        next_node.propagated = direct_error.multiply(&next_node.propagated);
                        if gate_type == GateType::Initialization {
                            next_node.propagated = ErrorType::I;  // no error after initialization
                        }
                        // but sometimes it also propagated to other qubits through CX gate
                        if gate_type == GateType::Control {
                            let connection = node_connection.expect("exist");
                            if node_propagated == ErrorType::X || node_propagated == ErrorType::Y {
                                let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                                peer_node.propagated = peer_node.propagated.multiply(&ErrorType::X);
                            }
                        } else if gate_type == GateType::Target {
                            let connection = node_connection.expect("exist");
                            if node_propagated == ErrorType::Z || node_propagated == ErrorType::Y {
                                let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                                peer_node.propagated = peer_node.propagated.multiply(&ErrorType::Z);
                            }
                        }
                    }
                }
            }
        }
    }
    /// iterate over every measurement errors
    pub fn iterate_measurement_stabilizers_mut<F>(&mut self, mut func: F) where F: FnMut(usize, usize, usize, &mut Node, QubitType) {
        for t in (6..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    let node = self.snapshot[t][i][j].as_ref().expect("exist");
                    let qubit_type = node.qubit_type.clone();
                    if qubit_type == QubitType::StabZ || qubit_type == QubitType::StabX {
                        assert_eq!(node.gate_type, GateType::Measurement);
                        func(t, i, j, self.snapshot[t][i][j].as_mut().expect("exist"), qubit_type);
                    }
                }
            }
        }
    }
    /// iterate over every measurement errors
    pub fn iterate_measurement_stabilizers<F>(&self, mut func: F) where F: FnMut(usize, usize, usize, &Node, QubitType) {
        for t in (6..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    let node = self.snapshot[t][i][j].as_ref().expect("exist");
                    let qubit_type = node.qubit_type.clone();
                    if qubit_type == QubitType::StabZ || qubit_type == QubitType::StabX {
                        assert_eq!(node.gate_type, GateType::Measurement);
                        func(t, i, j, self.snapshot[t][i][j].as_ref().expect("exist"), qubit_type);
                    }
                }
            }
        }
    }
    /// iterate over every measurement errors
    pub fn iterate_measurement_errors<F>(&self, mut func: F) where F: FnMut(usize, usize, usize, &Node, QubitType) {
        for t in (6..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    let node = self.snapshot[t][i][j].as_ref().expect("exist");
                    if node.qubit_type == QubitType::StabZ {
                        assert_eq!(node.gate_type, GateType::Measurement);
                        let this_result = node.propagated == ErrorType::I || node.propagated == ErrorType::Z;
                        let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                        let last_result = last_node.propagated == ErrorType::I || last_node.propagated == ErrorType::Z;
                        if this_result != last_result {
                            func(t, i, j, self.snapshot[t][i][j].as_ref().expect("exist"), QubitType::StabZ);
                        }
                    } else if node.qubit_type == QubitType::StabX {
                        assert_eq!(node.gate_type, GateType::Measurement);
                        let this_result = node.propagated == ErrorType::I || node.propagated == ErrorType::X;
                        let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                        let last_result = last_node.propagated == ErrorType::I || last_node.propagated == ErrorType::X;
                        if this_result != last_result {
                            func(t, i, j, self.snapshot[t][i][j].as_ref().expect("exist"), QubitType::StabX);
                        }
                    }
                }
            }
        }
    }
    /// get data qubit error pattern based on current `propagated` error on t=6,12,18,...
    pub fn get_data_qubit_error_pattern(&self) -> Correction {
        let width = 2 * self.L - 1;
        let mut x = ndarray::Array::from_elem((self.T, width, width), false);
        let mut x_mut = x.view_mut();
        let mut z = ndarray::Array::from_elem((self.T, width, width), false);
        let mut z_mut = z.view_mut();
        for (idx, t) in (6..self.snapshot.len()).step_by(6).enumerate() {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    let node = self.snapshot[t][i][j].as_ref().expect("exist");
                    if node.qubit_type == QubitType::Data {
                        match node.propagated {
                            ErrorType::X => { x_mut[[idx, i, j]] = true; }
                            ErrorType::Z => { z_mut[[idx, i, j]] = true; }
                            ErrorType::Y => { x_mut[[idx, i, j]] = true; z_mut[[idx, i, j]] = true; }
                            ErrorType::I => { }
                        }
                    }
                }
            }
        }
        Correction {
            x: x,
            z: z,
        }
    }
    /// corresponds to `build_graph_given_error_rate` in `FaultTolerantView.vue`
    pub fn build_graph(&mut self) {
        for t in 0..self.snapshot.len() {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        for error in [ErrorType::X, ErrorType::Z].iter() {
                            self.clear_error();
                            let node = self.snapshot[t][i][j].as_ref().expect("exist");
                            let p = if *error == ErrorType::X {
                                node.error_rate_x + node.error_rate_y
                            } else {
                                node.error_rate_z + node.error_rate_y
                            };  // probability of this error to occur
                            // simulate the error and measure it
                            self.add_error_at(t, i, j, error);
                            self.propagate_error();
                            let mut measurement_errors = Vec::new();
                            self.iterate_measurement_errors(|t, i, j, _node, _qubit_type| {
                                measurement_errors.push((t, i, j));
                            });
                            if measurement_errors.len() == 0 {  // no way to detect it, ignore
                                continue
                            }
                            assert!(measurement_errors.len() <= 2, "single qubit error should not cause more than 2 measurement errors");
                            // compute correction pattern, so that applying this error pattern will exactly recover data qubit errors
                            let correction = self.get_data_qubit_error_pattern();
                            // add this to edges and update probability
                            if measurement_errors.len() == 1 {  // boundary
                                let (t1, i1, j1) = measurement_errors[0];
                                let node = self.snapshot[t1][i1][j1].as_mut().expect("exist");
                                if node.boundary.is_none() {
                                    node.boundary = Some(Boundary {
                                        p: 0.,
                                        cases: Vec::new(),
                                    });
                                }
                                node.boundary.as_mut().expect("exist").add(p, correction);
                            } else if measurement_errors.len() == 2 {  // connection
                                let (t1, i1, j1) = measurement_errors[0];
                                let (t2, i2, j2) = measurement_errors[1];
                                add_edge_case(&mut self.snapshot[t1][i1][j1].as_mut().expect("exist").edges,
                                    t2, i2, j2, p, correction.clone());
                                add_edge_case(&mut self.snapshot[t2][i2][j2].as_mut().expect("exist").edges,
                                    t1, i1, j1, p, correction);
                            }
                        }
                    }
                }
            }
        }
        // reset graph to state without error
        self.clear_error();
        self.propagate_error();
    }
    /// exhaustively search the minimum path from every measurement stabilizer to the others.
    /// Running `build_graph` required before running this function.
    pub fn build_exhausted_path(&mut self) {
        // first build petgraph
        let mut graph = petgraph::graph::Graph::new_undirected();
        // add nodes before adding edge, so that they all have node number
        self.iterate_measurement_stabilizers_mut(|t, i, j, node, _qubit_type| {
            node.pet_node = Some(graph.add_node(Index {
                t: t, i: i, j: j
            }));
        });
        // then add every edge
        self.iterate_measurement_stabilizers(|t, i, j, node, _qubit_type| {
            for edge in &node.edges {
                let node_target = self.snapshot[edge.t][edge.i][edge.j].as_ref().expect("exist").pet_node.expect("exist");
                graph.add_edge(node.pet_node.expect("exist"), node_target, PetGraphEdge {
                    a: Index { t: t, i: i, j: j },
                    b: Index { t: edge.t, i: edge.i, j: edge.j },
                    weight: - edge.p.ln(),  // so that w1 + w2 = - log(p1) - log(p2) = - log(p1*p2) = - log(p_line)
                    // we want p_line to be as large as possible, it meets the goal of minimizing -log(p) 
                });
            }
        });
        // then run dijkstra for every node
        self.iterate_measurement_stabilizers_mut(|t, i, j, node, _qubit_type| {
            let map = petgraph::algo::dijkstra(&graph, node.pet_node.expect("exist"), None, |e| e.weight().weight);
            for (node_id, cost) in map.iter() {
                let index = graph.node_weight(*node_id).expect("exist");
                if index != &(Index{ t: t, i: i, j: j }) { // do not add map to itself
                    node.exhausted_map.insert(Index {
                        t: index.t,
                        i: index.i,
                        j: index.j,
                    }, ExhaustedElement {
                        cost: *cost,
                        next: None,
                        correction: None,
                    });
                }
            }
        });
        // TODO: use the result of dijkstra to build `next`, so that the shortest path is found is O(1) time

        // TODO: use the shortest path to build `correction`, so that correction is done in O(1) time
        
    }
}

/// Stage is determined by time t
#[derive(Debug, PartialEq)]
pub enum Stage {
    Initialization,
    CXGate1,
    CXGate2,
    CXGate3,
    CXGate4,
    Measurement,
}

impl From<usize> for Stage {
    fn from(t: usize) -> Self {
        match (t + 6 - 1) % 6 {  // add bias so that layer t=0 is measurement, like in `FaultTolerantView.vue`
            0 => Self::Initialization,
            1 => Self::CXGate1,
            2 => Self::CXGate2,
            3 => Self::CXGate3,
            4 => Self::CXGate4,
            5 => Self::Measurement,
            _ => panic!("why would usize % 6 >= 6 ?"),
        }
    }
}

/// Qubit type, corresponds to `QTYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone)]
pub enum QubitType {
    Data,
    StabX,
    StabZ,
}

/// Gate type, corresponds to `NTYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone)]
pub enum GateType {
    Initialization,
    Control,
    Target,
    Measurement,
    None,  // do nothing
}

/// Connection Information, corresponds to `connection` in `FaultTolerantView.vue`
#[derive(Debug, Clone)]
pub struct Connection {
    pub t: usize,
    pub i: usize,
    pub j: usize,
}

/// Edge Information, corresponds to `node.edges` in `FaultTolerantView.vue`
#[derive(Debug, Clone)]
pub struct Edge {
    pub t: usize,
    pub i: usize,
    pub j: usize,
    pub p: f64,
    pub cases: Vec::<(Correction, f64)>,
}

pub fn add_edge_case(edges: &mut Vec::<Edge>, t: usize, i: usize, j: usize, p: f64, correction: Correction) {
    for edge in edges.iter_mut() {
        if edge.t == t && edge.i == i && edge.j == j {
            edge.add(p, correction);
            return  // already found
        }
    }
    let mut edge = Edge {
        t: t, i: i, j: j, p: 0.,
        cases: Vec::new(),
    };
    edge.add(p, correction);
    edges.push(edge);
}

impl Edge {
    pub fn add(&mut self, p: f64, correction: Correction) {
        self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        self.cases.push((correction, p));
    }
}

/// Boundary Information, corresponds to `node.boundary` in `FaultTolerantView.vue`
#[derive(Debug, Clone)]
pub struct Boundary {
    pub p: f64,
    pub cases: Vec::<(Correction, f64)>,
}

impl Boundary {
    pub fn add(&mut self, p: f64, correction: Correction) {
        self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        self.cases.push((correction, p));
    }
}

/// Correction Information, including all the data qubit at measurement stage t=6,12,18,...
/// Optimized for space because it will occupy O(L^4 T) memory in graph
#[derive(Debug, Clone)]
pub struct Correction {
    pub x: ndarray::Array3<bool>,
    pub z: ndarray::Array3<bool>,
}

/// Error type, corresponds to `ETYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorType {
    I,
    X,
    Z,
    Y,
}

impl ErrorType {
    pub fn multiply(&self, err: &Self) -> Self {
        match (self, err) {
            (Self::I, Self::I) => Self::I,
            (Self::I, Self::X) => Self::X,
            (Self::I, Self::Z) => Self::Z,
            (Self::I, Self::Y) => Self::Y,
            (Self::X, Self::I) => Self::X,
            (Self::X, Self::X) => Self::I,
            (Self::X, Self::Z) => Self::Y,
            (Self::X, Self::Y) => Self::Z,
            (Self::Z, Self::I) => Self::Z,
            (Self::Z, Self::X) => Self::Y,
            (Self::Z, Self::Z) => Self::I,
            (Self::Z, Self::Y) => Self::X,
            (Self::Y, Self::I) => Self::Y,
            (Self::Y, Self::X) => Self::Z,
            (Self::Y, Self::Z) => Self::X,
            (Self::Y, Self::Y) => Self::I,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PetGraphEdge {
    pub a: Index,
    pub b: Index,
    pub weight: f64,
}

#[derive(Debug)]
pub struct ExhaustedElement {
    pub cost: f64,
    pub next: Option<Index>,
    pub correction: Option<Correction>,
}
