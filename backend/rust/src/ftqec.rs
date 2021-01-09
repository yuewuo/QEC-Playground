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
#![allow(dead_code)]

use super::ndarray;
use super::petgraph;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use super::blossom_v;
use super::mwpm_approx;
use std::sync::{Arc};

/// uniquely index a node
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Index {
    pub t: usize,
    pub i: usize,
    pub j: usize,
}

impl Index {
    pub fn new(t: usize, i: usize, j: usize) -> Self {
        Self { t: t, i: i, j: j }
    }
    pub fn from_measurement_idx(mt: usize, mi: usize, mj: usize) -> Self {
        Self { t: 6 * (mt + 2), i: mi, j: mj }
    }
    pub fn to_measurement_idx(&self) -> (usize, usize, usize) {
        assert!(self.t >= 6 && self.t % 6 == 0, "only these indexes can be matched to measurement index");
        (self.t / 6 - 2, self.i, self.j)
    }
}

/// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
#[derive(Debug, Clone)]
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
    pub exhausted_boundary: Option<ExhaustedElement>,
    pub pet_node: Option<petgraph::graph::NodeIndex>,
    pub exhausted_map: HashMap<Index, ExhaustedElement>,
}

/// The structure of surface code, including how quantum gates are implemented
#[derive(Debug, Clone)]
pub struct PlanarCodeModel {
    /// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
    pub snapshot: Vec::< Vec::< Vec::< Option<Node> > > >,
    pub L: usize,
    pub MeasurementRounds: usize,
    pub T: usize,
    pub graph: Option<petgraph::graph::Graph<Index, PetGraphEdge>>,
    /// for each line, XOR the result. Only if no less than half of the result is 1.
    /// We do this because stabilizer operators will definitely have all 0 (because it generate 2 or 0 errors on every homology lines, XOR = 0)
    /// Only logical error will pose all 1 results, but sometimes single qubit errors will "hide" the logical error (because it
    ///    makes some result to 0), thus we determine there's a logical error if no less than half of the results are 1
    z_homology_lines: Vec< Vec::<(usize, usize)> >,
    x_homology_lines: Vec< Vec::<(usize, usize)> >,
}

