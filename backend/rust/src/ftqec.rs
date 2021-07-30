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
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use super::blossom_v;
use super::mwpm_approx;
use std::sync::{Arc};
use super::types::{QubitType, ErrorType, CorrelatedErrorType, CorrelatedErrorModel, ErrorModel};
use super::union_find_decoder;
use super::either::Either;
use super::serde_json;
use std::time::Instant;

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
    /// note that correlated error is applied to next time step, without losing generality
    pub correlated_error_model: Option<CorrelatedErrorModel>,
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

/// record the code type
#[derive(Debug, Clone)]
pub enum CodeType {
    StandardPlanarCode,
    RotatedPlanarCode,
    StandardXZZXCode,
    RotatedXZZXCode,
    Unknown,
}

/// The structure of surface code, including how quantum gates are implemented
#[derive(Debug, Clone)]
pub struct PlanarCodeModel {
    pub code_type: CodeType,
    /// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
    pub snapshot: Vec::< Vec::< Vec::< Option<Node> > > >,
    pub di: usize,  // code distance of i dimension
    pub dj: usize,  // code distance of i dimension
    pub MeasurementRounds: usize,
    pub T: usize,
    pub graph: Option<petgraph::graph::Graph<Index, PetGraphEdge>>,
    pub use_combined_probability: bool,
    /// for each line, XOR the result. Only if no less than half of the result is 1.
    /// We do this because stabilizer operators will definitely have all 0 (because it generate 2 or 0 errors on every homology lines, XOR = 0)
    /// Only logical error will pose all 1 results, but sometimes single qubit errors will "hide" the logical error (because it
    ///    makes some result to 0), thus we determine there's a logical error if no less than half of the results are 1
    z_homology_lines: Vec< Vec::<(usize, usize)> >,
    x_homology_lines: Vec< Vec::<(usize, usize)> >,
}

