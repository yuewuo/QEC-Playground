//! # Fast Benchmark
//!
//! Yue Wu @ 10/17/2021
//!
//! Analyzing logical error rate at extremely low physical error rate is exponentially more time consuming.
//! https://arxiv.org/abs/1208.1334 shows a way to analytically estimate the logical error rate much faster at low error rate.
//! Inspired by that, I designed an estimator targeting higher accuracy and even more general.
//!
//! ## 1. Generality more than Fitting
//!
//! In the original paper, Dr.Fowler assumes the asymptotic logical error rate $p_L = A p^{d/2}$.
//! I don't assume any relation between the logical error rate and physical error rate, but directly estimate the error rate.
//! This is required when we have both erasure errors and Pauli errors and the transition of the major error happens at very low error rate, and we need to confirm the transition before we can assume the relationship.
//! 
//! ## 2. Generality with Erasure Errors
//!
//! We take erasure errors into account, possibly mixed with all kinds of Pauli errors.
//! 
//! ## 3. Generality with Different Decoders
//!
//! This fast benchmark estimator is decoupled from the decoder implementation.
//! People can test different decoders using this same estimator by input a customized decoding closure function.
//!
//! ## 4. Higher Accuracy with Randomizing Sub-Routine
//!
//! We not only consider a single error chain connecting the two opposite boundaries, but also randomize errors at all other positions so that it's more accurate especially when the decoder is far from optimal.
//! For example, union-find decoder has exactly the same decoding accuracy with MWPM decoder when there is only a single error chain, and the decoding accuracy only shows up when we have 2D or 3D randomized errors.
//! This randomizing sub-routine opens opportunities to reveal the difference between decoders, together with the generality beyond simple coefficient fitting.
//!


#![allow(non_snake_case)]
#![allow(dead_code)]

use super::ftqec;
use ftqec::{GateType, CodeType, Index};
use super::types::{QubitType, ErrorType, CorrelatedErrorType};
use std::collections::{BTreeMap, BTreeSet};
use super::either::Either;


pub struct PossiblePauli {
    pub pauli_type: Either<ErrorType, CorrelatedErrorType>,
    pub pauli_position: (usize, usize, usize),
    pub probability: f64,
}

pub struct PossibleErasure {
    pub erasure_position: (usize, usize, usize),
    pub probability: f64,
}

pub struct PossibleMatch {
    pub pauli_matches: Vec<PossiblePauli>,
    pub erasures_matches: Vec<PossibleErasure>,
    pub joint_probability: f64,
}

pub struct FBNode {
    pub mt: usize,
    pub i: usize,
    pub j: usize,
    // match information
    pub matches: BTreeMap<(usize, usize, usize), PossibleMatch>,
    // boundary information
    pub boundary_joint_probability: f64,
    pub pauli_boundaries: Vec<PossiblePauli>,
    pub erasure_boundaries: Vec<PossibleErasure>,
    // internal static information
    pub hop_right: Option<usize>,  // how many hops to left boundary (j = 2dj - 3) if applicable
    pub hop_front: Option<usize>,  // how many hops to front boundary (i = 2di - 3) if applicable
    // configurations
    pub weighted_path_sampling: bool,
    pub weighted_assignment_sampling: bool,
    // internal states
    possible_left_boundary: bool,
    possible_right_boundary: bool,
    possible_back_boundary: bool,
    possible_front_boundary: bool,
    string_count: usize,  // only valid when `possible_left_boundary` or `possible_back_boundary` is true
    // temporary registers
    path_counter: usize,
}

pub struct FastBenchmark {
    pub fb_nodes: Vec::< Vec::< Vec::< Option<FBNode> > > >,

}