impl PlanarCodeModel {
    pub fn new_standard_planar_code(MeasurementRounds: usize, L: usize) -> Self {
        // MeasurementRounds = 0 is means only one perfect measurement round
        assert!(L >= 2, "at lease one stabilizer is required");
        let mut model = Self::new_planar_code(MeasurementRounds, L, |_i, _j| true);
        // create Z stabilizer homology lines, detecting X errors
        for j in 0..L {
            let mut z_homology_line = Vec::new();
            for i in 0..L {
                z_homology_line.push((2 * i, 2 * j));
            }
            model.z_homology_lines.push(z_homology_line);
        }
        // create X stabilizer homology lines, detecting Z errors
        for i in 0..L {
            let mut x_homology_line = Vec::new();
            for j in 0..L {
                x_homology_line.push((2 * i, 2 * j));
            }
            model.x_homology_lines.push(x_homology_line);
        }
        model
    }
    pub fn new_rotated_planar_code(MeasurementRounds: usize, L: usize) -> Self {
        // MeasurementRounds = 0 is means only one perfect measurement round
        assert!(L >= 3 && L % 2 == 1, "at lease one stabilizer is required, L should be odd");
        let filter = |i, j| {
            let middle = (L - 1) as isize;
            let distance = (i as isize - middle).abs() + (j as isize - middle).abs();
            if distance <= middle {
                return true
            }
            if (i + j) % 2 == 0 {
                return false  // data qubit doesn't exist outside the middle radius in Manhattan distance
            }
            // but stabilizers exist outside that radius
            if i % 2 == 0 {  // Z stabilizers
                if (i as isize - middle) * (j as isize - middle) > 0 {
                    return distance <= middle + 1
                }
            } else {  // X stabilizers
                if (i as isize - middle) * (j as isize - middle) < 0 {
                    return distance <= middle + 1
                }
            }
            false
        };
        let mut model = Self::new_planar_code(MeasurementRounds, L, filter);
        // create Z stabilizer homology lines, detecting X errors
        for j in 0..L {
            let mut z_homology_line = Vec::new();
            for i in 0..L {
                z_homology_line.push((L - 1 - j + i, j + i));
            }
            model.z_homology_lines.push(z_homology_line);
        }
        // create X stabilizer homology lines, detecting Z errors
        for i in 0..L {
            let mut x_homology_line = Vec::new();
            for j in 0..L {
                x_homology_line.push((L - 1 + i - j, i + j));
            }
            model.x_homology_lines.push(x_homology_line);
        }
        model
    }
    pub fn new_planar_code<F>(MeasurementRounds: usize, L: usize, filter: F) -> Self
            where F: Fn(usize, usize) -> bool {
        let width = 2 * L - 1;
        let T = MeasurementRounds + 2;
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
                            exhausted_boundary: None,
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
            MeasurementRounds: MeasurementRounds,
            graph: None,
            z_homology_lines: Vec::new(),
            x_homology_lines: Vec::new(),
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
        let height = self.snapshot.len();
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            if t >= height - 6 {  // no error on the top, as a perfect measurement round
                node.error_rate_x = 0.;
                node.error_rate_z = 0.;
                node.error_rate_y = 0.;
            } else {
                node.error_rate_x = error_rate;
                node.error_rate_z = error_rate;
                node.error_rate_y = error_rate;
            }
        })
    }
    // this will remove bottom boundary
    pub fn set_depolarizing_error_with_perfect_initialization(&mut self, error_rate: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p(1-p)
        let height = self.snapshot.len();
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            if t >= height - 6 {  // no error on the top, as a perfect measurement round
                node.error_rate_x = 0.;
                node.error_rate_z = 0.;
                node.error_rate_y = 0.;
            } else if t <= 6 {
                node.error_rate_x = 0.;
                node.error_rate_z = 0.;
                node.error_rate_y = 0.;
            } else {
                node.error_rate_x = error_rate;
                node.error_rate_z = error_rate;
                node.error_rate_y = error_rate;
            }
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
                // println!("X error at {} {} {}",node.i, node.j, node.t);
            } else if random_number < node.error_rate_x + node.error_rate_z {
                node.error = ErrorType::Z;
                error_count += 1;
                // println!("Z error at {} {} {}",node.i, node.j, node.t);
            } else if random_number < node.error_rate_x + node.error_rate_z + node.error_rate_y {
                node.error = ErrorType::Y;
                error_count += 1;
                // println!("Y error at {} {} {}",node.i, node.j, node.t);
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
    pub fn iterate_measurement_stabilizers_mut<F>(&mut self, mut func: F) where F: FnMut(usize, usize, usize, &mut Node) {
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                        let qubit_type = node.qubit_type.clone();
                        if qubit_type == QubitType::StabZ || qubit_type == QubitType::StabX {
                            assert_eq!(node.gate_type, GateType::Measurement);
                            func(t, i, j, self.snapshot[t][i][j].as_mut().expect("exist"));
                        }
                    }
                }
            }
        }
    }
    /// iterate over every measurement errors
    pub fn iterate_measurement_stabilizers<F>(&self, mut func: F) where F: FnMut(usize, usize, usize, &Node) {
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                        let qubit_type = node.qubit_type.clone();
                        if qubit_type == QubitType::StabZ || qubit_type == QubitType::StabX {
                            assert_eq!(node.gate_type, GateType::Measurement);
                            func(t, i, j, self.snapshot[t][i][j].as_ref().expect("exist"));
                        }
                    }
                }
            }
        }
    }
    /// iterate over every measurement errors
    pub fn iterate_measurement_errors<F>(&self, mut func: F) where F: FnMut(usize, usize, usize, &Node) {
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                        if node.qubit_type == QubitType::StabZ {
                            assert_eq!(node.gate_type, GateType::Measurement);
                            let this_result = node.propagated == ErrorType::I || node.propagated == ErrorType::Z;
                            let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                            let last_result = last_node.propagated == ErrorType::I || last_node.propagated == ErrorType::Z;
                            if this_result != last_result {
                                func(t, i, j, self.snapshot[t][i][j].as_ref().expect("exist"));
                            }
                        } else if node.qubit_type == QubitType::StabX {
                            assert_eq!(node.gate_type, GateType::Measurement);
                            let this_result = node.propagated == ErrorType::I || node.propagated == ErrorType::X;
                            let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                            let last_result = last_node.propagated == ErrorType::I || last_node.propagated == ErrorType::X;
                            if this_result != last_result {
                                func(t, i, j, self.snapshot[t][i][j].as_ref().expect("exist"));
                            }
                        }
                    }
                }
            }
        }
    }
    /// generate default correction
    pub fn generate_default_correction(&self) -> Correction {
        let width = 2 * self.L - 1;
        let x = ndarray::Array::from_elem((self.MeasurementRounds + 1, width, width), false);
        let z = ndarray::Array::from_elem((self.MeasurementRounds + 1, width, width), false);
        Correction {
            x: x,
            z: z,
        }
    }
    /// get data qubit error pattern based on current `propagated` error on t=6,12,18,...
    pub fn get_data_qubit_error_pattern(&self) -> Correction {
        let mut correction = self.generate_default_correction();
        let mut x_mut = correction.x.view_mut();
        let mut z_mut = correction.z.view_mut();
        for (idx, t) in (12..self.snapshot.len()).step_by(6).enumerate() {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
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
        }
        correction
    }
    /// corresponds to `build_graph_given_error_rate` in `FaultTolerantView.vue`
    pub fn build_graph(&mut self) {
        for t in 1..self.snapshot.len() {  // 0 doesn't generate error
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
                            if p > 0. {
                                // simulate the error and measure it
                                self.add_error_at(t, i, j, error);
                                self.propagate_error();
                                let mut measurement_errors = Vec::new();
                                self.iterate_measurement_errors(|t, i, j, _node| {
                                    measurement_errors.push((t, i, j));
                                });
                                if measurement_errors.len() == 0 {  // no way to detect it, ignore
                                    continue
                                }
                                assert!(measurement_errors.len() <= 2, "single qubit error should not cause more than 2 measurement errors");
                                // compute correction pattern, so that applying this error pattern will exactly recover data qubit errors
                                let correction = Arc::new(self.get_data_qubit_error_pattern());
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
                                    if t1 <= 6 || t2 <= 6 {
                                        println!("error at {:?}", (t, i, j, error));
                                        println!("t1: {:?}, t2: {:?}", (t1, i1, j1), (t2, i2, j2));
                                        assert!(t1 > 6 || t2 > 6, "they shouldn't be both below 6");
                                        let node = if t1 > 6 {
                                            self.snapshot[t1][i1][j1].as_mut().expect("exist")
                                        } else {
                                            self.snapshot[t2][i2][j2].as_mut().expect("exist")
                                        };
                                        if node.boundary.is_none() {
                                            node.boundary = Some(Boundary {
                                                p: 0.,
                                                cases: Vec::new(),
                                            });
                                        }
                                        node.boundary.as_mut().expect("exist").add(p, correction);
                                    } else {
                                        add_edge_case(&mut self.snapshot[t1][i1][j1].as_mut().expect("exist").edges, t2, i2, j2, p, correction.clone());
                                        add_edge_case(&mut self.snapshot[t2][i2][j2].as_mut().expect("exist").edges, t1, i1, j1, p, correction);
                                    }
                                }
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
    fn optimize_correction_cases(original_cases: &Vec::<(Arc<Correction>, f64)>) -> Vec::<(Arc<Correction>, f64)> {
        let mut cases = HashMap::<Correction, f64>::new();
        for (correction, p) in original_cases.iter() {
            if cases.contains_key(&*correction) {
                let case_p = cases.get_mut(correction).expect("exist");
                let case_p_value: f64 = *case_p;
                *case_p = case_p_value * (1. - p) + p * (1. - case_p_value);
            } else {
                cases.insert((**correction).clone(), *p);
            }
        }
        let mut optimized_cases = Vec::with_capacity(cases.len());
        for (correction, p) in cases.drain() {
            optimized_cases.push((Arc::new(correction), p));
        }
        // println!("{} -> {}", original_cases.len(), optimized_cases.len());  // observation: the max amount of cases reduces from 7 to 3
        // sort the corrections based on its probability
        optimized_cases.sort_by(|(_, p1), (_, p2)| p2.partial_cmp(&p1).expect("probabilities shouldn't be NaN"));
        // let ps: Vec<f64> = optimized_cases.iter().map(|(_, p)| *p).collect();
        // println!("{:?}", ps);  // to check the order of it
        optimized_cases
    }
    /// combine and sort edges based on their probability.
    /// This shouldn't have much effect on the decoding performance, but I'm not sure of this, so just implement it and see
    pub fn optimize_correction_pattern(&mut self) {
        self.iterate_measurement_stabilizers_mut(|_t, _i, _j, node| {
            for edge in node.edges.iter_mut() {
                edge.cases = Self::optimize_correction_cases(&edge.cases);
            }
            if node.boundary.is_some() {
                let boundary = node.boundary.as_mut().expect("exist");
                boundary.cases = Self::optimize_correction_cases(&boundary.cases);
            }
        });
    }
    /// exhaustively search the minimum path from every measurement stabilizer to the others.
    /// Running `build_graph` required before running this function.
    pub fn build_exhausted_path<F>(&mut self, weight_of: F) where F: Fn(f64) -> f64 {
        // first build petgraph
        let mut graph = petgraph::graph::Graph::new_undirected();
        // add nodes before adding edge, so that they all have node number
        self.iterate_measurement_stabilizers_mut(|t, i, j, node| {
            node.pet_node = Some(graph.add_node(Index {
                t: t, i: i, j: j
            }));
        });
        // then add every edge
        self.iterate_measurement_stabilizers(|t, i, j, node| {
            for edge in &node.edges {
                let node_target = self.snapshot[edge.t][edge.i][edge.j].as_ref().expect("exist").pet_node.expect("exist");
                graph.add_edge(node.pet_node.expect("exist"), node_target, PetGraphEdge {
                    a: Index { t: t, i: i, j: j },
                    b: Index { t: edge.t, i: edge.i, j: edge.j },
                    weight: weight_of(edge.p),  // so that w1 + w2 = - log(p1) - log(p2) = - log(p1*p2) = - log(p_line)
                    // we want p_line to be as large as possible, it meets the goal of minimizing -log(p) 
                });
            }
        });
        // then run dijkstra for every node
        self.iterate_measurement_stabilizers_mut(|t, i, j, node| {
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
                        next_correction: None,
                    });
                }
            }
        });
        // use the result of dijkstra to build `next`, so that the shortest path is found is O(1) time
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        if self.snapshot[t][i][j].as_ref().expect("exist").gate_type == GateType::Measurement {
                            let node = self.snapshot[t][i][j].as_ref().expect("exist");
                            let target_indexes: Vec::<Index> = node.exhausted_map.keys().cloned().collect();
                            for target_index in target_indexes {
                                // find the next element by searching in `edges`
                                let node = self.snapshot[t][i][j].as_ref().expect("exist");
                                let mut min_cost = None;
                                let mut min_index = None;
                                for edge in &node.edges {
                                    let next_index = Index::from(edge);
                                    let mut current_cost = node.exhausted_map[&next_index].cost;
                                    if next_index != target_index {
                                        let next_node = self.snapshot[next_index.t][next_index.i][next_index.j].as_ref().expect("exist");
                                        current_cost += next_node.exhausted_map[&target_index].cost;
                                    }
                                    // compute the cost of node -> next_index -> target_index
                                    match min_cost.clone() {
                                        Some(min_cost_value) => {
                                            if current_cost < min_cost_value {
                                                min_cost = Some(current_cost);
                                                min_index = Some(next_index.clone());
                                            }
                                        }
                                        None => {
                                            min_cost = Some(current_cost);
                                            min_index = Some(next_index.clone());
                                        }
                                    }
                                }
                                // redefine node as a mutable one
                                let node = self.snapshot[t][i][j].as_mut().expect("exist");
                                node.exhausted_map.get_mut(&target_index).expect("exist").next = Some(min_index.expect("exist"));
                            }
                        }
                    }
                }
            }
        }
        // generate `next_correction` so that decoder works more efficiently
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        if self.snapshot[t][i][j].as_ref().expect("exist").gate_type == GateType::Measurement {
                            let node = self.snapshot[t][i][j].as_ref().expect("exist");
                            let target_indexes: Vec::<Index> = node.exhausted_map.keys().cloned().collect();
                            for target_index in target_indexes {
                                // go along `next` and combine over the `correction`
                                let this_index = Index{ t: t, i: i, j: j };
                                let this_node = self.snapshot[this_index.t][this_index.i][this_index.j].as_ref().expect("exist");
                                let next_index = this_node.exhausted_map[&target_index].next.as_ref().expect("exist");
                                let mut correction = None;
                                for edge in this_node.edges.iter() {  // find the edge of `next_index`
                                    if *next_index == Index::from(edge) {
                                        correction = Some(edge.cases[0].0.clone());
                                        break
                                    }
                                }
                                assert!(correction.is_some(), "next should be in `this_node.edges`");
                                let correction = correction.expect("exist");
                                // redefine node as a mutable one
                                let node = self.snapshot[t][i][j].as_mut().expect("exist");
                                node.exhausted_map.get_mut(&target_index).expect("exist").next_correction = Some(correction);
                            }
                        }
                    }
                }
            }
        }
        // generate `boundary.correction` so that every node has a path to boundary
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        if self.snapshot[t][i][j].as_ref().expect("exist").gate_type == GateType::Measurement {
                            let mut min_cost = None;
                            let mut min_index = None;
                            let node = self.snapshot[t][i][j].as_ref().expect("exist");
                            let index = Index::new(t, i, j);
                            self.iterate_measurement_stabilizers(|tb, ib, jb, node_b| {
                                match &node_b.boundary {
                                    Some(boundary) => {
                                        // only try if this node is directly connected to boundary
                                        if node.qubit_type == node_b.qubit_type {
                                            let (_, p) = &boundary.cases[0];
                                            let cost = weight_of(*p) + (if t == tb && i == ib && j == jb { 0. } else {
                                                node_b.exhausted_map[&index].cost
                                            });
                                            // println!("[{}][{}][{}] [{}][{}][{}] {}", t, i, j, tb, ib, jb, cost);
                                            match min_cost.clone() {
                                                Some(min_cost_value) => {
                                                    if cost < min_cost_value {
                                                        min_cost = Some(cost);
                                                        min_index = Some(Index::new(tb, ib, jb));
                                                    }
                                                }
                                                None => {
                                                    min_cost = Some(cost);
                                                    min_index = Some(Index::new(tb, ib, jb));
                                                }
                                            }
                                        }
                                    },
                                    None => { }
                                }
                            });
                            let min_cost = min_cost.expect("exist");
                            let min_index = min_index.expect("exist");
                            // println!("[{}][{}][{}] {} {:?}", t, i, j, min_cost, min_index);
                            let node_b = self.snapshot[min_index.t][min_index.i][min_index.j].as_ref().expect("exist");
                            let mut correction: Correction = (*node_b.boundary.as_ref().expect("exist").cases[0].0).clone();
                            if index != min_index {
                                correction.combine(node_b.exhausted_map[&index].next_correction.as_ref().expect("exist"));
                            }
                            // redefine node as a mutable one
                            let node = self.snapshot[t][i][j].as_mut().expect("exist");
                            node.exhausted_boundary = Some(ExhaustedElement {
                                cost: min_cost,
                                next: Some(min_index),
                                correction: Some(Arc::new(correction)),
                                next_correction: None,
                            });
                        }
                    }
                }
            }
        }
    }
    /// Autotune: compute weight based on error model
    pub fn build_exhausted_path_autotune(&mut self) {
        self.build_exhausted_path(|p| - p.ln())
    }
    /// Manhattan distance (but not exactly because there is 12 neighbors instead of 8) version
    pub fn build_exhausted_path_equally_weighted(&mut self) {
        self.build_exhausted_path(|_p| 1.)
    }
    /// get correction from two matched nodes
    /// use `correction` (or `next_correction` if former not provided) in `exhausted_map`
    pub fn get_correction_two_nodes(&self, a: &Index, b: &Index) -> Correction {
        let node_a = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist");
        let node_b = self.snapshot[b.t][b.i][b.j].as_ref().expect("exist");
        assert_eq!(node_a.gate_type, GateType::Measurement);
        assert_eq!(node_b.gate_type, GateType::Measurement);
        assert_eq!(node_a.qubit_type, node_b.qubit_type);  // so that it has a path
        if a == b {
            return self.generate_default_correction()
        }
        match &node_a.exhausted_map[&b].correction {
            Some(correction) => { (**correction).clone() }
            None => {
                let mut correction: Correction = (**node_a.exhausted_map[&b].next_correction.as_ref().expect("must call `build_exhausted_path`")).clone();
                let mut next_index = node_a.exhausted_map[&b].next.as_ref().expect("exist");
                while next_index != b {
                    let this_node = self.snapshot[next_index.t][next_index.i][next_index.j].as_ref().expect("exist");
                    correction.combine(&this_node.exhausted_map[&b].next_correction.as_ref().expect("must call `build_exhausted_path`"));
                    next_index = this_node.exhausted_map[&b].next.as_ref().expect("exist");
                }
                correction
            }
        }
    }
    pub fn generate_measurement(&self) -> Measurement {
        let width = 2 * self.L - 1;
        let mut measurement = Measurement(ndarray::Array::from_elem((self.MeasurementRounds + 1, width, width), false));
        let mut measurement_mut = measurement.view_mut();
        self.iterate_measurement_errors(|t, i, j, _node| {
            let (mt, mi, mj) = Index::new(t, i, j).to_measurement_idx();
            measurement_mut[[mt, mi, mj]] = true;
        });
        measurement
    }
    /// decode based on MWPM
    pub fn decode_MWPM(&self, measurement: &Measurement) -> Correction {
        // sanity check
        let shape = measurement.shape();
        let width = 2 * self.L - 1;
        assert_eq!(shape[0], self.MeasurementRounds + 1);
        assert_eq!(shape[1], width);
        assert_eq!(shape[2], width);
        // generate all the error measurements to be matched
        let mut to_be_matched = Vec::new();
        for mt in 0..self.MeasurementRounds + 1 {
            for mi in 0..width {
                for mj in 0..width {
                    if measurement[[mt, mi, mj]] {  // has a measurement error there
                        to_be_matched.push(Index::from_measurement_idx(mt, mi, mj));
                    }
                }
            }
        }
        // if to_be_matched.len() > 2 {
        //     println!{"TBM {:?}", to_be_matched};
        // }
        

        if to_be_matched.len() != 0 {
            // then add the edges to the graph
            let m_len = to_be_matched.len();  // boundary connection to `i` is `i + m_len`
            let node_num = m_len * 2;
            // Z (X) stabilizers are fully connected, boundaries are fully connected
            // stabilizer to boundary is one-to-one connected
            let mut weighted_edges = Vec::<(usize, usize, f64)>::new();
            for i in 0..m_len {
                for j in (i+1)..m_len {
                    let a = &to_be_matched[i];
                    let b = &to_be_matched[j];
                    let path = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").exhausted_map.get(&b);
                    if path.is_some() {
                        let cost = path.expect("exist").cost;
                        weighted_edges.push((i, j, cost));
                        weighted_edges.push((i + m_len, j + m_len, 0.));
                        // if to_be_matched.len() > 2 {
                        //     println!{"{} {} {} ", i, j, cost};
                        // }
                    }
                }
                let a = &to_be_matched[i];
                let cost = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").exhausted_boundary.as_ref().expect("exist").cost;
                weighted_edges.push((i, i + m_len, cost));
            }

            // if to_be_matched.len() > 2 {
            //     println!{"node num {:?}, weighted edges {:?}", node_num, weighted_edges};
            // }
            let matching = blossom_v::safe_minimum_weight_perfect_matching(node_num, weighted_edges);
            // println!("{:?}", to_be_matched);
            // println!("matching: {:?}", matching);
            // if to_be_matched.len() > 2 {
            //     println!("matching: {:?}", matching);
            // }
            let mut correction = self.generate_default_correction();
            for i in 0..m_len {
                let j = matching[i];
                let a = &to_be_matched[i];
                if j < i {  // only add correction if j < i, so that the same correction is not applied twice
                    // println!("match peer {:?} {:?}", to_be_matched[i], to_be_matched[j]);
                    correction.combine(&self.get_correction_two_nodes(a, &to_be_matched[j]));
                } else if j >= m_len {  // matched with boundary
                    // println!("match boundary {:?}", to_be_matched[i]);
                    let node = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist");
                    correction.combine(node.exhausted_boundary.as_ref().expect("exist").correction.as_ref().expect("exist"));
                }
            }
            // if to_be_matched.len() > 2 {
            //     println!("correction: {:?}", correction);
            // }
            correction
        } else {
            // no measurement errors found
            self.generate_default_correction()
        }
    }

    /// decode based on MWPM
    pub fn decode_MWPM_approx(&self, measurement: &Measurement, substreams: usize) -> Correction {
        // sanity check
        let shape = measurement.shape();
        let width = 2 * self.L - 1;
        assert_eq!(shape[0], self.T);
        assert_eq!(shape[1], width);
        assert_eq!(shape[2], width);
        // generate all the error measurements to be matched
        let mut to_be_matched = Vec::new();
        for mt in 0..self.T {
            for mi in 0..width {
                for mj in 0..width {
                    if measurement[[mt, mi, mj]] {  // has a measurement error there
                        to_be_matched.push(Index::from_measurement_idx(mt, mi, mj));
                    }
                }
            }
        }
        // if to_be_matched.len() > 2 {
        //     println!{"TBM {:?}", to_be_matched};
        // }
        

        if to_be_matched.len() != 0 {
            // then add the edges to the graph
            let m_len = to_be_matched.len();  // boundary connection to `i` is `i + m_len`
            let node_num = m_len * 2;
            // Z (X) stabilizers are fully connected, boundaries are fully connected
            // stabilizer to boundary is one-to-one connected
            let mut weighted_edges = Vec::<(usize, usize, f64)>::new();
            for i in 0..m_len {
                for j in (i+1)..m_len {
                    let a = &to_be_matched[i];
                    let b = &to_be_matched[j];
                    let path = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").exhausted_map.get(&b);
                    if path.is_some() {
                        let cost = path.expect("exist").cost;
                        weighted_edges.push((i, j, cost));
                        // weighted_edges.push((i + m_len, j + m_len, 0.));
                        // if to_be_matched.len() > 2 {
                        //     println!{"{} {} {} ", i, j, cost};
                        // }
                    }
                }
                let a = &to_be_matched[i];
                let cost = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").exhausted_boundary.as_ref().expect("exist").cost;
                weighted_edges.push((i, i + m_len, cost));
            }

            // if to_be_matched.len() > 2 {
            //     println!{"node num {:?}, weighted edges {:?}", node_num, weighted_edges};
            // }
            let matching = mwpm_approx::minimum_weight_perfect_matching_approx(node_num, weighted_edges, substreams);
            // println!("{:?}", to_be_matched);
            // println!("matching: {:?}", matching);
            // if to_be_matched.len() > 2 {
            //     println!("matching: {:?}", matching);
            // }
            let mut correction = self.generate_default_correction();
            for (i,j) in matching.iter() {
                if *i < m_len && *j < m_len{
                    correction.combine(&self.get_correction_two_nodes(&to_be_matched[*i], &to_be_matched[*j]));
                }
                else if *i < m_len {
                    let a = &to_be_matched[*i];
                    let node = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist");
                    correction.combine(node.exhausted_boundary.as_ref().expect("exist").correction.as_ref().expect("exist"));
                }
                else if *j < m_len {
                    let a = &to_be_matched[*j];
                    let node = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist");
                    correction.combine(node.exhausted_boundary.as_ref().expect("exist").correction.as_ref().expect("exist"));
                }
                else {
                    println!{"This case cannot occur i,j,m_len {} {} {}",i,j,m_len};
                }
            }
            // if to_be_matched.len() > 2 {
            //     println!("correction: {:?}", correction);
            // }
            correction
        } else {
            // no measurement errors found
            self.generate_default_correction()
        }
    }

    /// decode do nothing. This should be the actual baseline
    pub fn decode_do_nothing(&self, _measurement: &Measurement) -> Correction {
        self.generate_default_correction()
    }

    /// validate correction on the bottom layer strictly, see if there is logical error or uncorrected stabilizers.
    /// return Err(reason) if correction is not successful. reason is a readable string.
    pub fn validate_corrected_on_layer(&self, corrected: &Correction, layer: usize) -> Result<(), ValidationFailedReason> {
        assert!(layer < self.T, "layer ranges from 0 to T-1");
        assert!(self.z_homology_lines.len() > 0 && self.x_homology_lines.len() > 0, "single boundary required");
        let mut z_homology_results = Vec::new();
        let mut x_homology_results = Vec::new();
        for is_z in [false, true].iter() {
            let homology_results = if *is_z { &mut z_homology_results } else { &mut x_homology_results };
            let homology_lines = if *is_z { &self.z_homology_lines } else { &self.x_homology_lines };
            let corrected_array = if *is_z { &corrected.x } else { &corrected.z };  // Z detects X, X detects Z
            for homology_line in homology_lines {
                let mut xor = false;
                for (i, j) in homology_line {
                    xor = xor ^ corrected_array[[layer, *i, *j]];
                }
                homology_results.push(xor);
            }
        }
        let z_homology_counts = z_homology_results.iter().filter(|x| **x).count();
        let x_homology_counts = z_homology_results.iter().filter(|x| **x).count();
        let z_has_logical = z_homology_counts * 2 > z_homology_results.len();
        let x_has_logical = x_homology_counts * 2 > x_homology_results.len();
        // println!("z_homology_counts: {}, x_homology_counts: {}", z_homology_counts, x_homology_counts);
        if !z_has_logical && !x_has_logical {
            Ok(())
        } else if z_has_logical && x_has_logical {
            Err(ValidationFailedReason::BothXandZLogicalError(layer, x_homology_counts, x_homology_results.len(), z_homology_counts, z_homology_results.len()))
        } else if z_has_logical {
            Err(ValidationFailedReason::ZLogicalError(layer, z_homology_counts, z_homology_results.len()))
        } else {
            Err(ValidationFailedReason::ZLogicalError(layer, x_homology_counts, x_homology_results.len()))
        }
    }
    pub fn validate_correction_on_t_layer(&self, correction: &Correction, layer: usize) -> Result<(), ValidationFailedReason> {
        let mut corrected = self.get_data_qubit_error_pattern();
        // println!{"Corrected{:?}", corrected};
        corrected.combine(&correction);  // apply correction to error pattern
        self.validate_corrected_on_layer(&corrected, layer)
    }
    pub fn validate_correction_on_top_layer(&self, correction: &Correction) -> Result<(), ValidationFailedReason> {
        let mut corrected = self.get_data_qubit_error_pattern();
        corrected.combine(&correction);  // apply correction to error pattern
        self.validate_corrected_on_layer(&corrected, self.MeasurementRounds)
    }
    pub fn validate_correction_on_bottom_layer(&self, correction: &Correction) -> Result<(), ValidationFailedReason> {
        let mut corrected = self.get_data_qubit_error_pattern();
        corrected.combine(&correction);  // apply correction to error pattern
        self.validate_corrected_on_layer(&corrected, 0)
    }
    pub fn validate_correction_on_all_layers(&self, correction: &Correction) -> Result<(), ValidationFailedReason> {
        let mut corrected = self.get_data_qubit_error_pattern();
        // println!{"Before{:?}", corrected};
        corrected.combine(&correction);  // apply correction to error pattern
        // println!{"Corrected{:?}", corrected};
        for mt in 0..=self.MeasurementRounds {
            self.validate_corrected_on_layer(&corrected, mt)?;
        }
        Ok(())
    }

    pub fn validate_correction_on_boundary(&self, correction: &Correction) -> Result<(), ValidationFailedReason> {
        let mut corrected = self.get_data_qubit_error_pattern();
        // let mut corrected = self.get_data_qubit_error_pattern().clone();

        // println!{"Before{:?}", corrected};
        corrected.combine(&correction);  // apply correction to error pattern
        // println!{"Corrected{:?}", corrected};
        
        // Z stabilizer homology lines, j = 0
        let mut x_error_count = 0;
        let mut current_status;
        for i in 0..self.L {
            current_status = false;
            for layer in 0..=self.MeasurementRounds {
                if corrected.x[[layer, (i*2), 0]] != current_status{
                    x_error_count += 1;
                    current_status = corrected.x[[layer, (i*2), 0]];
                }
            }
        }
        if x_error_count %2 != 0 {
            // println!("Error X {}", x_error_count);
            return Err(ValidationFailedReason::XLogicalError(0, x_error_count, x_error_count))
        }
        
        // X stabilizer homology lines, i = 0
        let mut z_error_count = 0;
        for j in 0..self.L {
            current_status = false;
            for layer in 0..=self.MeasurementRounds {
                if corrected.z[[layer, 0, (j*2)]] != current_status {
                    z_error_count += 1;
                    current_status = corrected.z[[layer, 0, (j*2)]];
                }
            }
        }

        if z_error_count %2 != 0 {
            // println!("Error Z {}", z_error_count);
            return Err(ValidationFailedReason::ZLogicalError(0, z_error_count, z_error_count))
        }

        // println!("X {} Y {}", x_error_count, z_error_count);

        Ok(())
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
    pub cases: Vec::<(Arc<Correction>, f64)>,
}

