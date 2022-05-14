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
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use super::blossom_v;
use std::sync::{Arc};
use super::types::{QubitType, ErrorType, CorrelatedPauliErrorType, CorrelatedPauliErrorRates, ErrorModelName, CorrelatedErasureErrorRates};
use super::union_find_decoder;
use super::either::Either;
use super::serde_json;
use std::time::Instant;
use super::fast_benchmark::FastBenchmark;
use serde::Serialize;
use super::util;
use super::util::simple_hasher::SimpleHasher;
use super::union_find::UnionFind;

/// uniquely index a node
/// update 2022.3.13: remove hash support for index; if use hash, change to FastHashIndex
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
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
    pub fn distance(&self, other: &Self) -> usize {
        ((self.t as isize - other.t as isize).abs() + (self.i as isize - other.i as isize).abs() + (self.j as isize - other.j as isize).abs()) as usize
    }
}

impl Ord for Index {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.t.cmp(&other.t) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => {
                match self.i.cmp(&other.i) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => self.j.cmp(&other.j),
                }
            }
        }
    }
}

impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Eq, Clone, Copy)]
pub struct FastHashIndex {
    pub max_i: usize,
    pub max_j: usize,
    pub index: Index,
}

/// profiling tells me the default hasher is super slow... here's my own faster hasher leveraging the fact that i and j are typically under 16 bits and t is usually no more than 32 bits
impl std::hash::Hash for FastHashIndex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let combined = ((self.index.t / 6) * self.max_i * self.max_j + self.index.i * self.max_j + self.index.j) as u64;
        combined.hash(state);
    }
}

impl PartialEq for FastHashIndex {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index  // max_t,i,j doesn't matter when comparing
    }
}

impl FastHashIndex {
    pub fn with_di_dj(index: &Index, di: usize, dj: usize) -> Self {
        Self {
            max_i: 2 * di - 1,
            max_j: 2 * dj - 1,
            index: *index,
        }
    }
}

impl Ord for FastHashIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl PartialOrd for FastHashIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Autotune: compute weight based on error model
pub fn weight_autotune(p: f64) -> f64 {
    if p > 0. { - p.ln() } else { f64::from(f32::MAX) }  // use f32::MAX is enough
}
pub fn weight_autotune_minus_no_error(p: f64) -> f64 {
    if p > 0. { (1.-p).ln() - p.ln() } else { f64::from(f32::MAX) }  // use f32::MAX is enough
}
/// Manhattan distance (but not exactly because there is 12 neighbors instead of 8) version
pub fn weight_equal(p: f64) -> f64 {
    if p > 0. { 1. } else { f64::from(f32::MAX) }  // use f32::MAX is enough
}

/// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub t: usize,
    pub i: usize,
    pub j: usize,
    pub connection: Option<Connection>,
    /// note that correlated error is applied to next time step, without losing generality
    pub correlated_error_model: Option<CorrelatedPauliErrorRates>,
    pub correlated_erasure_error_model: Option<CorrelatedErasureErrorRates>,
    pub gate_type: GateType,
    pub qubit_type: QubitType,
    #[serde(skip)]
    pub error: ErrorType,
    #[serde(skip)]
    pub has_erasure: bool,
    pub error_rate_x: f64,
    pub error_rate_z: f64,
    pub error_rate_y: f64,
    pub erasure_error_rate: f64,
    #[serde(skip)]
    pub propagated: ErrorType,
    // connections generated by X, Z and Y errors
    #[serde(skip)]
    pub pauli_error_connections: Vec<Either<(Index, Index), Index>>,  // either connection between stabilizers or to boundary
    // graph information
    #[serde(skip)]
    pub edges: Vec::<Edge>,
    #[serde(skip)]
    pub boundary: Option<Boundary>,  // if connects to boundary in graph, this is the probability
    #[serde(skip)]
    pub exhausted_boundary: Option<ExhaustedElement>,
    #[serde(skip)]
    pub pet_node: Option<petgraph::graph::NodeIndex>,  // temporary parameter
    #[serde(skip)]
    // pub exhausted_map: HashMap<Index, ExhaustedElement>,  // note: only when the key is `Index` should I use SimpleHasher
    pub exhausted_map: HashMap<FastHashIndex, ExhaustedElement, std::hash::BuildHasherDefault::<SimpleHasher> >,
    // internal states used for temporary manipulation
    #[serde(skip)]
    pub disable_in_random_error_generator: bool,
    // specific for tailored surface code, where decoding high probability 4 non-trivial measurement errors are essential: arXiv:1907.02554v2
    #[serde(skip)] pub tailored_positive_edges: Vec::<Edge>,  // edges that connects stabilizers (x, y) and (x+1, y+1)
    #[serde(skip)] pub tailored_positive_boundary: Option<Boundary>,
    #[serde(skip)] pub tailored_positive_exhausted_boundary: Option<ExhaustedElement>,
    #[serde(skip)] pub tailored_positive_exhausted_map: HashMap<FastHashIndex, ExhaustedElement, std::hash::BuildHasherDefault::<SimpleHasher> >,
    #[serde(skip)] pub tailored_negative_edges: Vec::<Edge>,  // edges that connects stabilizers (x, y) and (x+1, y-1)
    #[serde(skip)] pub tailored_negative_boundary: Option<Boundary>,
    #[serde(skip)] pub tailored_negative_exhausted_boundary: Option<ExhaustedElement>,
    #[serde(skip)] pub tailored_negative_exhausted_map: HashMap<FastHashIndex, ExhaustedElement, std::hash::BuildHasherDefault::<SimpleHasher> >,
}

impl Node {
    fn __new_default(t: usize, i: usize, j: usize, gate_type: GateType, qubit_type: QubitType, exhausted_map_capacity: usize, enabled_tailored_decoding: bool) -> Self {
        let scaled_exhausted_map_capacity = exhausted_map_capacity / 8;
        Self {
            t: t, i: i, j: j,
            connection: None,
            correlated_error_model: None,
            correlated_erasure_error_model: None,
            gate_type: gate_type,
            qubit_type: qubit_type,
            error: ErrorType::I,
            has_erasure: false,
            error_rate_x: 0.25,  // by default error rate is the highest
            error_rate_z: 0.25,
            error_rate_y: 0.25,
            erasure_error_rate: 0.5,
            propagated: ErrorType::I,
            pauli_error_connections: Vec::new(),
            edges: Vec::new(),
            boundary: None,
            exhausted_boundary: None,
            pet_node: None,
            exhausted_map: HashMap::with_capacity_and_hasher(scaled_exhausted_map_capacity, std::hash::BuildHasherDefault::<SimpleHasher>::default()),
            disable_in_random_error_generator: false,
            tailored_positive_edges: Vec::new(),
            tailored_positive_boundary: None,
            tailored_positive_exhausted_boundary: None,
            tailored_positive_exhausted_map: HashMap::with_capacity_and_hasher(if enabled_tailored_decoding { scaled_exhausted_map_capacity } else { 0 }, std::hash::BuildHasherDefault::<SimpleHasher>::default()),
            tailored_negative_edges: Vec::new(),
            tailored_negative_boundary: None,
            tailored_negative_exhausted_boundary: None,
            tailored_negative_exhausted_map: HashMap::with_capacity_and_hasher(if enabled_tailored_decoding { scaled_exhausted_map_capacity } else { 0 }, std::hash::BuildHasherDefault::<SimpleHasher>::default()),
        }
    }
}

/// record the code type
#[derive(Debug, Clone, Serialize)]
pub enum CodeType {
    StandardPlanarCode,
    RotatedPlanarCode,
    StandardXZZXCode,
    RotatedXZZXCode,
    StandardTailoredCode,
    RotatedTailoredCode,
    Unknown,
}

/// The structure of surface code, including how quantum gates are implemented
#[derive(Debug, Clone, Serialize)]
pub struct PlanarCodeModel {
    pub code_type: CodeType,
    /// Corresponds to `this.snapshot` in `FaultTolerantView.vue`
    pub snapshot: Vec::< Vec::< Vec::< Option<Node> > > >,
    pub di: usize,  // code distance of i dimension
    pub dj: usize,  // code distance of i dimension
    pub MeasurementRounds: usize,
    pub T: usize,
    #[serde(skip)]
    pub use_combined_probability: bool,
    #[serde(skip)]
    pub use_reduced_graph: bool,  // feature that remove edge between two vertices if both of them have smaller weight matching to boundary than matching each other
    /// for each line, XOR the result. Only if no less than half of the result is 1.
    /// We do this because stabilizer operators will definitely have all 0 (because it generate 2 or 0 errors on every homology lines, XOR = 0)
    /// Only logical error will pose all 1 results, but sometimes single qubit errors will "hide" the logical error (because it
    ///    makes some result to 0), thus we determine there's a logical error if no less than half of the results are 1
    #[serde(skip)]
    z_homology_lines: Vec< Vec::<(usize, usize)> >,
    #[serde(skip)]
    x_homology_lines: Vec< Vec::<(usize, usize)> >,
    // define boundary
    #[serde(skip)]
    pub enabled_tailored_decoding: bool,  // by default to false, use special method to decode tailored surface code arXiv:1907.02554v2
}