impl PlanarCodeModel {
    pub fn new_standard_planar_code(MeasurementRounds: usize, L: usize) -> Self {
        // MeasurementRounds = 0 means only one perfect measurement round
        assert!(L >= 2, "at lease one stabilizer is required");
        let mut model = Self::new_planar_code(CodeType::StandardPlanarCode, MeasurementRounds, L, L, |_i, _j| true);
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
        let mut model = Self::new_planar_code(CodeType::RotatedPlanarCode, MeasurementRounds, L, L, filter);
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
    pub fn new_planar_code<F>(code_type: CodeType, MeasurementRounds: usize, di: usize, dj: usize, filter: F) -> Self
            where F: Fn(usize, usize) -> bool {
        let width_i = 2 * di - 1;
        let width_j = 2 * dj - 1;
        let T = MeasurementRounds + 2;
        let height = T * 6 + 1;
        let mut snapshot = Vec::with_capacity(height);
        for t in 0..height {
            let mut snapshot_row_0 = Vec::with_capacity(width_i);
            for i in 0..width_i {
                let mut snapshot_row_1 = Vec::with_capacity(width_j);
                for j in 0..width_j {
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
                                    if i+1 < width_i && filter(i+1, j) {
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
                                    if j+1 < width_j && filter(i, j+1) {
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
                                    if j+1 < width_i && filter(i, j+1) {
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
                                    if i+1 < width_j && filter(i+1, j) {
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
                            correlated_error_model: None,
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
            code_type: code_type,
            snapshot: snapshot,
            di: di,
            dj: dj,
            T: T,
            MeasurementRounds: MeasurementRounds,
            graph: None,
            use_combined_probability: false,
            z_homology_lines: Vec::new(),
            x_homology_lines: Vec::new(),
        }
    }

    pub fn new_rotated_XZZX_code(MeasurementRounds: usize, L: usize) -> Self {
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
        let model = Self::new_XZZX_code(CodeType::RotatedXZZXCode, MeasurementRounds, L, L, filter);
        model
    }

    pub fn new_standard_XZZX_code_rectangle(MeasurementRounds: usize, di: usize, dj: usize) -> Self {
        // MeasurementRounds = 0 means only one perfect measurement round
        assert!(di >= 2 && dj >= 2, "at lease one stabilizer is required");
        let model = Self::new_XZZX_code(CodeType::StandardXZZXCode, MeasurementRounds, di, dj, |_i, _j| true);
        // don't build homology lines since it's deprecated
        model
    }

    pub fn new_standard_XZZX_code(MeasurementRounds: usize, L: usize) -> Self {
        // MeasurementRounds = 0 means only one perfect measurement round
        assert!(L >= 2, "at lease one stabilizer is required");
        let model = Self::new_XZZX_code(CodeType::StandardXZZXCode, MeasurementRounds, L, L, |_i, _j| true);
        // don't build homology lines since it's deprecated
        model
    }

    pub fn new_XZZX_code<F>(code_type: CodeType, MeasurementRounds: usize, di: usize, dj: usize, filter: F) -> Self
            where F: Fn(usize, usize) -> bool {
        let width_i = 2 * di - 1;
        let width_j = 2 * dj - 1;
        let T = MeasurementRounds + 2;
        let height = T * 6 + 1;
        let mut snapshot = Vec::with_capacity(height);
        for t in 0..height {
            let mut snapshot_row_0 = Vec::with_capacity(width_i);
            for i in 0..width_i {
                let mut snapshot_row_1 = Vec::with_capacity(width_j);
                for j in 0..width_j {
                    if filter(i, j) {
                        let stage = Stage::from(t);
                        let qubit_type = if (i + j) % 2 == 0 { QubitType::Data } else
                            { if i % 2 == 0 { QubitType::StabXZZXLogicalZ } else { QubitType::StabXZZXLogicalX } };
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
                                    if i+1 < width_i && filter(i+1, j) {
                                        gate_type = GateType::ControlledPhase;
                                        connection = Some(Connection{ t: t, i: i+1, j: j });
                                    }
                                } else {
                                    if i >= 1 && filter(i-1, j) {
                                        gate_type = GateType::ControlledPhase;
                                        connection = Some(Connection{ t: t, i: i-1, j: j });
                                    }
                                }
                            },
                            Stage::CXGate2 => {
                                if qubit_type == QubitType::Data {
                                    if j+1 < width_j && filter(i, j+1) {
                                        gate_type = GateType::Target;
                                        connection = Some(Connection{ t: t, i: i, j: j+1 });
                                    }
                                } else {
                                    if j >= 1 && filter(i, j-1) {
                                        gate_type = GateType::Control;
                                        connection = Some(Connection{ t: t, i: i, j: j-1 });
                                    }
                                }
                            },
                            Stage::CXGate3 => {
                                if qubit_type == QubitType::Data {
                                    if j >= 1 && filter(i, j-1) {
                                        gate_type = GateType::Target;
                                        connection = Some(Connection{ t: t, i: i, j: j-1 });
                                    }
                                } else {
                                    if j+1 < width_j && filter(i, j+1) {
                                        gate_type = GateType::Control;
                                        connection = Some(Connection{ t: t, i: i, j: j+1 });
                                    }
                                }
                            },
                            Stage::CXGate4 => {
                                if qubit_type == QubitType::Data {
                                    if i >= 1 && filter(i-1, j) {
                                        gate_type = GateType::ControlledPhase;
                                        connection = Some(Connection{ t: t, i: i-1, j: j });
                                    }
                                } else {
                                    if i+1 < width_i && filter(i+1, j) {
                                        gate_type = GateType::ControlledPhase;
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
                            correlated_error_model: None,
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
            code_type: code_type,
            snapshot: snapshot,
            di: di,
            dj: dj,
            T: T,
            MeasurementRounds: MeasurementRounds,
            graph: None,
            use_combined_probability: false,
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
    pub fn set_individual_error(&mut self, px: f64, py: f64, pz: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p(1-p)
        let height = self.snapshot.len();
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            if t >= height - 6 {  // no error on the top, as a perfect measurement round
                node.error_rate_x = 0.;
                node.error_rate_z = 0.;
                node.error_rate_y = 0.;
            } else {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        })
    }
    pub fn set_depolarizing_error(&mut self, error_rate: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p(1-p)
        self.set_individual_error(error_rate, error_rate, error_rate)
    }
    // this will remove bottom boundary
    pub fn set_individual_error_with_perfect_initialization(&mut self, px: f64, py: f64, pz: f64) {
        assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
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
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        })
    }
    pub fn set_depolarizing_error_with_perfect_initialization(&mut self, error_rate: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p
        self.set_individual_error_with_perfect_initialization(error_rate, error_rate, error_rate)
    }
    // remove bottom boundary, (1-p)^2I + p(1-p)X + p(1-p)Z + p^2Y
    pub fn set_phenomenological_error_with_perfect_initialization(&mut self, error_rate: f64) {
        let height = self.snapshot.len();
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            node.error_rate_x = 0.;
            node.error_rate_z = 0.;
            node.error_rate_y = 0.;
            // no error on the top and bottom
            if t < height - 6 && t > 6 {
                let next_stage = Stage::from(t + 1);
                match next_stage {
                    Stage::Measurement => {
                        node.error_rate_x = error_rate * (1. - error_rate);
                        node.error_rate_z = error_rate * (1. - error_rate);
                        node.error_rate_y = error_rate * error_rate;
                    },
                    _ => {},
                }
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
    /// generate random error based on `error_rate` in each node, return the number of errors
    pub fn generate_random_errors<F>(&mut self, mut rng: F) -> usize where F: FnMut() -> f64 {
        let mut pending_errors = Vec::new();
        self.iterate_snapshot_mut(|t, i, j, node| {
            let random_number = rng();
            if random_number < node.error_rate_x {
                node.error = ErrorType::X;
                // println!("X error at {} {} {}",node.i, node.j, node.t);
            } else if random_number < node.error_rate_x + node.error_rate_z {
                node.error = ErrorType::Z;
                // println!("Z error at {} {} {}",node.i, node.j, node.t);
            } else if random_number < node.error_rate_x + node.error_rate_z + node.error_rate_y {
                node.error = ErrorType::Y;
                // println!("Y error at {} {} {}",node.i, node.j, node.t);
            } else {
                node.error = ErrorType::I;
            }
            match &node.correlated_error_model {
                Some(correlated_error_model) => {
                    let random_number = rng();
                    let correlated_error_type = correlated_error_model.generate_random_error(random_number);
                    let my_error = correlated_error_type.my_error();
                    if my_error != ErrorType::I {
                        pending_errors.push(((t, i, j), my_error));
                    }
                    let peer_error = correlated_error_type.peer_error();
                    if peer_error != ErrorType::I {
                        let connection = node.connection.as_ref().expect("correlated error must corresponds to a two-qubit gate");
                        let (ct, ci, cj) = (connection.t, connection.i, connection.j);
                        pending_errors.push(((ct, ci, cj), peer_error));
                    }
                },
                None => { },
            }
        });
        // apply pending errors
        for ((t, i, j), peer_error) in pending_errors.drain(..) {
            let mut node = self.snapshot[t][i][j].as_mut().expect("exist");
            node.error = node.error.multiply(&peer_error);
        }
        // count number of errors
        let mut error_count = 0;
        self.iterate_snapshot_mut(|_t, _i, _j, node| {
            if node.error != ErrorType::I {
                // println!("error [{}][{}][{}] : {:?}", _t, _i, _j, node.error);
                error_count += 1;
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

    pub fn print_errors(&self) {
        self.iterate_snapshot(|t, i, j, node| {
            if node.error != ErrorType::I {
                println!("{:?} at {} {} {}", node.error,t,i,j);
            }
        });
    }

    pub fn add_error_at_no_sanity_check(&mut self, t: usize, i: usize, j: usize, error: &ErrorType) {
        let node = &mut self.snapshot[t][i][j].as_mut().expect("exist");
        node.error = node.error.multiply(error);
    }

    pub fn add_error_at(&mut self, t: usize, i: usize, j: usize, error: &ErrorType) -> Option<ErrorType> {
        if let Some(array) = self.snapshot.get_mut(t) {
            if let Some(array) = array.get_mut(i) {
                if let Some(element) = array.get_mut(j) {
                    match element {
                        Some(ref mut node) => {
                            let p = match error {
                                ErrorType::X => node.error_rate_x + node.error_rate_y,
                                ErrorType::Z => node.error_rate_z + node.error_rate_y,
                                // Y error requires both x and z has corresponding edge
                                ErrorType::Y => (node.error_rate_x + node.error_rate_y) * (node.error_rate_z + node.error_rate_y),
                                ErrorType::I => (1. - node.error_rate_x - node.error_rate_y - node.error_rate_z),
                            };
                            if p > 0. {  // only add error if physical error rate is greater than 0.
                                node.error = node.error.multiply(error);
                                Some(node.error.clone())
                            } else {
                                None
                            }
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

    /// a faster version of clear_error used only internally
    pub fn _clear_error_interested_region(&mut self, interested_region: &mut HashSet<(usize, usize)>) {
        for t in 0..self.snapshot.len() {
            for &(i, j) in interested_region.iter() {
                let node = self.snapshot[t][i][j].as_mut().expect("exist");
                node.error = ErrorType::I;
                node.propagated = ErrorType::I;
            }
        }
    }
    /// a faster version of propagate_error used only internally
    pub fn _propagate_error_with_interested_region(&mut self, interested_region: &mut HashSet<(usize, usize)>) {
        for t in 0..self.snapshot.len() - 1 {
            let mut pending_interested_region = Vec::new();
            for &(i, j) in interested_region.iter() {
                let propagated_neighbor = self.propagate_error_at(t, i, j);
                match propagated_neighbor {
                    Some((pi, pj)) => pending_interested_region.push((pi, pj)),
                    None => { },
                }
            }
            for (i, j) in pending_interested_region.drain(..) {
                interested_region.insert((i, j));
            }
        }
    }
    /// return the propagated neighbor (i, j) if exists
    pub fn propagate_error_at(&mut self, t: usize, i: usize, j: usize) -> Option<(usize, usize)> {
        let mut propagated_neighbor = None;
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
        // propagated to other qubits through CX gate
        if gate_type == GateType::Control {
            let connection = node_connection.as_ref().expect("exist");
            if node_propagated == ErrorType::X || node_propagated == ErrorType::Y {
                let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                peer_node.propagated = peer_node.propagated.multiply(&ErrorType::X);
                propagated_neighbor = Some((connection.i, connection.j));
            }
        } else if gate_type == GateType::Target {
            let connection = node_connection.as_ref().expect("exist");
            if node_propagated == ErrorType::Z || node_propagated == ErrorType::Y {
                let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                peer_node.propagated = peer_node.propagated.multiply(&ErrorType::Z);
                propagated_neighbor = Some((connection.i, connection.j));
            }
        }
        // also propagated to other qubits via CZ gate
        if gate_type == GateType::ControlledPhase {
            let connection = node_connection.as_ref().expect("exist");
            if node_propagated == ErrorType::X || node_propagated == ErrorType::Y {
                let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                peer_node.propagated = peer_node.propagated.multiply(&ErrorType::Z);
                propagated_neighbor = Some((connection.i, connection.j));
            }
        }
        propagated_neighbor
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
                        self.propagate_error_at(t, i, j);
                    }
                }
            }
        }
    }
    /// iterate over every measurement stabilizers w/wo errors
    pub fn iterate_measurement_stabilizers_mut<F>(&mut self, mut func: F) where F: FnMut(usize, usize, usize, &mut Node) {
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                        let qubit_type = node.qubit_type.clone();
                        if qubit_type != QubitType::Data {
                            assert_eq!(node.gate_type, GateType::Measurement);
                            func(t, i, j, self.snapshot[t][i][j].as_mut().expect("exist"));
                        }
                    }
                }
            }
        }
    }
    /// iterate over every measurement stabilizers w/wo errors
    pub fn iterate_measurement_stabilizers<F>(&self, mut func: F) where F: FnMut(usize, usize, usize, &Node) {
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                        let qubit_type = node.qubit_type.clone();
                        if qubit_type != QubitType::Data {
                            assert_eq!(node.gate_type, GateType::Measurement);
                            func(t, i, j, self.snapshot[t][i][j].as_ref().expect("exist"));
                        }
                    }
                }
            }
        }
    }
    pub fn is_measurement_error_at(&self, t: usize, i: usize, j: usize) -> bool {
        let node = self.snapshot[t][i][j].as_ref().expect("exist");
        match node.qubit_type {
            QubitType::StabZ => {
                assert_eq!(node.gate_type, GateType::Measurement);
                let this_result = node.propagated == ErrorType::I || node.propagated == ErrorType::Z;
                let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                let last_result = last_node.propagated == ErrorType::I || last_node.propagated == ErrorType::Z;
                this_result != last_result
            },
            QubitType::StabX | QubitType::StabXZZXLogicalX | QubitType::StabXZZXLogicalZ => {
                assert_eq!(node.gate_type, GateType::Measurement);
                let this_result = node.propagated == ErrorType::I || node.propagated == ErrorType::X;
                let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                let last_result = last_node.propagated == ErrorType::I || last_node.propagated == ErrorType::X;
                this_result != last_result
            },
            _ => unreachable!(),
        }
    }
    /// iterate over every measurement errors
    pub fn iterate_measurement_errors<F>(&self, mut func: F) where F: FnMut(usize, usize, usize, &Node) {
        for t in (12..self.snapshot.len()).step_by(6) {
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                        if node.qubit_type != QubitType::Data {
                            if self.is_measurement_error_at(t, i, j) {
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
        let width_i = 2 * self.di - 1;
        let width_j = 2 * self.dj - 1;
        Correction::new_all_false(self.MeasurementRounds + 1, width_i, width_j)
    }
    pub fn generate_default_sparse_correction(&self) -> SparseCorrection {
        let width_i = 2 * self.di - 1;
        let width_j = 2 * self.dj - 1;
        SparseCorrection::new_all_false(self.MeasurementRounds + 1, width_i, width_j)
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
    /// this is to solve the very high complexity of the original `build_graph` function O(d^6) ~ O(d^7), by assuming few errors at each time
    pub fn fast_measurement_given_few_errors(&mut self, errors: &Vec<(usize, usize, usize, ErrorType)>) -> (SparseCorrection, Vec<(usize, usize, usize)>) {
        // observation: errors will mainly propagate vertically (t) but rarely propagate horizontally (i, j)
        let mut interested_region: HashSet<(usize, usize)> = HashSet::new();
        for (t, i, j, error) in errors.iter() {
            self.add_error_at_no_sanity_check(*t, *i, *j, error);
            interested_region.insert((*i, *j));
        }
        self._propagate_error_with_interested_region(&mut interested_region);
        let mut sparse_correction = self.generate_default_sparse_correction();
        for (idx, t) in (12..self.snapshot.len()).step_by(6).enumerate() {
            for &(i, j) in interested_region.iter() {
                let node = self.snapshot[t][i][j].as_ref().expect("exist");
                if node.qubit_type == QubitType::Data {
                    let node_propagated = node.propagated.clone();
                    if idx == 0 {
                        match node_propagated {
                            ErrorType::X => { sparse_correction.xs.push((idx, i, j)); }
                            ErrorType::Z => { sparse_correction.zs.push((idx, i, j)); }
                            ErrorType::Y => { sparse_correction.xs.push((idx, i, j)); sparse_correction.zs.push((idx, i, j)); }
                            ErrorType::I => { }
                        }
                    } else {
                        let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                        if node_propagated != last_node.propagated {
                            match node_propagated.multiply(&last_node.propagated) {
                                ErrorType::X => { sparse_correction.xs.push((idx, i, j)); }
                                ErrorType::Z => { sparse_correction.zs.push((idx, i, j)); }
                                ErrorType::Y => { sparse_correction.xs.push((idx, i, j)); sparse_correction.zs.push((idx, i, j)); }
                                ErrorType::I => { }
                            }
                        }
                    }
                }
            }
        }
        let mut measurement_errors = Vec::new();
        let t_max = self.snapshot.len();
        for t in (12..t_max).step_by(6) {
            for &(i, j) in interested_region.iter() {
                let node = self.snapshot[t][i][j].as_ref().expect("exist");
                if node.qubit_type != QubitType::Data {
                    if self.is_measurement_error_at(t, i, j) {
                        measurement_errors.push((t, i, j));
                    }
                }
            }
        }
        self._clear_error_interested_region(&mut interested_region);  // recovery the state
        (sparse_correction, measurement_errors)
    }
    /// corresponds to `build_graph_given_error_rate` in `FaultTolerantView.vue`
    pub fn build_graph(&mut self) {
        let mut all_possible_errors: Vec<Either<ErrorType, CorrelatedErrorType>> = Vec::new();
        for error_type in ErrorType::all_possible_errors().drain(..) {
            all_possible_errors.push(Either::Left(error_type));
        }
        for correlated_error_type in CorrelatedErrorType::all_possible_errors().drain(..) {
            all_possible_errors.push(Either::Right(correlated_error_type));
        }
        // necessary to clear all errors and propagated errors to run `fast_measurement_given_few_errors`
        self.clear_error();
        self.propagate_error();
        // println!("{:?}", all_possible_errors);
        for t in 1..self.snapshot.len() {  // 0 doesn't generate error
            for i in 0..self.snapshot[t].len() {
                for j in 0..self.snapshot[t][i].len() {
                    if self.snapshot[t][i][j].is_some() {
                        for error in all_possible_errors.iter() {
                            let node = self.snapshot[t][i][j].as_ref().expect("exist");
                            let p = match error {
                                Either::Left(error_type) => match error_type {
                                    ErrorType::X => node.error_rate_x,
                                    ErrorType::Z => node.error_rate_z,
                                    ErrorType::Y => node.error_rate_y,
                                    _ => unreachable!()
                                },
                                Either::Right(error_type) => {
                                    match &node.correlated_error_model {
                                        Some(correlated_error_model) => {
                                            correlated_error_model.error_rate(error_type)
                                        },
                                        None => 0.,
                                    }
                                },
                            }; // probability of this error to occur
                            if p > 0. {
                                // simulate the error and measure it
                                let mut errors = Vec::new();
                                match error {
                                    Either::Left(error_type) => {
                                        errors.push((t, i, j, error_type.clone()));
                                    },
                                    Either::Right(error_type) => {
                                        errors.push((t, i, j, error_type.my_error()));
                                        let connection = self.snapshot[t][i][j].as_ref().expect("exist").connection
                                            .as_ref().expect("correlated error must corresponds to a two-qubit gate");
                                        let (ct, ci, cj) = (connection.t, connection.i, connection.j);
                                        errors.push((ct, ci, cj, error_type.peer_error()));
                                    },
                                }
                                let (sparse_correction, measurement_errors) = self.fast_measurement_given_few_errors(&errors);
                                if measurement_errors.len() == 0 {  // no way to detect it, ignore
                                    continue
                                }
                                if measurement_errors.len() > 2 {  // MWPM cannot handle this kind of error... just ignore
                                    // println!("[warning] single qubit error cause more than 2 measurement errors: {:?}", error);
                                    continue
                                }
                                // compute correction pattern, so that applying this error pattern will exactly recover data qubit errors
                                let correction = Arc::new(sparse_correction);
                                // add this to edges and update probability
                                if measurement_errors.len() == 1 {  // boundary
                                    let (t1, i1, j1) = measurement_errors[0];
                                    // println!("[{}][{}][{}]:[{}] causes boundary error on [{}][{}][{}]", t, i, j, if *error == ErrorType::X { "X" } else { "Z" }, t1, i1, j1);
                                    let node = self.snapshot[t1][i1][j1].as_mut().expect("exist");
                                    if node.boundary.is_none() {
                                        node.boundary = Some(Boundary {
                                            p: 0.,
                                            cases: Vec::new(),
                                        });
                                    }
                                    node.boundary.as_mut().expect("exist").add(p, correction, self.use_combined_probability);
                                } else if measurement_errors.len() == 2 {  // connection
                                    let (t1, i1, j1) = measurement_errors[0];
                                    let (t2, i2, j2) = measurement_errors[1];
                                    if self.snapshot[t1][i1][j1].as_ref().unwrap().qubit_type == self.snapshot[t2][i2][j2].as_ref().unwrap().qubit_type {
                                        // println!("[{}][{}][{}]:[{}] causes paired errors on [{}][{}][{}] and [{}][{}][{}]", t, i, j, if *error == ErrorType::X { "X" } else { "Z" }, t1, i1, j1, t2, i2, j2);
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
                                            node.boundary.as_mut().expect("exist").add(p, correction, self.use_combined_probability);
                                        } else {
                                            // println!("add_edge_case [{}][{}][{}] [{}][{}][{}] with p = {}", t1, i1, j1, t2, i2, j2, p);
                                            add_edge_case(&mut self.snapshot[t1][i1][j1].as_mut().expect("exist").edges, t2, i2, j2, p, correction.clone()
                                                , self.use_combined_probability);
                                            add_edge_case(&mut self.snapshot[t2][i2][j2].as_mut().expect("exist").edges, t1, i1, j1, p, correction
                                                , self.use_combined_probability);
                                        }
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
    fn optimize_correction_cases(original_cases: &Vec::<(Arc<SparseCorrection>, f64)>) -> Vec::<(Arc<SparseCorrection>, f64)> {
        let mut cases = HashMap::<SparseCorrection, f64>::new();
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
                // println!("add edge [{}][{}][{}] and [{}][{}][{}] with weight {}", t, i, j, edge.t, edge.i, edge.j, weight_of(edge.p));
            }
            // println!("[{}][{}][{}] boundary: {:?}", t, i, j, node.boundary);
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
                    // println!("[{}][{}][{}] insert [{}][{}][{}] with cost = {}", t, i, j, index.t, index.i, index.j, *cost);
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
                                        if node.qubit_type == node_b.qubit_type && (node_b.exhausted_map.get(&index).is_some() || (t == tb && i == ib && j == jb)) {
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
                            if min_cost.is_none() {
                                continue  // node not involved
                            }
                            let min_cost = min_cost.expect("exist");
                            let min_index = min_index.expect("exist");
                            // println!("boundary of [{}][{}][{}] {} {:?}", t, i, j, min_cost, min_index);
                            let node_b = self.snapshot[min_index.t][min_index.i][min_index.j].as_ref().expect("exist");
                            let mut correction: SparseCorrection = (*node_b.boundary.as_ref().expect("exist").cases[0].0).clone();
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
    pub fn get_correction_two_nodes(&self, a: &Index, b: &Index) -> SparseCorrection {
        let node_a = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist");
        let node_b = self.snapshot[b.t][b.i][b.j].as_ref().expect("exist");
        assert_eq!(node_a.gate_type, GateType::Measurement);
        assert_eq!(node_b.gate_type, GateType::Measurement);
        assert_eq!(node_a.qubit_type, node_b.qubit_type);  // so that it has a path
        if a == b {
            return self.generate_default_sparse_correction()
        }
        match &node_a.exhausted_map[&b].correction {
            Some(correction) => { (**correction).clone() }
            None => {
                let mut correction: SparseCorrection = (**node_a.exhausted_map[&b].next_correction.as_ref().expect("must call `build_exhausted_path`")).clone();
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
        let width_i = 2 * self.di - 1;
        let width_j = 2 * self.dj - 1;
        let mut measurement = Measurement(ndarray::Array::from_elem((self.MeasurementRounds + 1, width_i, width_j), false));
        let mut measurement_mut = measurement.view_mut();
        self.iterate_measurement_errors(|t, i, j, _node| {
            let (mt, mi, mj) = Index::new(t, i, j).to_measurement_idx();
            measurement_mut[[mt, mi, mj]] = true;
        });
        measurement
    }
    /// decode based on MWPM
    pub fn decode_MWPM(&self, measurement: &Measurement) -> (Correction, serde_json::Value) {
        let (sparse_correction, runtime_statistics) = self.decode_MWPM_sparse_correction(measurement);
        (Correction::from(&sparse_correction), runtime_statistics)
    }
    pub fn decode_MWPM_sparse_correction(&self, measurement: &Measurement) -> (SparseCorrection, serde_json::Value) {
        let (sparse_correction, runtime_statistics, _, _) = self.decode_MWPM_sparse_correction_with_edge_matchings(measurement);
        (sparse_correction, runtime_statistics)
    }
    pub fn decode_MWPM_sparse_correction_with_edge_matchings(&self, measurement: &Measurement) ->
            (SparseCorrection, serde_json::Value, Vec<((usize, usize, usize), (usize, usize, usize))>, Vec<(usize, usize, usize)>) {
        // sanity check
        let shape = measurement.shape();
        let width_i = 2 * self.di - 1;
        let width_j = 2 * self.dj - 1;
        assert_eq!(shape[0], self.MeasurementRounds + 1);
        assert_eq!(shape[1], width_i);
        assert_eq!(shape[2], width_j);
        // generate all the error measurements to be matched
        let mut to_be_matched = Vec::new();
        for mt in 0..self.MeasurementRounds + 1 {
            for mi in 0..width_i {
                for mj in 0..width_j {
                    if measurement[[mt, mi, mj]] {  // has a measurement error there
                        to_be_matched.push(Index::from_measurement_idx(mt, mi, mj));
                    }
                }
            }
        }
        // if to_be_matched.len() > 2 {
        //     println!{"TBM {:?}", to_be_matched};
        // }
        let mut correction = self.generate_default_sparse_correction();
        let mut edge_matchings = Vec::new();
        let mut boundary_matchings = Vec::new();
        let mut time_prepare_graph = 0.;
        let mut time_blossom_v = 0.;
        let mut time_constructing_correction = 0.;
        if to_be_matched.len() != 0 {
            let begin = Instant::now();
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
                match self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").exhausted_boundary.as_ref() {
                    Some(exhausted_boundary) => {
                        let cost = exhausted_boundary.cost;
                        weighted_edges.push((i, i + m_len, cost));
                    },
                    None => { }
                }
            }
            time_prepare_graph = begin.elapsed().as_secs_f64();
            // if to_be_matched.len() > 2 {
            //     println!{"node num {:?}, weighted edges {:?}", node_num, weighted_edges};
            // }
            let begin = Instant::now();
            let matching = blossom_v::safe_minimum_weight_perfect_matching(node_num, weighted_edges);
            time_blossom_v = begin.elapsed().as_secs_f64();
            // println!("{:?}", to_be_matched);
            // println!("matching: {:?}", matching);
            // if to_be_matched.len() > 2 {
            //     println!("matching: {:?}", matching);
            // }
            let begin = Instant::now();
            for i in 0..m_len {
                let j = matching[i];
                let a = &to_be_matched[i];
                if j < i {  // only add correction if j < i, so that the same correction is not applied twice
                    // println!("match peer {:?} {:?}", to_be_matched[i], to_be_matched[j]);
                    let b = &to_be_matched[j];
                    correction.combine(&self.get_correction_two_nodes(a, b));
                    edge_matchings.push(((a.t, a.i, a.j), (b.t, b.i, b.j)));
                } else if j >= m_len {  // matched with boundary
                    // println!("match boundary {:?}", to_be_matched[i]);
                    let node = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist");
                    correction.combine(node.exhausted_boundary.as_ref().expect("exist").correction.as_ref().expect("exist"));
                    boundary_matchings.push((a.t, a.i, a.j));
                }
            }
            time_constructing_correction = begin.elapsed().as_secs_f64();
            // if to_be_matched.len() > 2 {
            //     println!("correction: {:?}", correction);
            // }
        }
        (correction, json!({
            "to_be_matched": to_be_matched.len(),
            "time_prepare_graph": time_prepare_graph,
            "time_blossom_v": time_blossom_v,
            "time_constructing_correction": time_constructing_correction,
        }), edge_matchings, boundary_matchings)
    }
    
    /// decode based on UnionFind decoder
    pub fn decode_UnionFind(&self, measurement: &Measurement, max_half_weight: usize, use_distributed: bool, detailed_runtime_statistics: bool) -> (Correction, serde_json::Value) {
        let (sparse_correction, runtime_statistics) = self.decode_UnionFind_sparse_correction(measurement, max_half_weight, use_distributed, detailed_runtime_statistics);
        (Correction::from(&sparse_correction), runtime_statistics)
    }
    pub fn decode_UnionFind_sparse_correction(&self, measurement: &Measurement, max_half_weight: usize, use_distributed: bool, detailed_runtime_statistics: bool) -> (SparseCorrection, serde_json::Value) {
        let (sparse_correction, runtime_statistics, _, _) = self.decode_UnionFind_sparse_correction_with_edge_matchings(measurement, max_half_weight, use_distributed, detailed_runtime_statistics);
        (sparse_correction, runtime_statistics)
    }
    pub fn decode_UnionFind_sparse_correction_with_edge_matchings(&self, measurement: &Measurement, max_half_weight: usize, use_distributed: bool, detailed_runtime_statistics: bool) ->
            (SparseCorrection, serde_json::Value, Vec<((usize, usize, usize), (usize, usize, usize))>, Vec<(usize, usize, usize)>) {
        // sanity check
        let shape = measurement.shape();
        let width_i = 2 * self.di - 1;
        let width_j = 2 * self.dj - 1;
        assert_eq!(shape[0], self.MeasurementRounds + 1);
        assert_eq!(shape[1], width_i);
        assert_eq!(shape[2], width_j);
        // run union find decoder
        let (edge_matchings, boundary_matchings, runtime_statistics) = union_find_decoder::suboptimal_matching_by_union_find_given_measurement(&self
            , measurement, max_half_weight, use_distributed, detailed_runtime_statistics);
        let mut correction = self.generate_default_sparse_correction();
        for &((t1, i1, j1), (t2, i2, j2)) in edge_matchings.iter() {
            correction.combine(&self.get_correction_two_nodes(&Index::new(t1, i1, j1), &Index::new(t2, i2, j2)));
        }
        for &(t, i, j) in boundary_matchings.iter() {
            let node = self.snapshot[t][i][j].as_ref().expect("exist");
            correction.combine(node.exhausted_boundary.as_ref().expect("exist").correction.as_ref().expect("exist"));
        }
        (correction, runtime_statistics, edge_matchings, boundary_matchings)
    }

    /// decode based on approximate MWPM
    pub fn decode_MWPM_approx(&self, measurement: &Measurement, substreams: usize, use_modified: bool) -> Correction {
        Correction::from(&self.decode_MWPM_approx_sparse_correction(measurement, substreams, use_modified))
    }
    pub fn decode_MWPM_approx_sparse_correction(&self, measurement: &Measurement, substreams: usize, use_modified: bool) -> SparseCorrection {
        // sanity check
        let shape = measurement.shape();
        let width_i = 2 * self.di - 1;
        let width_j = 2 * self.dj - 1;
        assert_eq!(shape[0], self.MeasurementRounds + 1);
        assert_eq!(shape[1], width_i);
        assert_eq!(shape[2], width_j);
        // generate all the error measurements to be matched
        let mut to_be_matched = Vec::new();
        for mt in 0..self.MeasurementRounds + 1 {
            for mi in 0..width_i {
                for mj in 0..width_j {
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

            let matching = match use_modified {
                true => mwpm_approx::minimum_weight_perfect_matching_approx_modified(node_num, weighted_edges, substreams),
                false =>  mwpm_approx::minimum_weight_perfect_matching_approx(node_num, weighted_edges, substreams),
            };

            // println!("{:?}", to_be_matched);
            // println!("matching: {:?}", matching);
            // if to_be_matched.len() > 2 {
            //     println!("matching: {:?}", matching);
            // }
            let mut correction = self.generate_default_sparse_correction();
            for (i,j,_w) in matching.iter() {
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
            self.generate_default_sparse_correction()
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

    // return (x_error_count, z_error_count)
    pub fn get_boundary_cardinality(&self, correction: &Correction) -> (usize, usize) {
        let mut corrected = self.get_data_qubit_error_pattern();
        corrected.combine(&correction);  // apply correction to error pattern
        let mut x_error_count = 0;
        let mut z_error_count = 0;
        match self.code_type {
            CodeType::StandardPlanarCode => {
                // Z stabilizer boundary, j = 0
                for i in 0..self.di {
                    if corrected.x[[self.MeasurementRounds, (i*2), 0]] {
                        x_error_count += 1;
                    }
                }
                // X stabilizer boundary, i = 0
                for j in 0..self.dj {
                    if corrected.z[[self.MeasurementRounds, 0, (j*2)]] {
                        z_error_count += 1;
                    }
                }
            },
            CodeType::StandardXZZXCode => {
                // logical Z boundary, j = 0
                for i in 0..self.di {
                    if corrected.z[[self.MeasurementRounds, (i*2), 0]] {
                        z_error_count += 1;
                    }
                }
                // logical X boundary, i = 0
                for j in 0..self.dj {
                    if corrected.x[[self.MeasurementRounds, 0, (j*2)]] {
                        x_error_count += 1;
                    }
                }
                // println!("z_error_count: {}, x_error_count: {}", z_error_count, x_error_count);
            },
            CodeType::RotatedXZZXCode => {
                assert_eq!(self.di, self.dj, "rotated XZZX code doesn't support rectangle lattice right now");
                let middle_point = self.di - 1;
                for delta in 0..self.di {
                    let has_error = if delta % 2 == 0 {
                        corrected.z[[self.MeasurementRounds, delta, middle_point + delta]]
                    } else {
                        corrected.x[[self.MeasurementRounds, delta, middle_point + delta]]
                    };
                    if has_error {
                        z_error_count += 1;
                    }
                }
                for delta in 0..self.di {
                    let has_error = if delta % 2 == 0 {
                        corrected.x[[self.MeasurementRounds, middle_point - delta, delta]]
                    } else {
                        corrected.z[[self.MeasurementRounds, middle_point - delta, delta]]
                    };
                    if has_error {
                        x_error_count += 1;
                    }
                }
            },
            _ => unimplemented!("boundary validation not implemented for this code type")
        }
        (x_error_count, z_error_count)
    }
    /// validate correction on the boundaries of top layer with perfect measurement. should be equivalent to `validate_correction_on_top_layer`
    pub fn validate_correction_on_boundary(&self, correction: &Correction) -> Result<(), ValidationFailedReason> {
        let (x_error_count, z_error_count) = self.get_boundary_cardinality(correction);
        match (x_error_count % 2 != 0, z_error_count % 2 != 0) {
            (true, true) => Err(ValidationFailedReason::BothXandZLogicalError(0, x_error_count, 0, z_error_count, 0)),
            (true, false) => Err(ValidationFailedReason::XLogicalError(0, x_error_count, 0)),
            (false, true) => Err(ValidationFailedReason::ZLogicalError(0, z_error_count, 0)),
            _ => Ok(())
        }
    }

    pub fn apply_error_model(&mut self, error_model: &ErrorModel, p: f64, bias_eta: f64) {
        match error_model {
            ErrorModel::GenericBiasedWithBiasedCX | ErrorModel::GenericBiasedWithStandardCX => {
                let height = self.snapshot.len();
                self.iterate_snapshot_mut(|t, _i, _j, node| {
                    // first clear error rate
                    node.error_rate_x = 0.;
                    node.error_rate_z = 0.;
                    node.error_rate_y = 0.;
                    if t >= height - 6 {  // no error on the top, as a perfect measurement round
                        return
                    } else if t <= 6 {
                        return  // perfect initialization
                    }
                    // do different things for each stage
                    let stage = Stage::from(t);
                    match stage {
                        Stage::Initialization => {
                            // note that error rate at measurement round will NOT cause measurement errors
                            //     to add measurement errors, need to be Stage::CXGate4
                            node.error_rate_x = p / bias_eta;
                            node.error_rate_z = p;
                            node.error_rate_y = p / bias_eta;
                        },
                        Stage::CXGate1 | Stage::CXGate2 | Stage::CXGate3 | Stage::CXGate4 => {
                            if stage == Stage::CXGate4 && node.qubit_type != QubitType::Data {  // add measurement errors (p + p/bias_eta)
                                node.error_rate_x = p / bias_eta;
                                node.error_rate_z = p;
                                node.error_rate_y = p / bias_eta;
                            }
                            match node.gate_type {
                                GateType::ControlledPhase => {
                                    if node.qubit_type != QubitType::Data {  // this is ancilla
                                        // better check whether peer is indeed data qubit, but it's hard here due to Rust's borrow check
                                        let mut correlated_error_model = CorrelatedErrorModel::default_with_probability(p / bias_eta);
                                        correlated_error_model.error_rate_ZI = p;
                                        correlated_error_model.error_rate_IZ = p;
                                        correlated_error_model.sanity_check();
                                        node.correlated_error_model = Some(correlated_error_model);
                                    }
                                },
                                GateType::Control => {  // this is ancilla in XZZX code, see arXiv:2104.09539v1
                                    let mut correlated_error_model = CorrelatedErrorModel::default_with_probability(p / bias_eta);
                                    correlated_error_model.error_rate_ZI = p;
                                    match error_model {
                                        ErrorModel::GenericBiasedWithStandardCX => {
                                            correlated_error_model.error_rate_IZ = 0.375 * p;
                                            correlated_error_model.error_rate_ZZ = 0.375 * p;
                                            correlated_error_model.error_rate_IY = 0.125 * p;
                                            correlated_error_model.error_rate_ZY = 0.125 * p;
                                        },
                                        ErrorModel::GenericBiasedWithBiasedCX => {
                                            correlated_error_model.error_rate_IZ = 0.5 * p;
                                            correlated_error_model.error_rate_ZZ = 0.5 * p;
                                        },
                                    }
                                    correlated_error_model.sanity_check();
                                    node.correlated_error_model = Some(correlated_error_model);
                                },
                                _ => { }
                            }
                        },
                        Stage::Measurement => { }  // do nothing
                    }
                });
            },
        }
    }

    #[allow(dead_code)]
    pub fn print_direct_connections(&self) {
        self.iterate_snapshot(|t, i, j, node| {
            if Stage::from(t) == Stage::Measurement && node.qubit_type != QubitType::Data {
                println!("[{}][{}][{}]: {:?}", t, i, j, node.qubit_type);
                match &node.boundary {
                    Some(boundary) => println!("boundary: p = {}", boundary.p),
                    None => println!("boundary: none"),
                }
                for edge in node.edges.iter() {
                    println!("edge [{}][{}][{}]: p = {}", edge.t, edge.i, edge.j, edge.p);
                }
            }
        });
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

/// Gate type, corresponds to `NTYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone)]
pub enum GateType {
    // initialization
    Initialization,
    // CX gate
    Control,
    Target,
    // measurement
    Measurement,
    // CZ gate or CPHASE gate
    ControlledPhase,
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
    pub cases: Vec::<(Arc<SparseCorrection>, f64)>,
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

pub fn add_edge_case(edges: &mut Vec::<Edge>, t: usize, i: usize, j: usize, p: f64, correction: Arc<SparseCorrection>, use_combined_probability: bool) {
    for edge in edges.iter_mut() {
        if edge.t == t && edge.i == i && edge.j == j {
            edge.add(p, correction, use_combined_probability);
            return  // already found
        }
    }
    let mut edge = Edge {
        t: t, i: i, j: j, p: 0.,
        cases: Vec::new(),
    };
    edge.add(p, correction, use_combined_probability);
    edges.push(edge);
}

impl Edge {
    pub fn add(&mut self, p: f64, correction: Arc<SparseCorrection>, use_combined_probability: bool) {
        if use_combined_probability {
            self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        } else {
            self.p = self.p.max(p);  // max
        }
        self.cases.push((correction, p));
    }
}

/// Boundary Information, corresponds to `node.boundary` in `FaultTolerantView.vue`
#[derive(Debug, Clone)]
pub struct Boundary {
    pub p: f64,
    pub cases: Vec::<(Arc<SparseCorrection>, f64)>,
}

impl Boundary {
    pub fn add(&mut self, p: f64, correction: Arc<SparseCorrection>, use_combined_probability: bool) {
        if use_combined_probability {
            self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        } else {
            self.p = self.p.max(p);  // max
        }
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

impl Correction {
    pub fn new_all_false(t_max: usize, i_max: usize, j_max: usize) -> Self {
        Self {
            x: ndarray::Array::from_elem((t_max, i_max, j_max), false),
            z: ndarray::Array::from_elem((t_max, i_max, j_max), false),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SparseCorrection {
    // for each element (t, i, j), errors happen at (t, i, j), (t+1, i, j), (t+2, i, j) ...
    pub shape: (usize, usize, usize),
    pub xs: Vec<(usize, usize, usize)>,
    pub zs: Vec<(usize, usize, usize)>,
}

impl From<&Correction> for SparseCorrection {
    fn from(correction: &Correction) -> Self {
        let shape = correction.x.shape();
        assert_eq!(shape, correction.z.shape());
        let mut xs = Vec::new();
        let mut zs = Vec::new();
        for k in 0..2 {
            let changes = if k == 0 { &mut xs } else { &mut zs };
            let pattern = if k == 0 { &correction.x } else { &correction.z };
            for i in 0..shape[1] {
                for j in 0..shape[2] {
                    if pattern[[0, i, j]] {
                        changes.push((0, i, j));
                    }
                }
            }
            for t in 1..shape[0] {
                for i in 0..shape[1] {
                    for j in 0..shape[2] {
                        if pattern[[t, i, j]] != pattern[[t-1, i, j]] {
                            changes.push((t, i, j));
                        }
                    }
                }
            }
        }
        SparseCorrection {
            xs: xs,
            zs: zs,
            shape: (shape[0], shape[1], shape[2]),
        }
    }
}

impl SparseCorrection {
    pub fn new_all_false(t_max: usize, i_max: usize, j_max: usize) -> Self {
        Self {
            shape: (t_max, i_max, j_max),
            xs: Vec::new(),
            zs: Vec::new(),
        }
    }
    pub fn combine(&mut self, next: &Self) {
        self.xs.extend(next.xs.clone());
        self.zs.extend(next.zs.clone());
    }
}

impl From<&SparseCorrection> for Correction {
    fn from(correction: &SparseCorrection) -> Self {
        let (t_max, i_max, j_max) = correction.shape;
        let mut x = ndarray::Array::from_elem((t_max, i_max, j_max), false);
        let mut z = ndarray::Array::from_elem((t_max, i_max, j_max), false);
        for k in 0..2 {
            let changes = if k == 0 { &correction.xs } else { &correction.zs };
            let pattern_ro = if k == 0 { &mut x } else { &mut z };
            let mut pattern = pattern_ro.view_mut();
            for (ts, i, j) in changes.iter() {
                for t in *ts..t_max {
                    pattern[[t, *i, *j]] = !pattern[[t, *i, *j]];
                }
            }
        }
        Correction {
            x: x,
            z: z,
        }
    }
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
    pub correction: Option< Arc<SparseCorrection> >,
    /// `next_correction` is generated by default
    pub next_correction: Option< Arc<SparseCorrection> >,
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::ErrorType;

    // use `cargo test xzzx_code_test_simulation_1 -- --nocapture` to run specific test

    #[test]
    fn xzzx_code_test_simulation_1() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let mut model = PlanarCodeModel::new_standard_XZZX_code(measurement_rounds, d);
        model.set_phenomenological_error_with_perfect_initialization(p);
        model.build_graph();
        let assert_error_is = |model: &mut PlanarCodeModel, errors| {
            model.propagate_error();
            let mut measurement_errors = Vec::new();
            model.iterate_measurement_errors(|t, i, j, _node| {
                measurement_errors.push((t, i, j));
            });
            // println!("{:?}", measurement_errors);
            assert_eq!(measurement_errors, errors);
        };
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        // single X error on the top boundary
        model.clear_error();
        model.add_error_at(el2t(0), 0, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2)]);
        // single Z error on the top boundary
        model.clear_error();
        model.add_error_at(el2t(0), 0, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 0, 1), (24, 0, 3)]);
        // single X error in the middle
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 3, 2)]);
        // single Z error in the middle
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 2, 1), (24, 2, 3)]);
        // 2 X errors
        model.clear_error();
        model.add_error_at(el2t(0), 0, 2, &ErrorType::X).expect("error rate = 0 here");
        model.add_error_at(el2t(0), 2, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 3, 2)]);
        // Z logical errors
        model.clear_error();
        model.add_error_at(el2t(0), 0, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.add_error_at(el2t(0), 0, 2, &ErrorType::Z).expect("error rate = 0 here");
        model.add_error_at(el2t(0), 0, 4, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
        // X logical errors
        model.clear_error();
        model.add_error_at(el2t(0), 0, 0, &ErrorType::X).expect("error rate = 0 here");
        model.add_error_at(el2t(0), 2, 0, &ErrorType::X).expect("error rate = 0 here");
        model.add_error_at(el2t(0), 4, 0, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
    }

    /*
     * Note 2021.7.4
     * MWPM decoder is even worse than union-find decoder... very strange
     * use the command "cargo run --release -- test union_find_decoder_xzzx_code_study 7 0.05 -c10 --max_half_weight 4 --bias_eta 10",
     *    I found that MWPM can fail even if there is only 3 errors under a d=7 XZZX code.
     * There must be some bug here, so this test case helps me to find it
     *
     * The bug is simple.... when I build the exhaustive boundary cost, I didn't consider the direct boundary
     * changing:
     * origin:  if node.qubit_type == node_b.qubit_type && node_b.exhausted_map.get(&index).is_some() {
     *    new:  if node.qubit_type == node_b.qubit_type && (node_b.exhausted_map.get(&index).is_some() || (t == tb && i == ib && j == jb)) {
     */
    #[test]
    fn xzzx_code_test_decoder_1() {
        let p = 0.05;
        let bias_eta = 10.;
        let L = 7;
        let mut model = PlanarCodeModel::new_standard_XZZX_code(1, L);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        model.set_individual_error(0., 0., 0.);  // clear all errors
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t == 12 && node.qubit_type == QubitType::Data {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
            }
        });
        model.build_graph();
        model.optimize_correction_pattern();
        model.build_exhausted_path_autotune();
        // add errors
        model.add_error_at(12, 0, 0, &ErrorType::Z).expect("error rate = 0 here");
        model.add_error_at(12, 0, 12, &ErrorType::Z).expect("error rate = 0 here");
        model.add_error_at(12, 9, 7, &ErrorType::Z).expect("error rate = 0 here");
        model.propagate_error();
        let measurement = model.generate_measurement();
        let (correction, _) = model.decode_MWPM(&measurement);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        assert!(validation_ret.is_ok(), "only 3 errors should not break code distance = 7");
    }

}