impl From<&Edge> for Index {
    fn from(edge: &Edge) -> Self {
        Self {
            t: edge.t,
            i: edge.i,
            j: edge.j,
        }
    }
}

pub fn add_edge_case(edges: &mut Vec::<Edge>, t: usize, i: usize, j: usize, p: f64, correction: Arc<Correction>) {
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
    pub fn add(&mut self, p: f64, correction: Arc<Correction>) {
        self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        self.cases.push((correction, p));
    }
}

/// Boundary Information, corresponds to `node.boundary` in `FaultTolerantView.vue`
#[derive(Debug, Clone)]
pub struct Boundary {
    pub p: f64,
    pub cases: Vec::<(Arc<Correction>, f64)>,
}

impl Boundary {
    pub fn add(&mut self, p: f64, correction: Arc<Correction>) {
        self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        self.cases.push((correction, p));
    }
}

/// Correction Information, including all the data qubit at measurement stage t=6,12,18,...
/// Optimized for space because it will occupy O(L^4 T) memory in graph
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Correction {
    pub x: ndarray::Array3<bool>,
    pub z: ndarray::Array3<bool>,
}

/// Measurement Result, including all the stabilizer at measurement stage t=6,12,18,...
#[derive(Debug, Clone)]
pub struct Measurement (ndarray::Array3<bool>);