impl FastBenchmark {
    pub fn new(model: &ftqec::PlanarCodeModel) -> Self {
        let mut fb_nodes = Vec::new();
        for mt in 0..model.MeasurementRounds + 1 {
            let t = Index::from_measurement_idx(mt, 0, 0).t;
            let array = &model.snapshot[t];
            let mut fb_array_t = Vec::new();
            for (i, array) in array.iter().enumerate() {
                let mut fb_array_i = Vec::new();
                for (j, element) in array.iter().enumerate() {
                    fb_array_i.push(match element {
                        Some(ref e) => {
                            if e.qubit_type != QubitType::Data && e.gate_type == GateType::Measurement {
                                Some(FBNode::new(mt, i, j))
                            } else {
                                None
                            }
                        },
                        None => None,
                    });
                }
                fb_array_t.push(fb_array_i);
            }
            fb_nodes.push(fb_array_t);
        }
        match model.code_type {
            CodeType::StandardPlanarCode | CodeType::StandardXZZXCode => {
                for (_mt, array) in fb_nodes.iter_mut().enumerate() {
                    for (i, array) in array.iter_mut().enumerate() {
                        for (j, element) in array.iter_mut().enumerate() {
                            match element {
                                Some(ref mut fb_node) => {
                                    let mut cnt = 0;
                                    if j == 1 {
                                        fb_node.possible_left_boundary = true;
                                        cnt += 1;
                                    }
                                    if j == 2 * model.dj - 3 {
                                        fb_node.possible_right_boundary = true;
                                        cnt += 1;
                                    }
                                    if i == 1 {
                                        fb_node.possible_back_boundary = true;
                                        cnt += 1;
                                    }
                                    if i == 2 * model.di - 3 {
                                        fb_node.possible_front_boundary = true;
                                        cnt += 1;
                                    }
                                    assert!(cnt == 0 || cnt == 1, "being multiple boundaries confuses me");
                                },
                                None => { },
                            }
                        }
                    }
                }
            },
            _ => unimplemented!("fast benchmark not implemented for this code type")
        }
        FastBenchmark {
            fb_nodes: fb_nodes,
        }
    }

    pub fn add_possible_boundary(&mut self, t1: usize, i1: usize, j1: usize, p: f64, te: usize, ie: usize, je: usize
            , pauli_or_erasure: Either<Either<ErrorType, CorrelatedErrorType>, ()>) {
        let (mt, i, j) = Index::new(t1, i1, j1).to_measurement_idx();
        let mut fb_node = self.fb_nodes[mt][i][j].as_mut().expect("exist");
        assert!(fb_node.possible_left_boundary || fb_node.possible_right_boundary || fb_node.possible_back_boundary
            || fb_node.possible_front_boundary, "unrecognized boundary");
        if fb_node.possible_right_boundary {
            fb_node.hop_right = Some(1);
        }
        if fb_node.possible_front_boundary {
            fb_node.hop_front = Some(1);
        }
        match pauli_or_erasure {
            Either::Left(pauli_type) => {
                fb_node.pauli_boundaries.push(PossiblePauli {
                    pauli_type: pauli_type,
                    pauli_position: (te, ie, je),
                    probability: p,
                });
            },
            Either::Right(_) => {
                fb_node.erasure_boundaries.push(PossibleErasure {
                    erasure_position: (te, ie, je),
                    probability: p,
                });
            },
        }
        fb_node.boundary_joint_probability = fb_node.boundary_joint_probability * (1. - p) + p * (1. - fb_node.boundary_joint_probability);
    }

    pub fn add_possible_match(&mut self, t1: usize, i1: usize, j1: usize, t2: usize, i2: usize, j2: usize, p: f64, te: usize, ie: usize, je: usize
            , pauli_or_erasure: Either<Either<ErrorType, CorrelatedErrorType>, ()>) {
        let mt1 = Index::new(t1, i1, j1).to_measurement_idx().0;
        let mt2 = Index::new(t2, i2, j2).to_measurement_idx().0;
        for (mt, i, j, mtp, ip, jp) in [(mt1, i1, j1, mt2, i2, j2), (mt2, i2, j2, mt1, i1, j1)] {  // p for peer
            let fb_node = self.fb_nodes[mt][i][j].as_mut().expect("exist");
            if !fb_node.matches.contains_key(&(mtp, ip, jp)) {
                fb_node.matches.insert((mtp, ip, jp), PossibleMatch::new()); 
            }
            let possible_match = fb_node.matches.get_mut(&(mtp, ip, jp)).expect("just inserted");
            match pauli_or_erasure {
                Either::Left(ref pauli_type) => {
                    possible_match.pauli_matches.push(PossiblePauli {
                        pauli_type: pauli_type.clone(),
                        pauli_position: (te, ie, je),
                        probability: p,
                    });
                },
                Either::Right(_) => {
                    possible_match.erasures_matches.push(PossibleErasure {
                        erasure_position: (te, ie, je),
                        probability: p,
                    });
                },
            }
            possible_match.joint_probability = possible_match.joint_probability * (1. - p) + p * (1. - possible_match.joint_probability);
        }
    }

    /// called to prepare for the computation
    pub fn prepare(&mut self) {
        self.prepare_boundary_hop();
        self.prepare_string_counts();
    }

