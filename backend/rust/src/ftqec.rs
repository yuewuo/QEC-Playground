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
}

/// The structure of surface code, including how quantum gates are implemented
#[derive(Debug)]
pub struct ErrorModel {
    /// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
    pub snapshot: Vec::< Vec::< Vec::< Option<Node> > > >,
    pub L: usize,
    pub T: usize,
}

impl ErrorModel {
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