impl PlanarCodeModel {
    fn __new_default(code_type: CodeType, snapshot: Vec::< Vec::< Vec::< Option<Node> > > >, di: usize, dj: usize, MeasurementRounds: usize, T: usize) -> Self {
        Self {
            code_type: code_type,
            snapshot: snapshot,
            di: di,
            dj: dj,
            T: T,
            MeasurementRounds: MeasurementRounds,
            use_combined_probability: true,  // this feature is stable, enable it by default (2022.3.13 Yue)
            use_reduced_graph: true,  // this feature is stable, enable it by default (2022.3.13 Yue)
            z_homology_lines: Vec::new(),
            x_homology_lines: Vec::new(),
            enabled_tailored_decoding: false,
        }
    }
    #[inline(always)]
    pub fn fhi(&self, index: Index) -> FastHashIndex {
        FastHashIndex::with_di_dj(&index, self.di, self.dj)
    }
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
        let exhausted_map_capacity = width_i * width_j * T;
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
                                if i % 2 == 0 {  // for Z stabilizers, operate with the data qubit on the left ("Z" shape)
                                    if qubit_type == QubitType::Data {
                                        if j+1 < width_j && filter(i, j+1) {
                                            gate_type = GateType::Control;
                                            connection = Some(Connection{ t: t, i: i, j: j+1 });
                                        }
                                    } else {
                                        if j >= 1 && filter(i, j-1) {
                                            gate_type = GateType::Target;
                                            connection = Some(Connection{ t: t, i: i, j: j-1 });
                                        }
                                    }
                                } else {  // for X stabilizers, operate with the data qubit on the right ("S" shape)
                                    if qubit_type == QubitType::Data {
                                        if j >= 1 && filter(i, j-1) {
                                            gate_type = GateType::Target;
                                            connection = Some(Connection{ t: t, i: i, j: j-1 });
                                        }
                                    } else {
                                        if j+1 < width_i && filter(i, j+1) {
                                            gate_type = GateType::Control;
                                            connection = Some(Connection{ t: t, i: i, j: j+1 });
                                        }
                                    }
                                }
                            },
                            Stage::CXGate3 => {
                                if i % 2 == 0 {  // for Z stabilizers, operate with the data qubit on the right ("Z" shape)
                                    if qubit_type == QubitType::Data {
                                        if j >= 1 && filter(i, j-1) {
                                            gate_type = GateType::Control;
                                            connection = Some(Connection{ t: t, i: i, j: j-1 });
                                        }
                                    } else {
                                        if j+1 < width_i && filter(i, j+1) {
                                            gate_type = GateType::Target;
                                            connection = Some(Connection{ t: t, i: i, j: j+1 });
                                        }
                                    }
                                } else {  // for X stabilizers, operate with the data qubit on the right ("S" shape)
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
                        let mut node = Node::__new_default(t, i, j, gate_type, qubit_type, if qubit_type != QubitType::Data && stage == Stage::Measurement { exhausted_map_capacity } else { 0 }, false);
                        node.connection = connection;
                        snapshot_row_1.push(Some(node));
                    } else {
                        snapshot_row_1.push(None);
                    }
                }
                snapshot_row_0.push(snapshot_row_1);
            }
            snapshot.push(snapshot_row_0);
        }
        let code = Self::__new_default(code_type, snapshot, di, dj, MeasurementRounds, T);
        code
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
        let exhausted_map_capacity = width_i * width_j * T;
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
                        let mut node = Node::__new_default(t, i, j, gate_type, qubit_type, if qubit_type != QubitType::Data && stage == Stage::Measurement { exhausted_map_capacity } else { 0 }, false);
                        node.connection = connection;
                        snapshot_row_1.push(Some(node));
                    } else {
                        snapshot_row_1.push(None);
                    }
                }
                snapshot_row_0.push(snapshot_row_1);
            }
            snapshot.push(snapshot_row_0);
        }
        let code = Self::__new_default(code_type, snapshot, di, dj, MeasurementRounds, T);
        code
    }

    pub fn new_standard_tailored_code(MeasurementRounds: usize, L: usize) -> Self {
        // MeasurementRounds = 0 means only one perfect measurement round
        assert!(L >= 2, "at lease one stabilizer is required");
        let model = Self::new_tailored_code(CodeType::StandardTailoredCode, MeasurementRounds, L, L, |_i, _j| true);
        model
    }

    pub fn new_rotated_tailored_code(MeasurementRounds: usize, L: usize) -> Self {
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
            if i % 2 == 0 {  // Y stabilizers
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
        let model = Self::new_tailored_code(CodeType::RotatedTailoredCode, MeasurementRounds, L, L, filter);
        model
    }

    pub fn new_tailored_code<F>(code_type: CodeType, MeasurementRounds: usize, di: usize, dj: usize, filter: F) -> Self
            where F: Fn(usize, usize) -> bool {
        let width_i = 2 * di - 1;
        let width_j = 2 * dj - 1;
        let T = MeasurementRounds + 2;
        let height = T * 6 + 1;
        let exhausted_map_capacity = width_i * width_j * T;
        let mut snapshot = Vec::with_capacity(height);
        for t in 0..height {
            let mut snapshot_row_0 = Vec::with_capacity(width_i);
            for i in 0..width_i {
                let mut snapshot_row_1 = Vec::with_capacity(width_j);
                for j in 0..width_j {
                    if filter(i, j) {
                        let stage = Stage::from(t);
                        let qubit_type = if (i + j) % 2 == 0 { QubitType::Data } else { if i % 2 == 0 { QubitType::StabY } else { QubitType::StabX } };
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
                                        gate_type = if j % 2 == 0 { GateType::Target } else { GateType::TargetCY };
                                        connection = Some(Connection{ t: t, i: i+1, j: j });
                                    }
                                } else {
                                    if i >= 1 && filter(i-1, j) {
                                        gate_type = if j % 2 == 0 { GateType::Control } else { GateType::ControlCY };
                                        connection = Some(Connection{ t: t, i: i-1, j: j });
                                    }
                                }
                            },
                            Stage::CXGate2 => {
                                if i % 2 == 0 {  // for Y stabilizers, operate with the data qubit on the left ("Z" shape)
                                    if qubit_type == QubitType::Data {
                                        if j+1 < width_j && filter(i, j+1) {
                                            gate_type = GateType::TargetCY;
                                            connection = Some(Connection{ t: t, i: i, j: j+1 });
                                        }
                                    } else {
                                        if j >= 1 && filter(i, j-1) {
                                            gate_type = GateType::ControlCY;
                                            connection = Some(Connection{ t: t, i: i, j: j-1 });
                                        }
                                    }
                                } else {  // for X stabilizers, operate with the data qubit on the right ("S" shape)
                                    if qubit_type == QubitType::Data {
                                        if j >= 1 && filter(i, j-1) {
                                            gate_type = GateType::Target;
                                            connection = Some(Connection{ t: t, i: i, j: j-1 });
                                        }
                                    } else {
                                        if j+1 < width_i && filter(i, j+1) {
                                            gate_type = GateType::Control;
                                            connection = Some(Connection{ t: t, i: i, j: j+1 });
                                        }
                                    }
                                }
                            },
                            Stage::CXGate3 => {
                                if i % 2 == 0 {  // for Y stabilizers, operate with the data qubit on the right ("Z" shape)
                                    if qubit_type == QubitType::Data {
                                        if j >= 1 && filter(i, j-1) {
                                            gate_type = GateType::TargetCY;
                                            connection = Some(Connection{ t: t, i: i, j: j-1 });
                                        }
                                    } else {
                                        if j+1 < width_i && filter(i, j+1) {
                                            gate_type = GateType::ControlCY;
                                            connection = Some(Connection{ t: t, i: i, j: j+1 });
                                        }
                                    }
                                } else {  // for X stabilizers, operate with the data qubit on the right ("S" shape)
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
                                }
                            },
                            Stage::CXGate4 => {
                                if qubit_type == QubitType::Data {
                                    if i >= 1 && filter(i-1, j) {
                                        gate_type = if j % 2 == 0 { GateType::Target} else { GateType::TargetCY};
                                        connection = Some(Connection{ t: t, i: i-1, j: j });
                                    }
                                } else {
                                    if i+1 < width_j && filter(i+1, j) {
                                        gate_type = if j % 2 == 0 { GateType::Control} else { GateType::ControlCY};
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
                        let mut node = Node::__new_default(t, i, j, gate_type, qubit_type, if qubit_type != QubitType::Data && stage == Stage::Measurement { exhausted_map_capacity } else { 0 }, true);
                        node.connection = connection;
                        snapshot_row_1.push(Some(node));
                    } else {
                        snapshot_row_1.push(None);
                    }
                }
                snapshot_row_0.push(snapshot_row_1);
            }
            snapshot.push(snapshot_row_0);
        }
        let mut code = Self::__new_default(code_type, snapshot, di, dj, MeasurementRounds, T);
        code.enabled_tailored_decoding = true;
        code
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
    pub fn set_individual_error_with_erasure(&mut self, px: f64, py: f64, pz: f64, pe: f64) {
        let height = self.snapshot.len();
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            if t >= height - 6 {  // no error on the top, as a perfect measurement round
                node.error_rate_x = 0.;
                node.error_rate_z = 0.;
                node.error_rate_y = 0.;
                node.erasure_error_rate = 0.;
            } else {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
                node.erasure_error_rate = pe;
            }
        })
    }
    pub fn set_individual_error(&mut self, px: f64, py: f64, pz: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p(1-p)
        self.set_individual_error_with_erasure(px, py, pz, 0.)
    }
    pub fn set_depolarizing_error(&mut self, error_rate: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p(1-p)
        self.set_individual_error(error_rate, error_rate, error_rate)
    }
    // this will remove bottom boundary
    pub fn set_individual_error_with_perfect_initialization_with_erasure(&mut self, px: f64, py: f64, pz: f64, pe: f64) {
        assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
        assert!(pe <= 1. && pe >= 0.);
        let height = self.snapshot.len();
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            if t >= height - 6 {  // no error on the top, as a perfect measurement round
                node.error_rate_x = 0.;
                node.error_rate_z = 0.;
                node.error_rate_y = 0.;
                node.erasure_error_rate = 0.;
            } else if t <= 6 {
                node.error_rate_x = 0.;
                node.error_rate_z = 0.;
                node.error_rate_y = 0.;
                node.erasure_error_rate = 0.;
            } else {
                node.error_rate_x = px;
                node.error_rate_z = pz;
                node.error_rate_y = py;
                node.erasure_error_rate = pe;
            }
        })
    }
    pub fn set_individual_error_with_perfect_initialization(&mut self, px: f64, py: f64, pz: f64) {
        self.set_individual_error_with_perfect_initialization_with_erasure(px, py, pz, 0.)
    }
    pub fn set_depolarizing_error_with_perfect_initialization(&mut self, error_rate: f64) {  // (1-3p)I + pX + pZ + pY: X error rate = Z error rate = 2p
        self.set_individual_error_with_perfect_initialization(error_rate, error_rate, error_rate)
    }
    // remove bottom boundary, (1-p)^2I + p(1-p)X + p(1-p)Z + p^2Y
    pub fn set_phenomenological_error_with_perfect_initialization(&mut self, error_rate: f64) {
        self.set_phenomenological_error_with_perfect_initialization_with_erasure(error_rate, 0.)
    }
    pub fn set_phenomenological_error_with_perfect_initialization_with_erasure(&mut self, error_rate: f64, pe: f64) {
        let height = self.snapshot.len();
        self.iterate_snapshot_mut(|t, _i, _j, node| {
            node.error_rate_x = 0.;
            node.error_rate_z = 0.;
            node.error_rate_y = 0.;
            node.erasure_error_rate = 0.;
            // no error on the top and bottom
            if t < height - 6 && t > 6 {
                let next_stage = Stage::from(t + 1);
                match next_stage {
                    Stage::Measurement => {
                        node.error_rate_x = error_rate * (1. - error_rate);
                        node.error_rate_z = error_rate * (1. - error_rate);
                        node.error_rate_y = error_rate * error_rate;
                        node.erasure_error_rate = pe
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
        let mut pending_erasure_errors = Vec::new();
        self.iterate_snapshot_mut(|t, i, j, node| {
            if node.disable_in_random_error_generator {
                node.error = ErrorType::I;
                node.has_erasure = false;
                return
            }
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
            let random_number = rng();
            if random_number < node.erasure_error_rate {
                node.has_erasure = true;  // apply erasure error after correlated error
            } else {
                node.has_erasure = false;
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
            match &node.correlated_erasure_error_model {
                Some(correlated_erasure_error_model) => {
                    let random_number = rng();
                    let correlated_erasure_error_type = correlated_erasure_error_model.generate_random_erasure_error(random_number);
                    let my_error = correlated_erasure_error_type.my_error();
                    if my_error {
                        pending_erasure_errors.push((t, i, j));
                    }
                    let peer_error = correlated_erasure_error_type.peer_error();
                    if peer_error {
                        let connection = node.connection.as_ref().expect("correlated erasure error must corresponds to a two-qubit gate");
                        let (ct, ci, cj) = (connection.t, connection.i, connection.j);
                        pending_erasure_errors.push((ct, ci, cj));
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
        // apply pending erasure errors
        for (t, i, j) in pending_erasure_errors.drain(..) {
            let mut node = self.snapshot[t][i][j].as_mut().expect("exist");
            node.has_erasure = true;
        }
        // apply erasure error and then count number of errors
        let mut error_count = 0;
        // println!("print erasure error pattern start");
        self.iterate_snapshot_mut(|_t, _i, _j, node| {
            if node.has_erasure {  // apply erasure error as equal probability of all kinds of Pauli errors
                let random_number = rng();
                node.error = if random_number < 0.25 { ErrorType::X }
                    else if random_number < 0.5 { ErrorType::Z }
                    else if random_number < 0.75 { ErrorType::Y }
                    else { ErrorType::I };
                // println!("random erasure error at [{}][{}][{}], type: {:?}", _t, _i, _j, node.error);
            }
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
            if node.error != ErrorType::I || node.has_erasure {
                count += 1;
            }
        });
        count
    }

    pub fn print_errors(&self) {
        self.iterate_snapshot(|t, i, j, node| {
            if node.error != ErrorType::I || node.has_erasure {
                println!("{:?} at {} {} {} {}", node.error, t, i, j, node.has_erasure);
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
                                ErrorType::X => node.error_rate_x,
                                ErrorType::Z => node.error_rate_z,
                                // Y error requires both x and z has corresponding edge
                                ErrorType::Y => node.error_rate_y,
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

    pub fn add_correlated_error_at(&mut self, t: usize, i: usize, j: usize, error: &CorrelatedPauliErrorType) -> Option<(ErrorType, ErrorType)> {
        let peer = if let Some(array) = self.snapshot.get_mut(t) {
            if let Some(array) = array.get_mut(i) {
                if let Some(element) = array.get_mut(j) {
                    match element {
                        Some(ref mut node) => {
                            let p = match node.correlated_error_model {
                                Some(ref correlated_error_model) => {
                                    correlated_error_model.error_rate(error)
                                },
                                None => 0.
                            };
                            if p > 0. {  // only add error if physical error rate is greater than 0.
                                let my_error = error.my_error();
                                let peer_error = error.peer_error();
                                let connection = node.connection.as_ref().expect("correlated error must corresponds to a two-qubit gate");
                                let (ct, ci, cj) = (connection.t, connection.i, connection.j);
                                node.error = node.error.multiply(&my_error);
                                Some((node.error.clone(), ct, ci, cj, peer_error.clone()))
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
        };
        if let Some((node_error, ct, ci, cj, peer_error)) = peer {
            let peer_node = self.snapshot[ct][ci][cj].as_mut().unwrap();
            peer_node.error = peer_node.error.multiply(&peer_error);
            Some((node_error, peer_node.error.clone()))
        } else {
            None
        }
    }

    pub fn add_random_erasure_error_at<F>(&mut self, t: usize, i: usize, j: usize, mut rng: F) -> Option<ErrorType> where F: FnMut() -> f64 {
        let random_number = rng();
        let error = if random_number < 0.25 { ErrorType::X }
            else if random_number < 0.5 { ErrorType::Z }
            else if random_number < 0.75 { ErrorType::Y }
            else { ErrorType::I };
        self.add_erasure_error_at(t, i, j, &error)
    }

    pub fn add_erasure_error_at(&mut self, t: usize, i: usize, j: usize, error: &ErrorType) -> Option<ErrorType> {
        if let Some(array) = self.snapshot.get_mut(t) {
            if let Some(array) = array.get_mut(i) {
                if let Some(element) = array.get_mut(j) {
                    match element {
                        Some(ref mut node) => {
                            let pe = node.erasure_error_rate;
                            if pe > 0. {  // only add erasure error if error rate is greater than 0.
                                node.error = node.error.multiply(error);
                                node.has_erasure = true;
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
        match gate_type {
            GateType::Control => {  // X propagated to other qubits' X through CX gate
                let connection = node_connection.as_ref().expect("exist");
                if node_propagated == ErrorType::X || node_propagated == ErrorType::Y {
                    let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                    peer_node.propagated = peer_node.propagated.multiply(&ErrorType::X);
                    propagated_neighbor = Some((connection.i, connection.j));
                }
            }
            GateType::Target => {  // Z propagated to other qubits' Z through CX gate
                let connection = node_connection.as_ref().expect("exist");
                if node_propagated == ErrorType::Z || node_propagated == ErrorType::Y {
                    let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                    peer_node.propagated = peer_node.propagated.multiply(&ErrorType::Z);
                    propagated_neighbor = Some((connection.i, connection.j));
                }
            }
            GateType::ControlledPhase => {  // X propagated to other qubits' Z via CZ gate
                let connection = node_connection.as_ref().expect("exist");
                if node_propagated == ErrorType::X || node_propagated == ErrorType::Y {
                    let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                    peer_node.propagated = peer_node.propagated.multiply(&ErrorType::Z);
                    propagated_neighbor = Some((connection.i, connection.j));
                }
            }
            GateType::ControlCY => {  // Y propagated to other qubits' Y through CX gate
                let connection = node_connection.as_ref().expect("exist");
                if node_propagated == ErrorType::X || node_propagated == ErrorType::Y {
                    let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                    peer_node.propagated = peer_node.propagated.multiply(&ErrorType::Y);
                    propagated_neighbor = Some((connection.i, connection.j));
                }
            }
            GateType::TargetCY => {  // Z propagated to other qubits' Z through CX gate
                let connection = node_connection.as_ref().expect("exist");
                if node_propagated == ErrorType::X || node_propagated == ErrorType::Z {
                    let peer_node = self.snapshot[t+1][connection.i][connection.j].as_mut().expect("exist");
                    peer_node.propagated = peer_node.propagated.multiply(&ErrorType::Z);
                    propagated_neighbor = Some((connection.i, connection.j));
                }
            }
            GateType::Initialization | GateType::Measurement | GateType::None => {
                // not propagate
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
            QubitType::StabX | QubitType::StabXZZXLogicalX | QubitType::StabXZZXLogicalZ | QubitType::StabY => {
                assert_eq!(node.gate_type, GateType::Measurement);
                let this_result = node.propagated == ErrorType::I || node.propagated == ErrorType::X;
                let last_node = self.snapshot[t-6][i][j].as_ref().expect("exist");
                let last_result = last_node.propagated == ErrorType::I || last_node.propagated == ErrorType::X;
                this_result != last_result
            },
            QubitType::Data => unreachable!(),
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

    /// get all errors (t, i, j, error, has_erasure)
    pub fn get_all_qubit_errors_vec(&self) -> Vec<(usize, usize, usize, ErrorType, bool)> {
        let mut errors_vec = Vec::new();
        self.iterate_snapshot(|t, i, j, node| {
            if node.error != ErrorType::I || node.has_erasure {
                errors_vec.push((t, i, j, node.error.clone(), node.has_erasure));
            }
        });
        errors_vec
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
    pub fn build_graph<F>(&mut self, weight_of: F) where F: Fn(f64) -> f64 + Copy {
        self.build_graph_fast_benchmark(weight_of, false);
    }
    pub fn build_graph_with_fast_benchmark<F>(&mut self, weight_of: F) -> FastBenchmark where F: Fn(f64) -> f64 + Copy {
        self.build_graph_fast_benchmark(weight_of, true).unwrap()
    }
    pub fn build_graph_fast_benchmark<F>(&mut self, weight_of: F, build_fast_benchmark: bool) -> Option<FastBenchmark> where F: Fn(f64) -> f64 + Copy {
        let mut fast_benchmark = None;
        if build_fast_benchmark {
            fast_benchmark = Some(FastBenchmark::new(&self));
        }
        let mut all_possible_errors: Vec<Either<ErrorType, CorrelatedPauliErrorType>> = Vec::new();
        for error_type in ErrorType::all_possible_errors().drain(..) {
            all_possible_errors.push(Either::Left(error_type));
        }
        for correlated_error_type in CorrelatedPauliErrorType::all_possible_errors().drain(..) {
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
                        self.snapshot[t][i][j].as_mut().expect("exist").pauli_error_connections.clear();
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
                            let possible_erasure_error = node.erasure_error_rate > 0. || node.correlated_erasure_error_model.is_some() || {
                                match self.snapshot[t][i][j].as_ref().expect("exist").connection.as_ref() {
                                    Some(connection) => {
                                        let (ct, ci, cj) = (connection.t, connection.i, connection.j);
                                        self.snapshot[ct][ci][cj].as_ref().expect("exist").correlated_erasure_error_model.clone().and_then(|v| if v.error_probability() > 0. {Some(true)} else {None}).is_some()
                                    },
                                    None => false,
                                }
                            };
                            let is_erasure = possible_erasure_error && error.is_left();
                            if p > 0. || is_erasure {  // always run single Pauli errors to build `pauli_error_connections`
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
                                // compute correction pattern, so that applying this error pattern will exactly recover data qubit errors
                                let correction = Arc::new(sparse_correction);
                                // add this to edges and update probability
                                if measurement_errors.len() == 1 {  // boundary
                                    let (t1, i1, j1) = measurement_errors[0];
                                    // println!("[{}][{}][{}]:[{}] causes boundary error on [{}][{}][{}]", t, i, j, if *error == ErrorType::X { "X" } else { "Z" }, t1, i1, j1);
                                    if p > 0. || is_erasure {
                                        let node = self.snapshot[t1][i1][j1].as_mut().expect("exist");
                                        if node.boundary.is_none() {
                                            node.boundary = Some(Boundary {
                                                p: 0.,
                                                weight: f64::MAX,
                                                cases: Vec::new(),
                                            });
                                        }
                                        node.boundary.as_mut().expect("exist").add(p, correction.clone(), self.use_combined_probability, weight_of);
                                    }
                                    if is_erasure {  // only consider single qubit errors
                                        let node = self.snapshot[t][i][j].as_mut().expect("exist");
                                        node.pauli_error_connections.push(Either::Right(Index::new(t1, i1, j1)));
                                    }
                                } else if measurement_errors.len() == 2 {  // connection
                                    let (t1, i1, j1) = measurement_errors[0];
                                    let (t2, i2, j2) = measurement_errors[1];
                                    let is_same_type = self.snapshot[t1][i1][j1].as_ref().unwrap().qubit_type == self.snapshot[t2][i2][j2].as_ref().unwrap().qubit_type;
                                    // currently do not consider fully connected version between X and Z graph (which shouldn't have too much difference in terms of decoding accuracy)
                                    // if enabled tailored decoding, this type of cross-type connection is allowed
                                    if is_same_type {
                                        if p > 0. || is_erasure {
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
                                                        weight: f64::MAX,
                                                        cases: Vec::new(),
                                                    });
                                                }
                                                node.boundary.as_mut().expect("exist").add(p, correction.clone(), self.use_combined_probability, weight_of);
                                            } else {
                                                // println!("add_edge_case [{}][{}][{}] [{}][{}][{}] with p = {}", t1, i1, j1, t2, i2, j2, p);
                                                add_edge_case(&mut self.snapshot[t1][i1][j1].as_mut().expect("exist").edges, t2, i2, j2, p, correction.clone()
                                                    , self.use_combined_probability, weight_of);
                                                add_edge_case(&mut self.snapshot[t2][i2][j2].as_mut().expect("exist").edges, t1, i1, j1, p, correction.clone()
                                                    , self.use_combined_probability, weight_of);
                                            }
                                        }
                                        if is_erasure {  // only consider single qubit errors causing same type of errors
                                            let node = self.snapshot[t][i][j].as_mut().expect("exist");
                                            node.pauli_error_connections.push(Either::Left((Index::new(t1, i1, j1), Index::new(t2, i2, j2))));
                                        }
                                    } else {
                                        if self.enabled_tailored_decoding {
                                            // TODO: add them to positive/negative decoding graph accordingly
                                            // or it's unnecessary? in i.i.d. error model, this edge will eventually be added by adjacent Z error
                                        }
                                    }
                                }  // MWPM cannot handle this kind of error... just ignore
                                if self.enabled_tailored_decoding && (measurement_errors.len() == 3 || measurement_errors.len() == 4) {
                                    // tailored surface code decoding method can handle special cases arXiv:1907.02554v2
                                    // first find the individual median i and j, then (i, j) must be the center data qubit
                                    let mut vec_i = Vec::new();
                                    let mut vec_j = Vec::new();
                                    for &(_tm, im, jm) in measurement_errors.iter() {
                                        vec_i.push(im);
                                        vec_j.push(jm);
                                    }
                                    let center_i = util::find_strict_one_median(&mut vec_i);
                                    let center_j = util::find_strict_one_median(&mut vec_j);
                                    let mut unknown_case_warning = false;
                                    match (center_i, center_j) {
                                        (Some(center_i), Some(center_j)) => {
                                            let mut up = None;
                                            let mut down = None;
                                            let mut left = None;
                                            let mut right = None;
                                            let mut counter = 0;
                                            for &(tm, im, jm) in measurement_errors.iter() {
                                                if im + 1 == center_i && jm == center_j {
                                                    up = Some((tm, im, jm));
                                                    counter += 1;
                                                }
                                                if im == center_i + 1 && jm == center_j {
                                                    down = Some((tm, im, jm));
                                                    counter += 1;
                                                }
                                                if im == center_i && jm == center_j + 1 {
                                                    right = Some((tm, im, jm));
                                                    counter += 1;
                                                }
                                                if im == center_i && jm + 1 == center_j {
                                                    left = Some((tm, im, jm));
                                                    counter += 1;
                                                }
                                            }
                                            if counter == measurement_errors.len() {
                                                // add them to `tailored_positive_edges` and `tailored_negative_edges`
                                                {  // positive: up + right, left + down
                                                    for (A, B) in [(up, right), (left, down)] {
                                                        match (A, B) {
                                                            (Some((t1, i1, j1)), Some((t2, i2, j2))) => {
                                                                // println!("add_edge_case tailored_positive_edges [{}][{}][{}] [{}][{}][{}] with p = {}", t1, i1, j1, t2, i2, j2, p);
                                                                add_edge_case(&mut self.snapshot[t1][i1][j1].as_mut().expect("exist").tailored_positive_edges, t2, i2, j2, p, correction.clone()
                                                                    , self.use_combined_probability, weight_of);
                                                                add_edge_case(&mut self.snapshot[t2][i2][j2].as_mut().expect("exist").tailored_positive_edges, t1, i1, j1, p, correction.clone()
                                                                    , self.use_combined_probability, weight_of);
                                                            }
                                                            (Some((tm, im, jm)), None) | (None, Some((tm, im, jm))) => {  // add to boundary
                                                                let node = self.snapshot[tm][im][jm].as_mut().expect("exist");
                                                                if node.tailored_positive_boundary.is_none() {
                                                                    node.tailored_positive_boundary = Some(Boundary {
                                                                        p: 0.,
                                                                        weight: f64::MAX,
                                                                        cases: Vec::new(),
                                                                    });
                                                                }
                                                                node.tailored_positive_boundary.as_mut().expect("exist").add(p, correction.clone(), self.use_combined_probability, weight_of);
                                                            }
                                                            _ => { unreachable!() }
                                                        }
                                                    }
                                                }
                                                {  // negative: left + up, down + right
                                                    for (A, B) in [(left, up), (down, right)] {
                                                        match (A, B) {
                                                            (Some((t1, i1, j1)), Some((t2, i2, j2))) => {
                                                                // println!("add_edge_case tailored_negative_edges [{}][{}][{}] [{}][{}][{}] with p = {}", t1, i1, j1, t2, i2, j2, p);
                                                                add_edge_case(&mut self.snapshot[t1][i1][j1].as_mut().expect("exist").tailored_negative_edges, t2, i2, j2, p, correction.clone()
                                                                    , self.use_combined_probability, weight_of);
                                                                add_edge_case(&mut self.snapshot[t2][i2][j2].as_mut().expect("exist").tailored_negative_edges, t1, i1, j1, p, correction.clone()
                                                                    , self.use_combined_probability, weight_of);
                                                            }
                                                            (Some((tm, im, jm)), None) | (None, Some((tm, im, jm))) => {  // add to boundary
                                                                let node = self.snapshot[tm][im][jm].as_mut().expect("exist");
                                                                if node.tailored_negative_boundary.is_none() {
                                                                    node.tailored_negative_boundary = Some(Boundary {
                                                                        p: 0.,
                                                                        weight: f64::MAX,
                                                                        cases: Vec::new(),
                                                                    });
                                                                }
                                                                node.tailored_negative_boundary.as_mut().expect("exist").add(p, correction.clone(), self.use_combined_probability, weight_of);
                                                            }
                                                            _ => { unreachable!() }
                                                        }
                                                    }
                                                }
                                            } else {
                                                unknown_case_warning = true;  // cannot fit them in
                                            }
                                        }
                                        _ => {
                                            unknown_case_warning = true;
                                        }
                                    }
                                    if unknown_case_warning {  // this cases seem to be normal for circuit-level noise model of tailored surface code: Pauli Y would generate some strange cases, but those are low-biased errors
                                        // println!("error at {:?}: cannot recognize the pattern of this 3 or 4 non-trivial measurements, strange... just skipped", (t, i, j, error));
                                        // for i in 0..measurement_errors.len() {
                                        //     let (tm, im, jm) = measurement_errors[i];
                                        //     print!("t{}: {:?}, ", i, (tm, im, jm));
                                        // }
                                        // println!("");
                                    }
                                }
                                // update fast benchmark
                                if build_fast_benchmark && measurement_errors.len() >= 1 {
                                    let (t0, i0, j0) = measurement_errors[0];
                                    let node0 = self.snapshot[t0][i0][j0].as_ref().unwrap();
                                    let mut group_1 = Vec::new();
                                    let mut group_2 = Vec::new();
                                    for &(tm, im, jm) in measurement_errors.iter() {
                                        let nodem = self.snapshot[tm][im][jm].as_ref().unwrap();
                                        if node0.qubit_type == nodem.qubit_type {
                                            group_1.push((tm, im, jm));
                                        } else {
                                            group_2.push((tm, im, jm));
                                        }
                                    }
                                    let joint_erasure_error_rate = {
                                        let node = self.snapshot[t][i][j].as_ref().expect("exist");
                                        let mut erasure_error_rate = node.erasure_error_rate;
                                        match &node.correlated_erasure_error_model {
                                            Some(correlated_model) => {
                                                let sub_error_rate = correlated_model.error_rate_EI + correlated_model.error_rate_EE;
                                                erasure_error_rate = 1. - (1. - sub_error_rate) * (1. - erasure_error_rate);
                                            }, None => { }
                                        }
                                        match node.connection.as_ref() {
                                            Some(connection) => {
                                                let (ct, ci, cj) = (connection.t, connection.i, connection.j);
                                                match &self.snapshot[ct][ci][cj].as_ref().expect("exist").correlated_erasure_error_model {
                                                    Some(correlated_model) => {
                                                        let sub_error_rate = correlated_model.error_rate_EI + correlated_model.error_rate_EE;
                                                        erasure_error_rate = 1. - (1. - sub_error_rate) * (1. - erasure_error_rate);
                                                    }, None => { }
                                                }
                                            }, None => { },
                                        }
                                        erasure_error_rate
                                    };
                                    for group in [group_1, group_2].iter() {
                                        if group.len() == 1 {
                                            let (t1, i1, j1) = group[0];
                                            if p > 0. {
                                                fast_benchmark.as_mut().unwrap().add_possible_boundary(t1, i1, j1, p, t, i, j, Either::Left(error.clone()));
                                            }
                                            if is_erasure && joint_erasure_error_rate > 0. {  // fast benchmark doesn't consider correlated erasure error
                                                fast_benchmark.as_mut().unwrap().add_possible_boundary(t1, i1, j1, joint_erasure_error_rate, t, i, j, Either::Right(()));
                                            }
                                        } else if group.len() == 2 {
                                            let (t1, i1, j1) = group[0];
                                            let (t2, i2, j2) = group[1];
                                            if p > 0. {
                                                fast_benchmark.as_mut().unwrap().add_possible_match(t1, i1, j1, t2, i2, j2, p, t, i, j, Either::Left(error.clone()));
                                            }
                                            if is_erasure && joint_erasure_error_rate > 0. {  // fast benchmark doesn't consider correlated erasure error
                                                fast_benchmark.as_mut().unwrap().add_possible_match(t1, i1, j1, t2, i2, j2, joint_erasure_error_rate, t, i, j, Either::Right(()));
                                            }
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
        fast_benchmark
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
        optimized_cases.sort_by(|(_, p1), (_, p2)| p2.partial_cmp(p1).expect("probabilities shouldn't be NaN"));
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
    /// bug fix 2021.12.20: building correction based on "next" may cause infinite loop, due to zero weight paths (e.g. erasure) that can jump between two nodes
    pub fn build_exhausted_path(&mut self) {
        let di = self.di;
        let dj = self.dj;
        let fhi = |index: Index| -> FastHashIndex {
            FastHashIndex::with_di_dj(&index, di, dj)
        };
        // first build petgraph
        let mut graph = petgraph::graph::Graph::new_undirected();
        // add nodes before adding edge, so that they all have node number
        self.iterate_measurement_stabilizers_mut(|t, i, j, node| {
            node.pet_node = Some(graph.add_node(fhi(Index {
                t: t, i: i, j: j
            })));
        });
        // then add every edge
        self.iterate_measurement_stabilizers(|t, i, j, node| {
            for edge in &node.edges {
                let node_target = self.snapshot[edge.t][edge.i][edge.j].as_ref().expect("exist").pet_node.expect("exist");
                graph.add_edge(node.pet_node.expect("exist"), node_target, PetGraphEdge {
                    a: self.fhi(Index { t: t, i: i, j: j }),
                    b: self.fhi(Index { t: edge.t, i: edge.i, j: edge.j }),
                    weight: edge.weight,  // so that w1 + w2 = - log(p1) - log(p2) = - log(p1*p2) = - log(p_line)
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
                let fh_index = graph.node_weight(*node_id).expect("exist");
                if fh_index != &(fhi(Index{ t: t, i: i, j: j })) { // do not add map to itself
                    node.exhausted_map.insert(*fh_index, ExhaustedElement {
                        cost: *cost,
                        next: None,
                        correction: None,
                        next_correction: None,
                        removed: false,
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
                            let fh_target_indexes: Vec::<FastHashIndex> = node.exhausted_map.keys().cloned().collect();
                            for fh_target_index in fh_target_indexes {
                                // find the next element by searching in `edges`
                                let node = self.snapshot[t][i][j].as_ref().expect("exist");
                                let mut min_cost: Option<f64> = None;
                                let mut min_index: Option<Index> = None;
                                for edge in &node.edges {
                                    let fh_next_index = self.fhi(Index::from(edge));
                                    let mut current_cost = node.exhausted_map[&fh_next_index].cost;
                                    if fh_next_index != fh_target_index {
                                        let next_node = self.snapshot[fh_next_index.index.t][fh_next_index.index.i][fh_next_index.index.j].as_ref().expect("exist");
                                        current_cost += next_node.exhausted_map[&fh_target_index].cost;
                                    }
                                    // compute the cost of node -> next_index -> target_index
                                    match min_cost.clone() {
                                        Some(min_cost_value) => {
                                            if current_cost < min_cost_value || (current_cost == min_cost_value && fh_target_index.index.distance(&fh_next_index.index) < fh_target_index.index.distance(&min_index.unwrap())) {
                                                min_cost = Some(current_cost);
                                                min_index = Some(fh_next_index.index);
                                            }
                                        }
                                        None => {
                                            min_cost = Some(current_cost);
                                            min_index = Some(fh_next_index.index);
                                        }
                                    }
                                }
                                // redefine node as a mutable one
                                let node = self.snapshot[t][i][j].as_mut().expect("exist");
                                node.exhausted_map.get_mut(&fh_target_index).expect("exist").next = Some(min_index.expect("exist"));
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
                            let fh_target_indexes: Vec::<FastHashIndex> = node.exhausted_map.keys().cloned().collect();
                            for fh_target_index in fh_target_indexes {
                                // go along `next` and combine over the `correction`
                                let this_index = Index{ t: t, i: i, j: j };
                                let this_node = self.snapshot[this_index.t][this_index.i][this_index.j].as_ref().expect("exist");
                                let next_index = this_node.exhausted_map[&fh_target_index].next.as_ref().expect("exist");
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
                                node.exhausted_map.get_mut(&fh_target_index).expect("exist").next_correction = Some(correction);
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
                                        if node.qubit_type == node_b.qubit_type && (node_b.exhausted_map.get(&self.fhi(index)).is_some() || (t == tb && i == ib && j == jb)) {
                                            let cost = boundary.weight + (if t == tb && i == ib && j == jb { 0. } else {
                                                node_b.exhausted_map[&self.fhi(index)].cost
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
                                correction.combine(node_b.exhausted_map[&self.fhi(index)].next_correction.as_ref().expect("exist"));
                            }
                            // redefine node as a mutable one
                            let node = self.snapshot[t][i][j].as_mut().expect("exist");
                            node.exhausted_boundary = Some(ExhaustedElement {
                                cost: min_cost,
                                next: Some(min_index),
                                correction: Some(Arc::new(correction)),
                                next_correction: None,
                                removed: false,
                            });
                        }
                    }
                }
            }
        }
        // if `use_reduced_graph` is enabled, remove edge between two vertices if both of them have smaller weight matching to boundary than matching each other
        if self.use_reduced_graph {
            for t in (12..self.snapshot.len()).step_by(6) {
                for i in 0..self.snapshot[t].len() {
                    for j in 0..self.snapshot[t][i].len() {
                        if self.snapshot[t][i][j].is_some() {
                            if self.snapshot[t][i][j].as_ref().expect("exist").gate_type == GateType::Measurement {
                                let node = self.snapshot[t][i][j].as_ref().expect("exist");
                                let index = Index::new(t, i, j);
                                let fh_target_indexes: Vec::<FastHashIndex> = node.exhausted_map.keys().cloned().collect();
                                let mut fh_to_be_removed = Vec::<FastHashIndex>::new();
                                if node.exhausted_boundary.is_none() {
                                    continue  // node not connected to boundary
                                }
                                let boundary_cost = node.exhausted_boundary.as_ref().unwrap().cost;
                                for fh_target_index in fh_target_indexes {
                                    let target_node = self.snapshot[fh_target_index.index.t][fh_target_index.index.i][fh_target_index.index.j].as_ref().expect("exist");
                                    if target_node.exhausted_boundary.is_none() {
                                        continue  // target node not connected to boundary
                                    }
                                    let target_boundary_cost = target_node.exhausted_boundary.as_ref().unwrap().cost;
                                    let need_remove = {
                                        if target_node.exhausted_map.contains_key(&self.fhi(index)) {
                                            let match_cost = target_node.exhausted_map[&self.fhi(index)].cost;
                                            boundary_cost + target_boundary_cost < match_cost
                                        } else {
                                            true  // always remove if the peer doesn't have it, because it's removed in the previous iterations already checked the above attribute
                                        }
                                    };
                                    if need_remove {
                                        fh_to_be_removed.push(fh_target_index);
                                    }
                                }
                                // for remove_index in to_be_removed {
                                //     println!("remove edge of [{}][{}][{}] and [{}][{}][{}]", t, i, j, remove_index.t, remove_index.i, remove_index.j);
                                // }
                                let node = self.snapshot[t][i][j].as_mut().expect("exist");
                                for fh_remove_index in fh_to_be_removed {
                                    node.exhausted_map.get_mut(&fh_remove_index).unwrap().removed = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        if self.enabled_tailored_decoding {
            macro_rules! build_tailored_exhausted_map_no_correction {
                // `()` indicates that the macro takes no argument.
                ($tailored_edges:ident, $tailored_boundary:ident, $tailored_exhausted_boundary:ident, $tailored_exhausted_map:ident) => {
                    let mut graph = petgraph::graph::Graph::new_undirected();
                    // add nodes before adding edge, so that they all have node number
                    self.iterate_measurement_stabilizers_mut(|t, i, j, node| {
                        node.pet_node = Some(graph.add_node(fhi(Index {
                            t: t, i: i, j: j
                        })));
                    });
                    // then add every edge
                    self.iterate_measurement_stabilizers(|t, i, j, node| {
                        for edge in &node.edges {  // these edges are added no matter in both positive and negative graphs
                            let node_target = self.snapshot[edge.t][edge.i][edge.j].as_ref().expect("exist").pet_node.expect("exist");
                            graph.add_edge(node.pet_node.expect("exist"), node_target, PetGraphEdge {
                                a: self.fhi(Index { t: t, i: i, j: j }),
                                b: self.fhi(Index { t: edge.t, i: edge.i, j: edge.j }),
                                weight: edge.weight,  // so that w1 + w2 = - log(p1) - log(p2) = - log(p1*p2) = - log(p_line)
                                // we want p_line to be as large as possible, it meets the goal of minimizing -log(p) 
                            });
                            // println!("add edge [{}][{}][{}] and [{}][{}][{}] with weight {}", t, i, j, edge.t, edge.i, edge.j, weight_of(edge.p));
                        }
                        for edge in &node.$tailored_edges {  // specific to positive or negative graph
                            let node_target = self.snapshot[edge.t][edge.i][edge.j].as_ref().expect("exist").pet_node.expect("exist");
                            graph.add_edge(node.pet_node.expect("exist"), node_target, PetGraphEdge {
                                a: self.fhi(Index { t: t, i: i, j: j }),
                                b: self.fhi(Index { t: edge.t, i: edge.i, j: edge.j }),
                                weight: edge.weight / 2.,  // Special: in tailored edges, each error generates 2 pairs of matchings, so each weight should be half of total and summing up with be the original
                                // we want p_line to be as large as possible, it meets the goal of minimizing -log(p) 
                            });
                            // println!("add edge [{}][{}][{}] and [{}][{}][{}] with weight {}", t, i, j, edge.t, edge.i, edge.j, weight_of(edge.p));
                        }
                        // println!("[{}][{}][{}] tailored_boundary: {:?}", t, i, j, node.$tailored_boundary);
                    });
                    // then run dijkstra for every node
                    self.iterate_measurement_stabilizers_mut(|t, i, j, node| {
                        let map = petgraph::algo::dijkstra(&graph, node.pet_node.expect("exist"), None, |e| e.weight().weight);
                        for (node_id, cost) in map.iter() {
                            let fh_index = graph.node_weight(*node_id).expect("exist");
                            if fh_index != &(fhi(Index{ t: t, i: i, j: j })) { // do not add map to itself
                                node.$tailored_exhausted_map.insert(*fh_index, ExhaustedElement {
                                    cost: *cost,
                                    next: None,
                                    correction: None,
                                    next_correction: None,
                                    removed: false,
                                });
                                // println!("[{}][{}][{}] insert [{}][{}][{}] with cost = {}", t, i, j, index.t, index.i, index.j, *cost);
                            }
                        }
                    });
                    // generate exhaust tailored_boundary without building correction pattern
                    for t in (12..self.snapshot.len()).step_by(6) {
                        for i in 0..self.snapshot[t].len() {
                            for j in 0..self.snapshot[t][i].len() {
                                if self.snapshot[t][i][j].is_some() {
                                    if self.snapshot[t][i][j].as_ref().expect("exist").gate_type == GateType::Measurement {
                                        let mut min_cost = None;
                                        let mut min_index = None;
                                        let index = Index::new(t, i, j);
                                        self.iterate_measurement_stabilizers(|tb, ib, jb, node_b| {
                                            match &node_b.boundary {
                                                Some(boundary) => {
                                                    // only try if this node is directly connected to boundary
                                                    if node_b.$tailored_exhausted_map.get(&self.fhi(index)).is_some() || (t == tb && i == ib && j == jb) {
                                                        let cost = boundary.weight + (if t == tb && i == ib && j == jb { 0. } else {
                                                            node_b.$tailored_exhausted_map[&self.fhi(index)].cost
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
                                            match &node_b.$tailored_boundary {
                                                Some(tailored_boundary) => {
                                                    // only try if this node is directly connected to boundary
                                                    if node_b.$tailored_exhausted_map.get(&self.fhi(index)).is_some() || (t == tb && i == ib && j == jb) {
                                                        let cost = tailored_boundary.weight / 2. + (if t == tb && i == ib && j == jb { 0. } else { // bug fix 2022.3.20: only tailored_boundary.weight should be half, not all expression because exhausted_map is already a half cost
                                                            node_b.$tailored_exhausted_map[&self.fhi(index)].cost
                                                        });  // Special: in tailored edges, each error generates 2 pairs of matchings, so each weight should be half of total and summing up with be the original
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
                                        // println!("tailored_boundary of [{}][{}][{}] {} {:?}", t, i, j, min_cost, min_index);
                                        // redefine node as a mutable one
                                        let node = self.snapshot[t][i][j].as_mut().expect("exist");
                                        node.$tailored_exhausted_boundary = Some(ExhaustedElement {
                                            cost: min_cost,
                                            next: Some(min_index),
                                            correction: None,
                                            next_correction: None,
                                            removed: false,
                                        });
                                    }
                                }
                            }
                        }
                    }
                };
            }
            // println!("positive");
            build_tailored_exhausted_map_no_correction!(tailored_positive_edges, tailored_positive_boundary, tailored_positive_exhausted_boundary, tailored_positive_exhausted_map);
            // println!("negative");
            build_tailored_exhausted_map_no_correction!(tailored_negative_edges, tailored_negative_boundary, tailored_negative_exhausted_boundary, tailored_negative_exhausted_map);
        }
    }
    /// get correction from two matched nodes
    /// use `correction` (or `next_correction` if former not provided) in `exhausted_map`
    pub fn get_correction_two_nodes(&self, a: &Index, b: &Index) -> SparseCorrection {
        let node_a = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist");
        let node_b = self.snapshot[b.t][b.i][b.j].as_ref().expect("exist");
        assert_eq!(node_a.gate_type, GateType::Measurement);
        assert_eq!(node_b.gate_type, GateType::Measurement);
        // if !self.enabled_tailored_decoding {  // this requirement doesn't exist in a tailored surface code decoding; commented out 2022.3.18: even in tailored surface code, this should not happen
        assert_eq!(node_a.qubit_type, node_b.qubit_type);  // so that it has a path
        // }
        if a == b {
            return self.generate_default_sparse_correction()
        }
        match &node_a.exhausted_map[&self.fhi(*b)].correction {
            Some(correction) => { (**correction).clone() }
            None => {
                let mut correction: SparseCorrection = (**node_a.exhausted_map[&self.fhi(*b)].next_correction.as_ref().expect("must call `build_exhausted_path`")).clone();
                let mut next_index = node_a.exhausted_map[&self.fhi(*b)].next.as_ref().expect("exist");
                let mut loop_counter = 10000;  // should not exceed 10000 for a path, this is used to detect infinite loop
                while next_index != b && loop_counter > 0 {
                    let this_node = self.snapshot[next_index.t][next_index.i][next_index.j].as_ref().expect("exist");
                    correction.combine(&this_node.exhausted_map[&self.fhi(*b)].next_correction.as_ref().expect("must call `build_exhausted_path`"));
                    next_index = this_node.exhausted_map[&self.fhi(*b)].next.as_ref().expect("exist");
                    loop_counter -= 1;
                }
                if loop_counter == 0 {
                    panic!("potential infinite loop detected in get_correction_two_nodes, check exhausted path building")
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
    pub fn generate_detected_erasures(&self) -> DetectedErasures {
        let mut detected_erasures = DetectedErasures::new(self.di, self.dj);
        self.iterate_snapshot(|t, i, j, node| {
            if node.has_erasure {
                detected_erasures.erasures.push(Index::new(t, i, j));
                for pauli_error_connection in node.pauli_error_connections.iter() {
                    match pauli_error_connection {
                        Either::Left((index1, index2)) => {
                            detected_erasures.connected_insert(index1, index2);
                        },
                        Either::Right(idx) => {
                            detected_erasures.boundaries.insert(self.fhi(*idx));
                        },
                    }
                }
            }
        });
        detected_erasures
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
        let mut time_tailored_prepare_graph = 0.;
        let mut time_tailored_blossom_v_union = 0.;
        let mut time_prepare_graph = 0.;
        let mut time_blossom_v = 0.;
        let mut time_constructing_correction = 0.;
        if to_be_matched.len() != 0 {
            // then add the edges to the graph
            let m_len = to_be_matched.len();  // boundary connection to `i` is `i + m_len`
            let node_num = m_len * 2;
            // prepare union-find structures
            let mut tailored_union = UnionFind::new(if self.enabled_tailored_decoding { m_len * 3 } else { 0 } );
            // note: tailored_union needs to keep track of boundary connections as well, so the length is `m_len * 3`
            //     this is because it needs to distinguish the positive boundary and negative boundary
            if self.enabled_tailored_decoding {
                macro_rules! run_tailored_sub_matching_and_union {
                    // `()` indicates that the macro takes no argument.
                    ($tailored_exhausted_map:ident, $tailored_exhausted_boundary:ident, $is_negative:expr) => {
                        let begin = Instant::now();
                        // Y (X) stabilizers are fully connected, boundaries are fully connected
                        // stabilizer to boundary is one-to-one connected
                        let mut tailored_weighted_edges = Vec::<(usize, usize, f64)>::new();
                        for i in 0..m_len {
                            for j in (i+1)..m_len {
                                tailored_weighted_edges.push((i + m_len, j + m_len, 0.));
                                let a = &to_be_matched[i];
                                let b = &to_be_matched[j];
                                let tailored_path = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").$tailored_exhausted_map.get(&self.fhi(*b));
                                if tailored_path.is_some() && !tailored_path.expect("exist").removed {
                                    let cost = tailored_path.expect("exist").cost;
                                    tailored_weighted_edges.push((i, j, cost));
                                    // println!("[debug] {} can match with {} with cost {}", i, j, cost);
                                }
                            }
                            let a = &to_be_matched[i];
                            match self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").$tailored_exhausted_boundary.as_ref() {
                                Some(tailored_exhausted_boundary) => {
                                    let cost = tailored_exhausted_boundary.cost;
                                    tailored_weighted_edges.push((i, i + m_len, cost));
                                    // println!("[debug] {} can match to {} boundary with cost {}", i, if $is_negative { "negative" } else { "positive" }, cost);
                                },
                                None => { }
                            }
                        }
                        time_tailored_prepare_graph += begin.elapsed().as_secs_f64();
                        let begin = Instant::now();
                        let tailored_matching = blossom_v::safe_minimum_weight_perfect_matching(node_num, tailored_weighted_edges);
                        // println!("matchings from {}", stringify!($tailored_exhausted_map));
                        for i in 0..m_len {
                            let j = tailored_matching[i];
                            if j < m_len {  // only non-boundary connections will be added
                                tailored_union.union(i, j);
                                println!("    union {} {}", i, j);
                            } else {  // boundary connection needs special handling in tailored decoding, because we need to remember "where" is this boundary: left or right
                                // without this special handling, 4 Z errors in a code distance 5 will cause logical error (tailored_code_test_decode_phenomenological, debug 2.5), which is unacceptable
                                let j_bias = if $is_negative { m_len } else { 0 };  // negative boundary will be biased
                                tailored_union.union(i, j + j_bias);
                                println!("    union {} {} boundary", i, if $is_negative { "negative" } else { "positive" });
                            }
                        }
                        time_tailored_blossom_v_union += begin.elapsed().as_secs_f64();
                    }
                }
                run_tailored_sub_matching_and_union!(tailored_positive_exhausted_map, tailored_positive_exhausted_boundary, false);
                run_tailored_sub_matching_and_union!(tailored_negative_exhausted_map, tailored_negative_exhausted_boundary, true);
                // then update the cardinality of each cluster to see if it is charged
                // for i in 0..m_len {
                //     if tailored_union.immutable_get(i).cardinality == 0 {  // once run, this will update cardinality to nonzero
                //         // first check this cluster indeed contains even number of vertices
                //         let mut vertices_count = 1;  // itself
                //         if tailored_union.find(i) == tailored_union.find(i + m_len) { vertices_count += 1; }
                //         if tailored_union.find(i) == tailored_union.find(i + m_len * 2) { vertices_count += 1; }
                //         for j in (i+1)..m_len {
                //             if tailored_union.find(i) == tailored_union.find(j) { vertices_count += 1; }
                //             if tailored_union.find(i) == tailored_union.find(j + m_len) { vertices_count += 1; }
                //             if tailored_union.find(i) == tailored_union.find(j + m_len * 2) { vertices_count += 1; }
                //         }
                //         println!("vertices_count = {}", vertices_count);
                //         assert_eq!(vertices_count % 2, 0, "cluster must have even number of vertices, if you see this, the algorithm has bug");
                //         tailored_union.get_mut(i).cardinality = vertices_count;  // TODO: update this
                //     }
                // }
            }
            let begin = Instant::now();
            // Z (X) stabilizers are fully connected, boundaries are fully connected
            // stabilizer to boundary is one-to-one connected
            let mut weighted_edges = Vec::<(usize, usize, f64)>::new();
            for i in 0..m_len {
                for j in (i+1)..m_len {
                    weighted_edges.push((i + m_len, j + m_len, 0.));  // update 2022.3.15 Yue: virtual boundaries are always fully connected, no matter whether exhaust path exists
                    let a = &to_be_matched[i];
                    let b = &to_be_matched[j];
                    let path = self.snapshot[a.t][a.i][a.j].as_ref().expect("exist").exhausted_map.get(&self.fhi(*b));
                    if path.is_some() && !path.expect("exist").removed {
                        let cost = path.expect("exist").cost;
                        // (tailored_union.immutable_get(i).cardinality % 2 == 1 && tailored_union.immutable_get(j).cardinality % 2 == 1)
                        if !self.enabled_tailored_decoding || tailored_union.find(i) == tailored_union.find(j) {
                            weighted_edges.push((i, j, cost));
                        }
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
            time_prepare_graph += begin.elapsed().as_secs_f64();
            // if to_be_matched.len() > 2 {
            //     println!{"node num {:?}, weighted edges {:?}", node_num, weighted_edges};
            // }
            let begin = Instant::now();
            let matching = blossom_v::safe_minimum_weight_perfect_matching(node_num, weighted_edges);
            time_blossom_v += begin.elapsed().as_secs_f64();
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
            time_constructing_correction += begin.elapsed().as_secs_f64();
            // if to_be_matched.len() > 2 {
            //     println!("correction: {:?}", correction);
            // }
        }
        (correction, json!({
            "to_be_matched": to_be_matched.len(),
            "time_tailored_prepare_graph": time_tailored_prepare_graph,
            "time_tailored_blossom_v_union": time_tailored_blossom_v_union,
            "time_prepare_graph": time_prepare_graph,
            "time_blossom_v": time_blossom_v,
            "time_constructing_correction": time_constructing_correction,
        }), edge_matchings, boundary_matchings)
    }
    
    /// decode based on UnionFind decoder
    pub fn decode_UnionFind(&self, measurement: &Measurement, detected_erasures: &DetectedErasures, max_half_weight: usize, use_distributed: bool
            , detailed_runtime_statistics: bool) -> (Correction, serde_json::Value) {
        let (sparse_correction, runtime_statistics) = self.decode_UnionFind_sparse_correction(measurement, detected_erasures, max_half_weight
            , use_distributed, detailed_runtime_statistics);
        (Correction::from(&sparse_correction), runtime_statistics)
    }
    pub fn decode_UnionFind_sparse_correction(&self, measurement: &Measurement, detected_erasures: &DetectedErasures, max_half_weight: usize
            , use_distributed: bool, detailed_runtime_statistics: bool) -> (SparseCorrection, serde_json::Value) {
        let (sparse_correction, runtime_statistics, _, _) = self.decode_UnionFind_sparse_correction_with_edge_matchings(measurement, detected_erasures
            , max_half_weight, use_distributed, detailed_runtime_statistics);
        (sparse_correction, runtime_statistics)
    }
    pub fn decode_UnionFind_sparse_correction_with_edge_matchings(&self, measurement: &Measurement, detected_erasures: &DetectedErasures
            , max_half_weight: usize, use_distributed: bool, detailed_runtime_statistics: bool) ->
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
            , measurement, detected_erasures, max_half_weight, use_distributed, detailed_runtime_statistics);
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
            CodeType::RotatedPlanarCode => {
                assert_eq!(self.di, self.dj, "rotated CSS code doesn't support rectangle lattice right now");
                let middle_point = self.di - 1;
                // Z stabilizer boundary, e.g. d=3: left boundary: [(2,0), (3,1), (4,2)]
                for delta in 0..self.di {
                    if corrected.x[[self.MeasurementRounds, middle_point + delta, delta]] {
                        x_error_count += 1;
                    }
                }
                // X stabilizer boundary, e.g. d=3: left boundary: [(0,2), (1,1), (2,0)]
                for delta in 0..self.di {
                    if corrected.z[[self.MeasurementRounds, delta, middle_point - delta]] {
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
            CodeType::StandardTailoredCode => {
                // Y stabilizer boundary, j = 0
                for i in 0..self.di {
                    // single X or single Z anti-commute with Y stabilizer, otherwise commute
                    if corrected.x[[self.MeasurementRounds, (i*2), 0]] ^ corrected.z[[self.MeasurementRounds, (i*2), 0]] {
                        x_error_count += 1;
                    }
                }
                // X stabilizer boundary, i = 0
                for j in 0..self.dj {
                    if corrected.z[[self.MeasurementRounds, 0, (j*2)]] {
                        z_error_count += 1;  // logical Y error in tailored surface code
                    }
                }
            },
            CodeType::RotatedTailoredCode => {
                assert_eq!(self.di, self.dj, "rotated tailored code doesn't support rectangle lattice right now");
                let middle_point = self.di - 1;
                // Y stabilizer boundary, e.g. d=3: left boundary: [(2,0), (3,1), (4,2)]
                for delta in 0..self.di {
                    // single X or single Z anti-commute with Y stabilizer, otherwise commute
                    if corrected.x[[self.MeasurementRounds, middle_point + delta, delta]] ^ corrected.z[[self.MeasurementRounds, middle_point + delta, delta]] {
                        x_error_count += 1;
                    }
                }
                // X stabilizer boundary, e.g. d=3: left boundary: [(0,2), (1,1), (2,0)]
                for delta in 0..self.di {
                    if corrected.z[[self.MeasurementRounds, delta, middle_point - delta]] {
                        z_error_count += 1;  // logical Y error in tailored surface code
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

    /// check as strictly as possible!
    pub fn apply_error_model_modifier(&mut self, modifier: &serde_json::Value) -> Result<(), String> {
        if modifier.get("code_type").ok_or(format!("missing field: code_type"))? != &json!(self.code_type) {
            return Err(format!("mismatch: code_type"))
        }
        if modifier.get("di").ok_or(format!("missing field: di"))? != &json!(self.di) {
            return Err(format!("mismatch: di"))
        }
        if modifier.get("dj").ok_or(format!("missing field: dj"))? != &json!(self.dj) {
            return Err(format!("mismatch: dj"))
        }
        if modifier.get("MeasurementRounds").ok_or(format!("missing field: MeasurementRounds"))? != &json!(self.MeasurementRounds) {
            return Err(format!("mismatch: MeasurementRounds"))
        }
        if modifier.get("T").ok_or(format!("missing field: T"))? != &json!(self.T) {
            return Err(format!("mismatch: T"))
        }
        // snapshot
        let snapshot = modifier.get("snapshot").ok_or(format!("missing field: snapshot"))?.as_array().ok_or(format!("format error: snapshot"))?;
        if self.snapshot.len() != snapshot.len() {
            return Err(format!("mismatch: snapshot"))
        }
        for t in 0..snapshot.len() {
            let snapshot_row_0 = snapshot[t].as_array().ok_or(format!("format error: snapshot[{}]", t))?;
            if snapshot_row_0.len() != self.snapshot[t].len() {
                return Err(format!("mismatch: snapshot[{}]", t))
            }
            for i in 0..snapshot_row_0.len() {
                let snapshot_row_1 = snapshot_row_0[i].as_array().ok_or(format!("format error: snapshot[{}][{}]", t, i))?;
                if snapshot_row_1.len() != self.snapshot[t][i].len() {
                    return Err(format!("mismatch: snapshot[{}][{}]", t, i))
                }
                for j in 0..snapshot_row_1.len() {
                    let node = &snapshot_row_1[j];
                    if node.is_null() != self.snapshot[t][i][j].is_none() {
                        return Err(format!("mismatch: snapshot[{}][{}][{}]", t, i, j))
                    }
                    if !node.is_null() {
                        let self_node = self.snapshot[t][i][j].as_mut().unwrap();  // already checked existance
                        if node.get("t").ok_or(format!("missing field: t"))? != &json!(self_node.t) {
                            return Err(format!("mismatch [{}][{}][{}]: t", t, i, j))
                        }
                        if node.get("i").ok_or(format!("missing field: i"))? != &json!(self_node.i) {
                            return Err(format!("mismatch [{}][{}][{}]: i", t, i, j))
                        }
                        if node.get("j").ok_or(format!("missing field: j"))? != &json!(self_node.j) {
                            return Err(format!("mismatch [{}][{}][{}]: j", t, i, j))
                        }
                        if node.get("connection").ok_or(format!("missing field: connection"))? != &json!(self_node.connection) {
                            return Err(format!("mismatch [{}][{}][{}]: connection", t, i, j))
                        }
                        if node.get("gate_type").ok_or(format!("missing field: gate_type"))? != &json!(self_node.gate_type) {
                            return Err(format!("mismatch [{}][{}][{}]: gate_type", t, i, j))
                        }
                        if node.get("qubit_type").ok_or(format!("missing field: qubit_type"))? != &json!(self_node.qubit_type) {
                            return Err(format!("mismatch [{}][{}][{}]: qubit_type", t, i, j))
                        }
                        // then copy error rate data
                        self_node.error_rate_x = node.get("error_rate_x").ok_or(format!("missing field: error_rate_x"))?.as_f64().ok_or(format!("format error: error_rate_x"))?;
                        self_node.error_rate_z = node.get("error_rate_z").ok_or(format!("missing field: error_rate_z"))?.as_f64().ok_or(format!("format error: error_rate_z"))?;
                        self_node.error_rate_y = node.get("error_rate_y").ok_or(format!("missing field: error_rate_y"))?.as_f64().ok_or(format!("format error: error_rate_y"))?;
                        self_node.erasure_error_rate = node.get("erasure_error_rate").ok_or(format!("missing field: erasure_error_rate"))?.as_f64().ok_or(format!("format error: erasure_error_rate"))?;
                        let correlated_error_model = node.get("correlated_error_model").ok_or(format!("missing field: correlated_error_model"))?;
                        if !correlated_error_model.is_null() {
                            self_node.correlated_error_model = Some(serde_json::from_value(correlated_error_model.clone()).map_err(|_| format!("correlated_error_model deserialize error"))?);
                        } else {
                            self_node.correlated_error_model = None;
                        }
                        let correlated_erasure_error_model = node.get("correlated_erasure_error_model").ok_or(format!("missing field: correlated_erasure_error_model"))?;
                        if !correlated_erasure_error_model.is_null() {
                            self_node.correlated_erasure_error_model = Some(serde_json::from_value(correlated_erasure_error_model.clone()).map_err(|_| format!("correlated_erasure_error_model deserialize error"))?);
                        } else {
                            self_node.correlated_erasure_error_model = None;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn apply_error_model(&mut self, error_model: &ErrorModelName, error_model_configuration: Option<&serde_json::Value>, p: f64, bias_eta: f64, pe: f64) {
        let mut error_model_configuration_recognized = false;
        match error_model {
            ErrorModelName::GenericBiasedWithBiasedCX | ErrorModelName::GenericBiasedWithStandardCX => {
                let height = self.snapshot.len();
                let mut initialization_error_rate = p;  // by default initialization error rate is the same as p
                let mut measurement_error_rate = p;
                error_model_configuration_recognized = true;
                error_model_configuration.map(|config| {
                    let mut config_cloned = config.clone();
                    let config = config_cloned.as_object_mut().expect("error_model_configuration must be JSON object");
                    config.remove("initialization_error_rate").map(|value| initialization_error_rate = value.as_f64().expect("f64"));
                    config.remove("measurement_error_rate").map(|value| measurement_error_rate = value.as_f64().expect("f64"));
                    if !config.is_empty() { panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>()); }
                });
                self.iterate_snapshot_mut(|t, _i, _j, node| {
                    // first clear error rate
                    node.error_rate_x = 0.;
                    node.error_rate_z = 0.;
                    node.error_rate_y = 0.;
                    node.erasure_error_rate = 0.;
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
                            node.error_rate_x = initialization_error_rate / bias_eta;
                            node.error_rate_z = initialization_error_rate;
                            node.error_rate_y = initialization_error_rate / bias_eta;
                        },
                        Stage::CXGate1 | Stage::CXGate2 | Stage::CXGate3 | Stage::CXGate4 => {
                            if stage == Stage::CXGate4 && node.qubit_type != QubitType::Data {  // add measurement errors (p + p/bias_eta)
                                node.error_rate_x = measurement_error_rate / bias_eta;
                                node.error_rate_z = measurement_error_rate;
                                node.error_rate_y = measurement_error_rate / bias_eta;
                            }
                            match node.gate_type {
                                GateType::ControlledPhase => {
                                    if node.qubit_type != QubitType::Data {  // this is ancilla
                                        // better check whether peer is indeed data qubit, but it's hard here due to Rust's borrow check
                                        let mut correlated_error_model = CorrelatedPauliErrorRates::default_with_probability(p / bias_eta);
                                        correlated_error_model.error_rate_ZI = p;
                                        correlated_error_model.error_rate_IZ = p;
                                        correlated_error_model.sanity_check();
                                        node.correlated_error_model = Some(correlated_error_model);
                                    }
                                },
                                GateType::Control => {  // this is ancilla in XZZX code, see arXiv:2104.09539v1
                                    let mut correlated_error_model = CorrelatedPauliErrorRates::default_with_probability(p / bias_eta);
                                    correlated_error_model.error_rate_ZI = p;
                                    match error_model {
                                        ErrorModelName::GenericBiasedWithStandardCX => {
                                            correlated_error_model.error_rate_IZ = 0.375 * p;
                                            correlated_error_model.error_rate_ZZ = 0.375 * p;
                                            correlated_error_model.error_rate_IY = 0.125 * p;
                                            correlated_error_model.error_rate_ZY = 0.125 * p;
                                        },
                                        ErrorModelName::GenericBiasedWithBiasedCX => {
                                            correlated_error_model.error_rate_IZ = 0.5 * p;
                                            correlated_error_model.error_rate_ZZ = 0.5 * p;
                                        },
                                        _ => { }
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
            ErrorModelName::ErasureOnlyPhenomenological => {
                assert_eq!(p, 0., "pauli error should be 0 in this error model");
                let height = self.snapshot.len();
                self.iterate_snapshot_mut(|t, _i, _j, node| {
                    // first clear error rate
                    node.error_rate_x = 0.;
                    node.error_rate_z = 0.;
                    node.error_rate_y = 0.;
                    node.erasure_error_rate = 0.;
                    if t >= height - 6 {  // no error on the top, as a perfect measurement round
                        return
                    } else if t <= 6 {
                        return  // perfect initialization
                    }
                    // do different things for each stage
                    let stage = Stage::from(t);
                    match stage {
                        Stage::CXGate4 => {
                            // qubit is before the next measurement round's gates, measurement is after current measurement round's gates
                            node.erasure_error_rate = pe;
                        },
                        _ => { }
                    }
                });
            },
            ErrorModelName::PauliZandErasurePhenomenological => {  // this error model is from https://arxiv.org/pdf/1709.06218v3.pdf
                let height = self.snapshot.len();
                error_model_configuration_recognized = true;
                let mut also_include_pauli_x = false;
                error_model_configuration.map(|config| {
                    let mut config_cloned = config.clone();
                    let config = config_cloned.as_object_mut().expect("error_model_configuration must be JSON object");
                    config.remove("also_include_pauli_x").map(|value| also_include_pauli_x = value.as_bool().expect("bool"));
                    if !config.is_empty() { panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>()); }
                });
                self.iterate_snapshot_mut(|t, _i, _j, node| {
                    // first clear error rate
                    node.error_rate_x = 0.;
                    node.error_rate_z = 0.;
                    node.error_rate_y = 0.;
                    node.erasure_error_rate = 0.;
                    if t >= height - 6 {  // no error on the top, as a perfect measurement round
                        return
                    } else if t <= 6 {
                        return  // perfect initialization
                    }
                    // do different things for each stage
                    let stage = Stage::from(t);
                    match stage {
                        Stage::CXGate4 => {
                            // qubit is before the next measurement round's gates, measurement is after current measurement round's gates
                            node.erasure_error_rate = pe;
                            if node.qubit_type == QubitType::Data {
                                node.error_rate_z = p;
                                if also_include_pauli_x {
                                    node.error_rate_x = p;
                                }
                            } else { // ancilla, to make sure it always cause only 1 measurement error if it happens
                                node.error_rate_z = p;
                                node.error_rate_x = p;
                            }
                        },
                        _ => { }
                    }
                });
            },
            ErrorModelName::OnlyGateErrorCircuitLevel | ErrorModelName::OnlyGateErrorCircuitLevelCorrelatedErasure => {
                let is_correlated_erasure = error_model == &ErrorModelName::OnlyGateErrorCircuitLevelCorrelatedErasure;
                let height = self.snapshot.len();
                let mut initialization_error_rate = 0.;
                let mut measurement_error_rate = 0.;
                let mut use_correlated_pauli = false;
                let mut before_pauli_bug_fix = false;
                error_model_configuration_recognized = true;
                error_model_configuration.map(|config| {
                    let mut config_cloned = config.clone();
                    let config = config_cloned.as_object_mut().expect("error_model_configuration must be JSON object");
                    config.remove("initialization_error_rate").map(|value| initialization_error_rate = value.as_f64().expect("f64"));
                    config.remove("measurement_error_rate").map(|value| measurement_error_rate = value.as_f64().expect("f64"));
                    config.remove("use_correlated_pauli").map(|value| use_correlated_pauli = value.as_bool().expect("bool"));
                    config.remove("before_pauli_bug_fix").map(|value| before_pauli_bug_fix = value.as_bool().expect("bool"));
                    if !config.is_empty() { panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>()); }
                });
                self.iterate_snapshot_mut(|t, _i, _j, node| {
                    // first clear error rate
                    node.error_rate_x = 0.;
                    node.error_rate_z = 0.;
                    node.error_rate_y = 0.;
                    node.erasure_error_rate = 0.;
                    if t >= height - 6 {  // no error on the top, as a perfect measurement round
                        return
                    } else if t <= 6 {
                        return  // perfect initialization
                    }
                    // do different things for each stage
                    let stage = Stage::from(t);
                    match stage {
                        Stage::Initialization => {
                            if node.qubit_type != QubitType::Data {
                                node.error_rate_x = initialization_error_rate / 3.;
                                node.error_rate_z = initialization_error_rate / 3.;
                                node.error_rate_y = initialization_error_rate / 3.;
                            }
                        },
                        Stage::CXGate1 | Stage::CXGate2 | Stage::CXGate3 | Stage::CXGate4 => {
                            // errors everywhere
                            let mut this_position_use_correlated_pauli = false;
                            if is_correlated_erasure {
                                match node.gate_type {
                                    GateType::ControlledPhase => {
                                        if node.qubit_type != QubitType::Data {  // this is ancilla
                                            // better check whether peer is indeed data qubit, but it's hard here due to Rust's borrow check
                                            let mut correlated_erasure_error_model = CorrelatedErasureErrorRates::default_with_probability(0.);
                                            correlated_erasure_error_model.error_rate_EE = pe;
                                            correlated_erasure_error_model.sanity_check();
                                            node.correlated_erasure_error_model = Some(correlated_erasure_error_model);
                                            this_position_use_correlated_pauli = use_correlated_pauli;
                                        }
                                    },
                                    GateType::Control => {  // this is ancilla
                                        let mut correlated_erasure_error_model = CorrelatedErasureErrorRates::default_with_probability(0.);
                                        correlated_erasure_error_model.error_rate_EE = pe;
                                        correlated_erasure_error_model.sanity_check();
                                        node.correlated_erasure_error_model = Some(correlated_erasure_error_model);
                                        this_position_use_correlated_pauli = use_correlated_pauli;
                                    },
                                    _ => { }
                                }
                            } else {
                                node.erasure_error_rate = pe;
                            }
                            // this bug is hard to find without visualization tool...
                            // so I develop such a tool at https://qec.wuyue98.cn/ErrorModelViewer2D.html
                            // to compare: (in url, %20 is space, %22 is double quote)
                            //     https://qec.wuyue98.cn/ErrorModelViewer2D.html?p=0.01&pe=0.05&parameters=--use_xzzx_code%20--error_model%20OnlyGateErrorCircuitLevelCorrelatedErasure%20--error_model_configuration%20%22{\%22use_correlated_pauli\%22:true}%22
                            //     https://qec.wuyue98.cn/ErrorModelViewer2D.html?p=0.01&pe=0.05&parameters=--use_xzzx_code%20--error_model%20OnlyGateErrorCircuitLevelCorrelatedErasure%20--error_model_configuration%20%22{\%22use_correlated_pauli\%22:true,\%22before_pauli_bug_fix\%22:true}%22
                            let mut px_py_pz = if before_pauli_bug_fix {
                                if this_position_use_correlated_pauli { (0., 0., 0.) } else { (p/3., p/3., p/3.) }
                            } else {
                                if use_correlated_pauli { (0., 0., 0.) } else { (p/3., p/3., p/3.) }
                            };
                            if stage == Stage::CXGate4 && node.qubit_type != QubitType::Data {
                                // add additional measurement error
                                // whether it's X axis measurement or Z axis measurement, the additional error rate is always `measurement_error_rate`
                                px_py_pz = ErrorType::combine_probability(px_py_pz, (measurement_error_rate / 2., measurement_error_rate / 2., measurement_error_rate / 2.));
                            }
                            let (px, py, pz) = px_py_pz;
                            node.error_rate_x = px;
                            node.error_rate_y = py;
                            node.error_rate_z = pz;
                            if this_position_use_correlated_pauli {
                                let correlated_error_model = CorrelatedPauliErrorRates::default_with_probability(p / 15.);  // 15 possible errors equally probable
                                correlated_error_model.sanity_check();
                                node.correlated_error_model = Some(correlated_error_model);
                            }
                        },
                        _ => { }
                    }
                });
            },
            ErrorModelName::Arxiv200404693 => {
                let height = self.snapshot.len();
                let mut use_nature_initialization_error = false;  // by default use the one defined in the paper (although I believe it's the same with p/3 X,Y,Z pauli errors)
                let mut use_nature_measurement_error = false;  // I believe they're the same
                error_model_configuration_recognized = true;
                error_model_configuration.map(|config| {
                    let mut config_cloned = config.clone();
                    let config = config_cloned.as_object_mut().expect("error_model_configuration must be JSON object");
                    config.remove("use_nature_initialization_error").map(|value| use_nature_initialization_error = value.as_bool().expect("bool"));
                    config.remove("use_nature_measurement_error").map(|value| use_nature_measurement_error = value.as_bool().expect("bool"));
                    if !config.is_empty() { panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>()); }
                });
                self.iterate_snapshot_mut(|t, _i, _j, node| {
                    // first clear error rate
                    node.error_rate_x = 0.;
                    node.error_rate_z = 0.;
                    node.error_rate_y = 0.;
                    node.erasure_error_rate = 0.;
                    if t >= height - 6 {  // no error on the top, as a perfect measurement round
                        return
                    } else if t <= 6 {
                        return  // perfect initialization
                    }
                    // do different things for each stage
                    let stage = Stage::from(t);
                    match stage {
                        Stage::Initialization => {
                            if node.qubit_type == QubitType::Data {
                                node.error_rate_x = p / 3.;
                                node.error_rate_z = p / 3.;
                                node.error_rate_y = p / 3.;
                            } else {
                                if use_nature_initialization_error {
                                    node.error_rate_x = p / 3.;
                                    node.error_rate_z = p / 3.;
                                    node.error_rate_y = p / 3.;
                                } else {
                                    // |0> has 2p/3 probability to be |1>, |+> has 2p/3 probability to be |->
                                    if node.qubit_type.is_measured_in_z_basis().unwrap() {
                                        node.error_rate_x = 2. * p / 3.;  // |0> apply X error is |1>
                                    } else {
                                        node.error_rate_z = 2. * p / 3.;  // |+> apply Z error is |->
                                    }
                                }
                            }
                        },
                        Stage::CXGate1 | Stage::CXGate2 | Stage::CXGate3 | Stage::CXGate4 => {
                            if stage == Stage::CXGate4 && node.qubit_type != QubitType::Data {
                                // add additional measurement error
                                // paper requires that whether measure in X or Z basis, the error probability should both be 2p/3
                                if use_nature_measurement_error {
                                    node.error_rate_x = p / 3.;
                                    node.error_rate_z = p / 3.;
                                    node.error_rate_y = p / 3.;
                                } else {
                                    if node.qubit_type.is_measured_in_z_basis().unwrap() {
                                        node.error_rate_x = 2. * p / 3.;  // sensitive to X errors
                                    } else {
                                        node.error_rate_z = 2. * p / 3.;  // sensitive to Z errors
                                    }
                                }
                            }
                            match node.gate_type {
                                GateType::ControlledPhase => {
                                    if node.qubit_type != QubitType::Data {  // this is ancilla
                                        // better check whether peer is indeed data qubit, but it's hard here due to Rust's borrow check
                                        let correlated_error_model = CorrelatedPauliErrorRates::default_with_probability(p / 15.);
                                        correlated_error_model.sanity_check();
                                        node.correlated_error_model = Some(correlated_error_model);
                                    }
                                },
                                GateType::Control => {  // this is ancilla in XZZX code, see arXiv:2104.09539v1
                                    let correlated_error_model = CorrelatedPauliErrorRates::default_with_probability(p / 15.);
                                    correlated_error_model.sanity_check();
                                    node.correlated_error_model = Some(correlated_error_model);
                                },
                                _ => { }
                            }
                        },
                        Stage::Measurement => {
                            // idle gate on data qubits
                            if node.qubit_type == QubitType::Data {
                                node.error_rate_x = p / 3.;
                                node.error_rate_z = p / 3.;
                                node.error_rate_y = p / 3.;
                            }
                        },
                    }
                });
            },
            ErrorModelName::TailoredPhenomenological => {
                self.enabled_tailored_decoding = true;  // mark it to use tailored decoding
                let height = self.snapshot.len();
                let px = p / (1. + bias_eta) / 2.;
                let py = px;
                let pz = p - 2. * px;
                let mut measurement_error_rate = p;  // by default, can be adjusted using configuration
                error_model_configuration_recognized = true;
                error_model_configuration.map(|config| {
                    let mut config_cloned = config.clone();
                    let config = config_cloned.as_object_mut().expect("error_model_configuration must be JSON object");
                    config.remove("measurement_error_rate").map(|value| measurement_error_rate = value.as_f64().expect("f64"));
                    if !config.is_empty() { panic!("unknown keys: {:?}", config.keys().collect::<Vec<&String>>()); }
                });
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
                        Stage::CXGate4 => {
                            // qubit is before the next measurement round's gates, measurement is after current measurement round's gates
                            node.erasure_error_rate = pe;
                            if node.qubit_type == QubitType::Data {
                                node.error_rate_x = px;
                                node.error_rate_z = pz;
                                node.error_rate_y = py;
                            } else { // ancilla, since they all measure in X basis, a Z error is enough to flip the measurement result
                                node.error_rate_z = measurement_error_rate;
                            }
                        },
                        _ => { }
                    }
                });
            },
        }
        assert_eq!(error_model_configuration_recognized || error_model_configuration.is_none(), true
            , "error model configuration must be recognized if exists");
    }

    pub fn print_direct_connections(&self) {
        self.iterate_snapshot(|t, i, j, node| {
            if Stage::from(t) == Stage::Measurement && node.qubit_type != QubitType::Data {
                println!("[{}][{}][{}]: {:?}", t, i, j, node.qubit_type);
                match &node.boundary {
                    Some(boundary) => println!("boundary: p = {}", boundary.p),
                    None => println!("boundary: none"),
                }
                match &node.tailored_positive_boundary {
                    Some(boundary) => println!("tailored positive boundary: p = {}", boundary.p),
                    None => { },
                }
                match &node.tailored_negative_boundary {
                    Some(boundary) => println!("tailored negative boundary: p = {}", boundary.p),
                    None => { },
                }
                for edge in node.edges.iter().filter(|edge| edge.p > 0.) {
                    println!("edge [{}][{}][{}]: p = {}", edge.t, edge.i, edge.j, edge.p);
                }
                for edge in node.tailored_positive_edges.iter().filter(|edge| edge.p > 0.) {
                    println!("tailored positive edge [{}][{}][{}]: p = {}", edge.t, edge.i, edge.j, edge.p);
                }
                for edge in node.tailored_negative_edges.iter().filter(|edge| edge.p > 0.) {
                    println!("tailored negative edge [{}][{}][{}]: p = {}", edge.t, edge.i, edge.j, edge.p);
                }
                println!("");
            }
        });
    }

    pub fn print_exhausted_connections(&self) {
        self.iterate_snapshot(|t, i, j, node| {
            if Stage::from(t) == Stage::Measurement && node.qubit_type != QubitType::Data {
                println!("[{}][{}][{}]: {:?}", t, i, j, node.qubit_type);
                match &node.exhausted_boundary {
                    Some(boundary) => println!("boundary: c = {}", boundary.cost),
                    None => println!("boundary: none"),
                }
                // sort it before print
                let mut exhausted_map_vec = Vec::new();
                for (idx, edge) in node.exhausted_map.iter() {
                    exhausted_map_vec.push((idx, edge));
                }
                exhausted_map_vec.sort_by(|a, b| a.0.partial_cmp(b.0).unwrap());
                for (idx, edge) in exhausted_map_vec.iter() {
                    println!("edge [{}][{}][{}]: c = {}", idx.index.t, idx.index.i, idx.index.j, edge.cost);
                }
            }
        });
    }
}

/// Stage is determined by time t
#[derive(Debug, PartialEq, Serialize)]
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
#[derive(Debug, PartialEq, Clone, Serialize)]
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
    // Controlled Y gate, used in tailored surface code
    ControlCY,
    TargetCY,
    None,  // do nothing
}

/// Connection Information, corresponds to `connection` in `FaultTolerantView.vue`
#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub t: usize,
    pub i: usize,
    pub j: usize,
}

/// Edge Information, corresponds to `node.edges` in `FaultTolerantView.vue`
#[derive(Debug, Clone, Serialize)]
pub struct Edge {
    pub t: usize,
    pub i: usize,
    pub j: usize,
    pub p: f64,
    pub cases: Vec::<(Arc<SparseCorrection>, f64)>,
    // calculated
    pub weight: f64,
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

pub fn add_edge_case<F>(edges: &mut Vec::<Edge>, t: usize, i: usize, j: usize, p: f64, correction: Arc<SparseCorrection>, use_combined_probability: bool, weight_of: F) where F: Fn(f64) -> f64 {
    for edge in edges.iter_mut() {
        if edge.t == t && edge.i == i && edge.j == j {
            edge.add(p, correction, use_combined_probability, weight_of);
            return  // already found
        }
    }
    let mut edge = Edge {
        t: t, i: i, j: j, p: 0.,
        weight: f64::MAX,
        cases: Vec::new(),
    };
    edge.add(p, correction, use_combined_probability, weight_of);
    edges.push(edge);
}

impl Edge {
    pub fn add<F>(&mut self, p: f64, correction: Arc<SparseCorrection>, use_combined_probability: bool, weight_of: F) where F: Fn(f64) -> f64 {
        if use_combined_probability {
            self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        } else {
            self.p = self.p.max(p);  // max
        }
        self.weight = weight_of(self.p);
        self.cases.push((correction, p));
    }
}

/// Boundary Information, corresponds to `node.boundary` in `FaultTolerantView.vue`
#[derive(Debug, Clone, Serialize)]
pub struct Boundary {
    pub p: f64,
    pub cases: Vec::<(Arc<SparseCorrection>, f64)>,
    // calculated
    pub weight: f64,
}

impl Boundary {
    pub fn add<F>(&mut self, p: f64, correction: Arc<SparseCorrection>, use_combined_probability: bool, weight_of: F) where F: Fn(f64) -> f64 {
        if use_combined_probability {
            self.p = self.p * (1. - p) + p * (1. - self.p);  // XOR
        } else {
            self.p = self.p.max(p);  // max
        }
        self.weight = weight_of(self.p);
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedErasures {
    /// used for converting hash
    pub di: usize,
    pub dj: usize,
    /// the position of the erasure error
    pub erasures: Vec<Index>,
    /// two-stabilizer connected with 0 weighted caused by the erasure, used by UF decoder or (indirectly) by MWPM decoder
    pub connected: HashSet<(FastHashIndex, FastHashIndex)>,
    /// stabilizer connected to boundary with 0 weight caused by the erasure, used by UF decoder
    pub boundaries: HashSet<FastHashIndex, std::hash::BuildHasherDefault::<SimpleHasher>>,
    /// connected edges for each node, used by UF decoder
    pub connected_edges: HashMap<FastHashIndex, HashSet<FastHashIndex, std::hash::BuildHasherDefault::<SimpleHasher> >, std::hash::BuildHasherDefault::<SimpleHasher> >,
}

impl DetectedErasures {
    pub fn new(di: usize, dj: usize) -> Self {
        Self {
            di: di,
            dj: dj,
            erasures: Vec::new(),
            connected: HashSet::default(),
            boundaries: HashSet::default(),
            connected_edges: HashMap::default(),
        }
    }
    #[inline(always)]
    pub fn fhi(&self, index: Index) -> FastHashIndex {
        FastHashIndex::with_di_dj(&index, self.di, self.dj)
    }
    pub fn has_erasures(&self) -> bool {
        !self.erasures.is_empty()
    }
    fn sorted_index<'a>(index1: &'a Index, index2: &'a Index) -> (&'a Index, &'a Index) {
        let idx1 = std::cmp::min(index1, index2);
        let idx2 = std::cmp::max(index1, index2);
        (idx1, idx2)
    }
    pub fn connected_insert(&mut self, index1: &Index, index2: &Index) -> bool {
        let di = self.di;
        let dj = self.dj;
        let fhi = |index: Index| -> FastHashIndex {
            FastHashIndex::with_di_dj(&index, di, dj)
        };
        assert!(index1 != index2, "one cannot connect to itself");
        // insert to `connected_edges` for ease of querying
        if !self.connected_edges.contains_key(&self.fhi(*index1)) { self.connected_edges.insert(self.fhi(*index1), HashSet::default()); }
        self.connected_edges.get_mut(&self.fhi(*index1)).unwrap().insert(fhi(*index2));
        if !self.connected_edges.contains_key(&self.fhi(*index2)) { self.connected_edges.insert(self.fhi(*index2), HashSet::default()); }
        self.connected_edges.get_mut(&self.fhi(*index2)).unwrap().insert(fhi(*index1));
        // also insert to `connected`
        let (idx1, idx2) = Self::sorted_index(index1, index2);
        self.connected.insert((self.fhi(*idx1), self.fhi(*idx2)))
    }
    pub fn connected_contains(&self, index1: &Index, index2: &Index) -> bool {
        let (idx1, idx2) = Self::sorted_index(index1, index2);
        self.connected.contains(&(self.fhi(*idx1), self.fhi(*idx2)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PetGraphEdge {
    pub a: FastHashIndex,
    pub b: FastHashIndex,
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
    /// if `removed`, not included in matching graph
    pub removed: bool,
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
    use super::super::rand::prelude::*;

    // use `cargo test xzzx_code_test_simulation_1 -- --nocapture` to run specific test

    fn assert_error_is(model: &mut PlanarCodeModel, errors: Vec<(usize, usize, usize)>) {
        model.propagate_error();
        let mut measurement_errors = Vec::new();
        model.iterate_measurement_errors(|t, i, j, _node| {
            measurement_errors.push((t, i, j));
        });
        // println!("{:?}", measurement_errors);
        assert_eq!(measurement_errors, errors);
    }

    #[test]
    fn xzzx_code_test_simulation_1() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let mut model = PlanarCodeModel::new_standard_XZZX_code(measurement_rounds, d);
        model.set_phenomenological_error_with_perfect_initialization(p);
        model.build_graph(weight_autotune);
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
        model.build_graph(weight_autotune);
        model.optimize_correction_pattern();
        model.build_exhausted_path();
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

    #[test]
    fn erasure_error_model_test_generate_erasure_errors() {
        let d = 3;
        let pe = 0.1;  // erasure error rate
        let mut rng = thread_rng();
        let mut model = PlanarCodeModel::new_standard_XZZX_code(0, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(0., 0., 0., 0.);
        model.iterate_snapshot_mut(|t, _i, _j, node| {  // shallow error on bottom
            if t == 6 && node.qubit_type == QubitType::Data {
                node.erasure_error_rate = pe;
            }
        });
        model.build_graph(weight_autotune);
        let error_count = model.generate_random_errors(|| rng.gen::<f64>());
        println!("error_count: {}", error_count);
        let detected_erasures = model.generate_detected_erasures();
        println!("{:?}", detected_erasures);
        println!("\n\nget_all_qubit_errors_vec:\n{:?}", model.get_all_qubit_errors_vec());
    }

    #[test]
    fn tailored_code_test_simulation_1() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let bias_eta = 100.;
        let mut model = PlanarCodeModel::new_rotated_tailored_code(measurement_rounds, d);
        model.apply_error_model(&ErrorModelName::TailoredPhenomenological, None, p, bias_eta, 0.);
        model.build_graph(weight_autotune);
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        // single X error on the top corner
        model.clear_error();
        model.add_error_at(el2t(0), 0, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 0, 1)]);
        // single Z error on the top corner
        model.clear_error();
        model.add_error_at(el2t(0), 0, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 0, 1), (24, 1, 2)]);
        // single Y error on the top corner
        model.clear_error();
        model.add_error_at(el2t(0), 0, 2, &ErrorType::Y).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2)]);
        // single X error in the middle
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 2, 1), (24, 2, 3)]);
        // single Z error in the middle
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 2, 1), (24, 2, 3), (24, 3, 2)]);
        // single Y error in the middle
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::Y).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 3, 2)]);
    }

    #[test]
    fn tailored_code_test_simulation_circuit_level() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let bias_eta = 100.;
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        let mut model = PlanarCodeModel::new_rotated_tailored_code(measurement_rounds, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(px, py, pz, 0.);
        model.build_graph(weight_autotune);
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        // single Z error at data qubit
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 2, 1), (24, 2, 3), (24, 3, 2)]);
        model.clear_error();
        model.add_error_at(el2t(0) + 1, 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 2, 1), (24, 2, 3), (24, 3, 2)]);
        model.clear_error();
        model.add_error_at(el2t(0) + 2, 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 2, 1), (24, 2, 3), (24, 3, 2)]);
        model.clear_error();
        model.add_error_at(el2t(0) + 3, 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 2, 1), (24, 2, 3), (30, 3, 2)]);  // this is equivalent to 4 errors + a measurement error, p' = py ^ 2
        model.clear_error();
        model.add_error_at(el2t(0) + 4, 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 2, 1), (30, 2, 3), (30, 3, 2)]);  // this is equivalent to 4 errors + 2 measurement errors, p' = py ^ 3
        model.clear_error();
        model.add_error_at(el2t(0) + 5, 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (30, 2, 1), (30, 2, 3), (30, 3, 2)]);  // this is equivalent to 4 errors + a measurement error, p' = py ^ 2
        // single Z error at X stabilizer ancilla qubit
        model.clear_error();
        model.add_error_at(el2t(0), 1, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(18, 1, 2), (24, 1, 2)]);  // single measurement error, because it just flip the measurement result
        model.clear_error();
        model.add_error_at(el2t(0) + 1, 1, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);  // no error here because it will be reset by initialization
        model.clear_error();
        model.add_error_at(el2t(0) + 2, 1, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);  // no error here because it will be reset by initialization
        model.clear_error();
        model.add_error_at(el2t(0) + 3, 1, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (30, 1, 2)]);  // a measurement error
        model.clear_error();
        model.add_error_at(el2t(0) + 4, 1, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (30, 1, 2)]);  // a measurement error
        model.clear_error();
        model.add_error_at(el2t(0) + 5, 1, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (30, 1, 2)]);  // a measurement error
        // single Z error at Y stabilizer ancilla qubit
        model.clear_error();
        model.add_error_at(el2t(0), 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(18, 2, 1), (24, 2, 1)]);  // single measurement error, because it just flip the measurement result
        model.clear_error();
        model.add_error_at(el2t(0) + 1, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);  // no error here because it will be reset by initialization
        model.clear_error();
        model.add_error_at(el2t(0) + 2, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);  // no error here because it will be reset by initialization
        model.clear_error();
        model.add_error_at(el2t(0) + 3, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 2, 1), (30, 2, 1)]);  // a measurement error
        model.clear_error();
        model.add_error_at(el2t(0) + 4, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 2, 1), (30, 2, 1)]);  // a measurement error
        model.clear_error();
        model.add_error_at(el2t(0) + 5, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 2, 1), (30, 2, 1)]);  // a measurement error
    }

    #[test]
    fn tailored_code_test_simulation_phenomenological() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let bias_eta = 100.;
        let mut model = PlanarCodeModel::new_rotated_tailored_code(measurement_rounds, d);
        model.apply_error_model(&ErrorModelName::TailoredPhenomenological, None, p, bias_eta, 0.);
        model.build_graph(weight_autotune);
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        // single error at data qubit
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::Y).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 3, 2)]);
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 2, 1), (24, 2, 3)]);
        model.clear_error();
        model.add_error_at(el2t(0), 2, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (24, 2, 1), (24, 2, 3), (24, 3, 2)]);
        // single measurement error on X stabilizers
        model.clear_error();
        model.add_error_at(el2t(0), 1, 2, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(18, 1, 2), (24, 1, 2)]);  // single measurement error, because it just flip the measurement result
        // single measurement error on Y stabilizers
        model.clear_error();
        model.add_error_at(el2t(0), 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(18, 2, 1), (24, 2, 1)]);  // single measurement error, because it just flip the measurement result
    }

    #[test]
    fn reordered_css_surface_code_test_simulation_circuit_level() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let mut model = PlanarCodeModel::new_rotated_planar_code(measurement_rounds, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(p/3., p/3., p/3., 0.);
        model.build_graph(weight_autotune);
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        // Z error on Z stabilizers will not propagate to 2 data qubits along the logical operator direction
        model.clear_error();
        model.add_error_at(el2t(0), 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
        model.clear_error();
        model.add_error_at(el2t(0)+1, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
        model.clear_error();
        model.add_error_at(el2t(0)+2, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
        model.clear_error();
        model.add_error_at(el2t(0)+3, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2)]);  // fine
        model.clear_error();
        model.add_error_at(el2t(0)+4, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 1, 2), (30, 3, 0)]);  // fine
        model.clear_error();
        model.add_error_at(el2t(0)+5, 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(30, 3, 0), (30, 3, 2)]);  // fine

        model.clear_error();
        model.add_error_at(el2t(0), 1, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
        model.clear_error();
        model.add_error_at(el2t(0)+1, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
        model.clear_error();
        model.add_error_at(el2t(0)+2, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![]);
        model.clear_error();
        model.add_error_at(el2t(0)+3, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 0, 1)]);  // fine
        model.clear_error();
        model.add_error_at(el2t(0)+4, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 0, 1), (30, 2, 3)]);  // fine: fixed!!!
        model.clear_error();
        model.add_error_at(el2t(0)+5, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(30, 2, 1), (30, 2, 3)]);
        
        // // before the fix of this, 
        // model.clear_error();
        // model.add_error_at(el2t(0), 1, 2, &ErrorType::X).expect("error rate = 0 here");
        // assert_error_is(&mut model, vec![]);
        // model.clear_error();
        // model.add_error_at(el2t(0)+1, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        // assert_error_is(&mut model, vec![]);
        // model.clear_error();
        // model.add_error_at(el2t(0)+2, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        // assert_error_is(&mut model, vec![]);
        // model.clear_error();
        // model.add_error_at(el2t(0)+3, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        // assert_error_is(&mut model, vec![(24, 0, 1)]);  // fine
        // model.clear_error();
        // model.add_error_at(el2t(0)+4, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        // assert_error_is(&mut model, vec![(30, 2, 1)]);  // NO!!! this will cause logical error!!! a single error in circuit-level noise model causes a logical error, this is unacceptable
        // // to fix this issue, refer to arXiv:1404.3747v3, the order of Z stabilizers and X stabilizers should be different to fight against this, i.e. one should be "Z" shape, the other should be "S" 
        // // enable feature "reordered_css_gates" to fix this issue (by default enabled)
        // model.clear_error();
        // model.add_error_at(el2t(0)+5, 1, 2, &ErrorType::X).expect("error rate = 0 here");
        // assert_error_is(&mut model, vec![(30, 2, 1), (30, 2, 3)]);
    }

    #[test]
    fn tailored_code_test_decode_phenomenological() {
        let measurement_rounds = 5;
        let d = 5;
        let p = 0.01;  // physical error rate
        let bias_eta = 100.;
        let mut model = PlanarCodeModel::new_rotated_tailored_code(measurement_rounds, d);
        model.apply_error_model(&ErrorModelName::TailoredPhenomenological, None, p, bias_eta, 0.);
        model.build_graph(weight_autotune);
        model.build_exhausted_path();
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        model.clear_error();
        // single error at data qubit
        model.add_error_at(el2t(0), 4, 4, &ErrorType::Z).expect("error rate = 0 here");
        assert_error_is(&mut model, vec![(24, 3, 4), (24, 4, 3), (24, 4, 5), (24, 5, 4)]);
        // single measurement error on Y stabilizers
        // model.add_error_at(el2t(2), 2, 1, &ErrorType::Z).expect("error rate = 0 here");
        // assert_error_is(&mut model, vec![(18, 2, 1), (24, 2, 1)]);  // single measurement error, because it just flip the measurement result
        model.propagate_error();
        let measurement = model.generate_measurement();
        let (correction, runtime_statistics) = model.decode_MWPM(&measurement);
        println!("runtime_statistics: {:?}", runtime_statistics);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        println!("validation_ret: {:?}", validation_ret);
    }

}