impl Deref for Measurement {
    type Target = ndarray::Array3<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Measurement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Correction {
    pub fn xor_ndarray3(a: &mut ndarray::Array3<bool>, b: &ndarray::Array3<bool>) {
        let shape = b.shape();
        assert_eq!(shape, a.shape());
        let mut am = a.view_mut();
        for t in 0..shape[0] {
            for i in 0..shape[1] {
                for j in 0..shape[2] {
                    am[[t, i, j]] = am[[t, i, j]] ^ b[[t, i, j]];
                }
            }
        }
    }
    pub fn combine(&mut self, next: &Self) {
        Correction::xor_ndarray3(&mut self.x, &next.x);
        Correction::xor_ndarray3(&mut self.z, &next.z);
    }
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

#[derive(Debug, PartialEq, Clone)]
pub struct PetGraphEdge {
    pub a: Index,
    pub b: Index,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct ExhaustedElement {
    pub cost: f64,
    pub next: Option<Index>,
    /// either `correction` or `next_correction` is needed for decoder to work
    /// `correction` will be used first if exists, which occupies too much memory and too many initialization time
    pub correction: Option< Arc<Correction> >,
    /// `next_correction` is generated by default
    pub next_correction: Option< Arc<Correction> >,
}

#[derive(Debug, Clone)]
pub enum ValidationFailedReason {
    /// layer, homology_counts, homology_results.len()
    XLogicalError(usize, usize, usize),
    /// layer, homology_counts, homology_results.len()
    ZLogicalError(usize, usize, usize),
    /// layer, x_homology_counts, x_homology_results.len(), z_homology_counts, z_homology_results.len()
    BothXandZLogicalError(usize, usize, usize, usize, usize),
}

impl From<&ValidationFailedReason> for String {
    fn from(edge: &ValidationFailedReason) -> Self {
        match edge {
            ValidationFailedReason::XLogicalError(layer, homology_counts, homology_results_len) => 
                format!("X logical error is detected on measurement layer {}, homology count / len = {} / {}", layer, homology_counts, homology_results_len),
            ValidationFailedReason::ZLogicalError(layer, homology_counts, homology_results_len) => 
                format!("Z logical error is detected on measurement layer {}, homology count / len = {} / {}", layer, homology_counts, homology_results_len),
            ValidationFailedReason::BothXandZLogicalError(layer, x_homology_counts, x_homology_results_len, z_homology_counts, z_homology_results_len) =>
                format!("X logical error and Z logical error are both detected on measurement layer {}, {}/{}, {}/{}", layer
                    , x_homology_counts, x_homology_results_len, z_homology_counts, z_homology_results_len),
        }
    }
}