    pub fn prepare_boundary_hop(&mut self) {
        let mut has_update = true;
        let mt_len = self.fb_nodes.len();
        let i_len = self.fb_nodes[0].len();
        let j_len = self.fb_nodes[0][0].len();
        while has_update {
            has_update = false;
            for mt in 0..mt_len {
                for i in 0..i_len {
                    for j in 0..j_len {
                        let mut update_hop_right = None;
                        let mut update_hop_front = None;
                        match self.fb_nodes[mt][i][j] {
                            Some(ref fb_node) => {
                                for (&(mtp, ip, jp), _possible_match) in fb_node.matches.iter() {
                                    let fb_node_peer = self.fb_nodes[mtp][ip][jp].as_ref().expect("exist");
                                    match fb_node_peer.hop_right {
                                        Some(hop_right_peer) => {
                                            if update_hop_right.is_none() {
                                                update_hop_right = Some(hop_right_peer + 1);
                                            } else {
                                                update_hop_right = Some(std::cmp::min(hop_right_peer + 1, update_hop_right.unwrap()))
                                            }
                                        }, None => { }
                                    }
                                    match fb_node_peer.hop_front {
                                        Some(hop_front_peer) => {
                                            if update_hop_front.is_none() {
                                                update_hop_front = Some(hop_front_peer + 1);
                                            } else {
                                                update_hop_front = Some(std::cmp::min(hop_front_peer + 1, update_hop_front.unwrap()))
                                            }
                                        }, None => { }
                                    }
                                }
                            }
                            None => { }
                        }
                        if update_hop_right.is_some() || update_hop_front.is_some() {
                            let fb_node = self.fb_nodes[mt][i][j].as_mut().expect("exist");
                            match update_hop_right {
                                Some(update_hop_right) => {
                                    if fb_node.hop_right.is_none() {
                                        fb_node.hop_right = Some(update_hop_right);
                                        has_update = true;
                                    } else {
                                        if update_hop_right < fb_node.hop_right.unwrap() {
                                            fb_node.hop_right = Some(update_hop_right);
                                            has_update = true;
                                        }
                                    }
                                }, None => { }
                            }
                            match update_hop_front {
                                Some(update_hop_front) => {
                                    if fb_node.hop_front.is_none() {
                                        fb_node.hop_front = Some(update_hop_front);
                                        has_update = true;
                                    } else {
                                        if update_hop_front < fb_node.hop_front.unwrap() {
                                            fb_node.hop_front = Some(update_hop_front);
                                            has_update = true;
                                        }
                                    }
                                }, None => { }
                            }
                        }
                    }
                }
            }
        }
    }

    fn clear_path_counter(&mut self) {
        for (_mt, array) in self.fb_nodes.iter_mut().enumerate() {
            for (_i, array) in array.iter_mut().enumerate() {
                for (_j, element) in array.iter_mut().enumerate() {
                    match element {
                        Some(ref mut fb_node) => {
                            fb_node.path_counter = 0;
                        }, None => { }
                    }
                }
            }
        }
    }

    pub fn prepare_string_counts(&mut self) {
        let mt_len = self.fb_nodes.len();
        let i_len = self.fb_nodes[0].len();
        let j_len = self.fb_nodes[0][0].len();
        for mt in 0..mt_len {
            for i in 0..i_len {
                for j in 0..j_len {
                    self.clear_path_counter();
                    let mut run_with_hop_count = None;
                    match self.fb_nodes[mt][i][j] {
                        Some(ref fb_node) => {
                            if (fb_node.possible_left_boundary || fb_node.possible_back_boundary) && fb_node.boundary_joint_probability > 0. {
                                // println!("[{}][{}][{}] {} {} {}", mt, i, j, fb_node.possible_left_boundary, fb_node.possible_back_boundary, fb_node.boundary_joint_probability);
                                match fb_node.hop_front.clone().or(fb_node.hop_right.clone()) {
                                    Some(hop_count) => {
                                        // println!("[{}][{}][{}] hop_count: {}", mt, i, j, hop_count);
                                        run_with_hop_count = Some(hop_count);
                                    }, None => { }
                                }
                            }
                        }, None => { }
                    }
                    match run_with_hop_count {
                        Some(hop_count) => {
                            self.fb_nodes[mt][i][j].as_mut().unwrap().path_counter = 1;
                            let mut growing = BTreeSet::new();
                            growing.insert((mt, i, j));
                            for required_hop in (1..hop_count).rev() {
                                let mut next_growing = BTreeSet::new();
                                // println!("required_hop: {}, growing.len(): {}", required_hop, growing.len());
                                for &(mtg, ig, jg) in growing.iter() {
                                    let mut matches = Vec::new();
                                    let path_counter = self.fb_nodes[mtg][ig][jg].as_ref().unwrap().path_counter;
                                    for (&(mtp, ip, jp), _possible_match) in self.fb_nodes[mtg][ig][jg].as_ref().unwrap().matches.iter() {
                                        matches.push((mtp, ip, jp));
                                    }
                                    for &(mtp, ip, jp) in matches.iter() {
                                        let fb_node_peer = self.fb_nodes[mtp][ip][jp].as_mut().expect("exist");
                                        match fb_node_peer.hop_front.clone().or(fb_node_peer.hop_right.clone()) {
                                            Some(hop_count) => {
                                                if hop_count == required_hop {
                                                    fb_node_peer.path_counter += path_counter;
                                                    next_growing.insert((mtp, ip, jp));
                                                }
                                            }, None => { }
                                        }
                                    }
                                }
                                growing = next_growing;
                            }
                            let mut string_count = 0;
                            for &(mtg, ig, jg) in growing.iter() {
                                // println!("growing [{}][{}][{}]: path_counter {}", mtg, ig, jg, self.fb_nodes[mtg][ig][jg].as_ref().unwrap().path_counter);
                                string_count += self.fb_nodes[mtg][ig][jg].as_ref().unwrap().path_counter;
                            }
                            // println!("[{}][{}][{}] final growing.len(): {}, string_count: {}", mt, i, j, growing.len(), string_count);
                        }, None => { }
                    }
                }
            }
        }
        self.clear_path_counter();
    }

    /// error estimate
    pub fn benchmark(&mut self) {

    }

    pub fn debug_print(&self) {
        for (mt, array) in self.fb_nodes.iter().enumerate() {
            for (i, array) in array.iter().enumerate() {
                for (j, element) in array.iter().enumerate() {
                    match element {
                        Some(ref fb_node) => {
                            // let t = Index::from_measurement_idx(mt, 0, 0).t;
                            println!("[{}][{}][{}] right={:?} front={:?}", mt, i, j, fb_node.hop_right, fb_node.hop_front);
                            if fb_node.boundary_joint_probability > 0. {  // print pauli and erasure boundaries
                                println!("  Boundary joint probability: {}", fb_node.boundary_joint_probability);
                                for possible_pauli in fb_node.pauli_boundaries.iter() {
                                    println!("    Pauli: {} {}", possible_pauli.probability, possible_pauli.pretty_error());
                                }
                                for possible_erasure in fb_node.erasure_boundaries.iter() {
                                    println!("    erasure: {} at {:?}", possible_erasure.probability, possible_erasure.erasure_position);
                                }
                            }
                            {  // print matches
                                for ((mtp, ip, jp), possible_match) in fb_node.matches.iter() {
                                    println!("  Match with [{}][{}][{}] joint probability: {}", mtp, ip, jp, possible_match.joint_probability);
                                    for possible_pauli in possible_match.pauli_matches.iter() {
                                        println!("    Pauli: {} {}", possible_pauli.probability, possible_pauli.pretty_error());
                                    }
                                    for possible_erasure in possible_match.erasures_matches.iter() {
                                        println!("    erasure: {} at {:?}", possible_erasure.probability, possible_erasure.erasure_position);
                                    }
                                }
                            }
                        }
                        None => { }
                    }
                }
            }
        }
    }
}

impl FBNode {
    pub fn new(mt: usize, i: usize, j: usize) -> Self {
        FBNode {
            mt: mt,
            i: i,
            j: j,
            matches: BTreeMap::new(),
            boundary_joint_probability: 0.,
            pauli_boundaries: Vec::new(),
            erasure_boundaries: Vec::new(),
            hop_right: None,
            hop_front: None,
            weighted_path_sampling: true,
            weighted_assignment_sampling: true,
            // internal state
            possible_left_boundary: false,
            possible_right_boundary: false,  // possible search for hop_right=Some(1)
            possible_back_boundary: false,
            possible_front_boundary: false,  // possible search for hop_front=Some(1)
            string_count: 0,
            // temporary state
            path_counter: 0,
        }
    }
}

impl PossiblePauli {
    fn pretty_error(&self) -> String {
        match &self.pauli_type {
            Either::Left(error_type) => format!("{}{:?}", error_type, self.pauli_position),
            Either::Right(error_type) => format!("{}{:?}", error_type, self.pauli_position),
        }
    }
}

impl PossibleMatch {
    fn new() -> Self {
        Self {
            pauli_matches: Vec::new(),
            erasures_matches: Vec::new(),
            joint_probability: 0.,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use `cargo test fast_benchmark_1 -- --nocapture` to run specific test

    #[test]
    fn fast_benchmark_1() {
        let d = 3;
        let p = 0.01;
        let pe = 0.02;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(d, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(p/3., p/3., p/3., pe);
        let mut fast_benchmark = model.build_graph_and_build_fast_benchmark();
        fast_benchmark.prepare();
        // fast_benchmark.debug_print();
        model.optimize_correction_pattern();
        model.build_exhausted_path_autotune();
        // run benchmark

        
    }

}
