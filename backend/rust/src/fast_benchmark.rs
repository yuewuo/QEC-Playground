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
use super::rand::seq::SliceRandom;
use super::rand_core::{RngCore};
use super::rug;
use super::rug::Complete;


#[derive(Debug, PartialEq, Clone)]
pub struct PossiblePauli {
    pub pauli_type: Either<ErrorType, CorrelatedErrorType>,
    pub pauli_position: (usize, usize, usize),
    pub probability: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PossibleErasure {
    pub erasure_position: (usize, usize, usize),
    pub probability: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PossibleMatch {
    pub pauli_matches: Vec<PossiblePauli>,
    pub pauli_joint_probability: f64,
    pub erasure_matches: Vec<PossibleErasure>,
    pub erasure_joint_probability: f64,
    pub joint_probability: f64,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum BoundaryCandidate {
    Left,
    Right,
    Back,
    Front,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
enum StringElementType {
    Boundary,
    MatchNext,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum AssignmentElementType {
    Pauli,
    Erasure,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FBNode {
    pub mt: usize,
    pub i: usize,
    pub j: usize,
    // match information
    pub matches: BTreeMap<(usize, usize, usize), PossibleMatch>,
    // boundary information
    pub boundary_joint_probability: f64,
    pub pauli_boundaries: Vec<PossiblePauli>,
    pub pauli_boundaries_joint_probability: f64,
    pub erasure_boundaries: Vec<PossibleErasure>,
    pub erasure_boundaries_joint_probability: f64,
    // internal static information
    pub hop: Option<usize>,  // how many hops to left boundary (j = 2dj - 3) or front boundary (i = 2di - 3) if applicable
    // internal states
    boundary_candidate: Option<BoundaryCandidate>,
    string_count: rug::Integer,  // only valid when `boundary_candidate` is Left or Back
    sampling_k: usize,
    sampling_sum_ps: rug::Float,
    sampling_sum_elements: rug::Float,
    // temporary registers and sampling registers
    path_counter: rug::Integer,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FastBenchmark {
    pub fb_nodes: Vec< Vec< Vec< Option<FBNode> > > >,
    starting_nodes: Vec<(usize, usize, usize)>,
    // configurations
    pub use_weighted_path_sampling: bool,
    pub use_weighted_assignment_sampling: bool,
    pub assignment_sampling_amount: usize,
    pub use_simple_sum: bool,
    pub rug_precision: u32,  // default to 128 bits
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
                                    if j == 1 {
                                        assert!(fb_node.boundary_candidate.is_none(), "cannot be multiple type of boundary at once");
                                        fb_node.boundary_candidate = Some(BoundaryCandidate::Left);
                                    }
                                    if j == 2 * model.dj - 3 {
                                        assert!(fb_node.boundary_candidate.is_none(), "cannot be multiple type of boundary at once");
                                        fb_node.boundary_candidate = Some(BoundaryCandidate::Right);
                                    }
                                    if i == 1 {
                                        assert!(fb_node.boundary_candidate.is_none(), "cannot be multiple type of boundary at once");
                                        fb_node.boundary_candidate = Some(BoundaryCandidate::Back);
                                    }
                                    if i == 2 * model.di - 3 {
                                        assert!(fb_node.boundary_candidate.is_none(), "cannot be multiple type of boundary at once");
                                        fb_node.boundary_candidate = Some(BoundaryCandidate::Front);
                                    }
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
            starting_nodes: Vec::new(),
            use_weighted_path_sampling: true,
            use_weighted_assignment_sampling: true,
            assignment_sampling_amount: 10,
            use_simple_sum: false,
            rug_precision: 128,  // bits
        }
    }

    pub fn add_possible_boundary(&mut self, t1: usize, i1: usize, j1: usize, p: f64, te: usize, ie: usize, je: usize
            , pauli_or_erasure: Either<Either<ErrorType, CorrelatedErrorType>, ()>) {
        let (mt, i, j) = Index::new(t1, i1, j1).to_measurement_idx();
        let mut fb_node = self.fb_nodes[mt][i][j].as_mut().expect("exist");
        assert!(fb_node.boundary_candidate.is_some(), "unrecognized boundary, remember to add to boundary candidate");
        if Self::is_ending_boundary(fb_node) {
            fb_node.hop = Some(1);
        }
        match pauli_or_erasure {
            Either::Left(pauli_type) => {
                fb_node.pauli_boundaries.push(PossiblePauli {
                    pauli_type: pauli_type,
                    pauli_position: (te, ie, je),
                    probability: p,
                });
                fb_node.pauli_boundaries_joint_probability = fb_node.pauli_boundaries_joint_probability * (1. - p) + p * (1. - fb_node.pauli_boundaries_joint_probability);
            },
            Either::Right(_) => {
                // need to check whether this erasure has already been in here
                for possible_erasure in fb_node.erasure_boundaries.iter() {
                    if possible_erasure.erasure_position == (te, ie, je) {
                        assert_eq!(possible_erasure.probability, p, "erasure of same position should be added with joint probability");
                        return  // already been here, avoid duplicate
                    }
                }
                fb_node.erasure_boundaries.push(PossibleErasure {
                    erasure_position: (te, ie, je),
                    probability: p,
                });
                fb_node.erasure_boundaries_joint_probability = fb_node.erasure_boundaries_joint_probability * (1. - p) + p * (1. - fb_node.erasure_boundaries_joint_probability);
            },
        }
        fb_node.boundary_joint_probability = fb_node.boundary_joint_probability * (1. - p) + p * (1. - fb_node.boundary_joint_probability);
    }

    pub fn add_possible_match(&mut self, t1: usize, i1: usize, j1: usize, t2: usize, i2: usize, j2: usize, p: f64, te: usize, ie: usize, je: usize
            , pauli_or_erasure: Either<Either<ErrorType, CorrelatedErrorType>, ()>) {
        let mt1 = Index::new(t1, i1, j1).to_measurement_idx().0;
        let mt2 = Index::new(t2, i2, j2).to_measurement_idx().0;
        for &(mt, i, j, mtp, ip, jp) in [(mt1, i1, j1, mt2, i2, j2), (mt2, i2, j2, mt1, i1, j1)].iter() {  // p for peer
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
                    possible_match.pauli_joint_probability = possible_match.pauli_joint_probability * (1. - p) + p * (1. - possible_match.pauli_joint_probability);
                },
                Either::Right(_) => {
                    // need to check whether this erasure has already been in here
                    for possible_erasure in possible_match.erasure_matches.iter() {
                        if possible_erasure.erasure_position == (te, ie, je) {
                            assert_eq!(possible_erasure.probability, p, "erasure of same position should be added with joint probability");
                            return  // already been here, avoid duplicate
                        }
                    }
                    possible_match.erasure_matches.push(PossibleErasure {
                        erasure_position: (te, ie, je),
                        probability: p,
                    });
                    possible_match.erasure_joint_probability = possible_match.erasure_joint_probability * (1. - p) + p * (1. - possible_match.erasure_joint_probability);
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
                        let mut update_hop = None;
                        match self.fb_nodes[mt][i][j] {
                            Some(ref fb_node) => {
                                for (&(mtp, ip, jp), _possible_match) in fb_node.matches.iter() {
                                    let fb_node_peer = self.fb_nodes[mtp][ip][jp].as_ref().expect("exist");
                                    match fb_node_peer.hop {
                                        Some(hop_peer) => {
                                            if update_hop.is_none() {
                                                update_hop = Some(hop_peer + 1);
                                            } else {
                                                update_hop = Some(std::cmp::min(hop_peer + 1, update_hop.unwrap()))
                                            }
                                        }, None => { }
                                    }
                                }
                            }
                            None => { }
                        }
                        if update_hop.is_some() {
                            let fb_node = self.fb_nodes[mt][i][j].as_mut().expect("exist");
                            match update_hop {
                                Some(update_hop) => {
                                    if fb_node.hop.is_none() {
                                        fb_node.hop = Some(update_hop);
                                        has_update = true;
                                    } else {
                                        if update_hop < fb_node.hop.unwrap() {
                                            fb_node.hop = Some(update_hop);
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
                            fb_node.path_counter = rug::Integer::from(0);
                        }, None => { }
                    }
                }
            }
        }
    }

    fn is_starting_boundary(fb_node: &FBNode) -> bool {
        match fb_node.boundary_candidate {
            Some(BoundaryCandidate::Left) | Some(BoundaryCandidate::Back) => true,
            _ => false,
        }
    }

    fn is_ending_boundary(fb_node: &FBNode) -> bool {
        match fb_node.boundary_candidate {
            Some(BoundaryCandidate::Right) | Some(BoundaryCandidate::Front) => true,
            _ => false,
        }
    }

    pub fn prepare_string_counts(&mut self) {
        let mt_len = self.fb_nodes.len();
        let i_len = self.fb_nodes[0].len();
        let j_len = self.fb_nodes[0][0].len();
        let mut starting_nodes = Vec::new();
        for mt in 0..mt_len {
            for i in 0..i_len {
                for j in 0..j_len {
                    self.clear_path_counter();
                    let mut run_with_hop = None;
                    match self.fb_nodes[mt][i][j] {
                        Some(ref fb_node) => {
                            if Self::is_starting_boundary(fb_node) && fb_node.boundary_joint_probability > 0. {
                                starting_nodes.push((mt, i, j));
                                // println!("[{}][{}][{}] {} {} {}", mt, i, j, fb_node.possible_left_boundary, fb_node.possible_back_boundary, fb_node.boundary_joint_probability);
                                match fb_node.hop.clone() {
                                    Some(hop) => {
                                        // println!("[{}][{}][{}] hop: {}", mt, i, j, hop);
                                        run_with_hop = Some(hop);
                                    }, None => { }
                                }
                            }
                        }, None => { }
                    }
                    match run_with_hop {
                        Some(hop) => {
                            self.fb_nodes[mt][i][j].as_mut().unwrap().path_counter = rug::Integer::from(1);
                            let mut growing = BTreeSet::new();
                            growing.insert((mt, i, j));
                            for required_hop in (1..hop).rev() {  // note: hop-1, hop-2, ..., 1
                                let mut next_growing = BTreeSet::new();
                                // println!("required_hop: {}, growing.len(): {}", required_hop, growing.len());
                                for &(mtg, ig, jg) in growing.iter() {
                                    let mut matches = Vec::new();
                                    let path_counter = self.fb_nodes[mtg][ig][jg].as_ref().unwrap().path_counter.clone();
                                    for (&(mtp, ip, jp), _possible_match) in self.fb_nodes[mtg][ig][jg].as_ref().unwrap().matches.iter() {
                                        matches.push((mtp, ip, jp));
                                    }
                                    for &(mtp, ip, jp) in matches.iter() {
                                        let fb_node_peer = self.fb_nodes[mtp][ip][jp].as_mut().expect("exist");
                                        match fb_node_peer.hop.clone() {
                                            Some(hop_peer) => {
                                                if hop_peer == required_hop {
                                                    fb_node_peer.path_counter += path_counter.clone();
                                                    next_growing.insert((mtp, ip, jp));
                                                }
                                            }, None => { }
                                        }
                                    }
                                }
                                growing = next_growing;
                            }
                            let mut string_count = rug::Integer::from(0);
                            for &(mtg, ig, jg) in growing.iter() {
                                // println!("growing [{}][{}][{}]: path_counter {}", mtg, ig, jg, self.fb_nodes[mtg][ig][jg].as_ref().unwrap().path_counter);
                                string_count += self.fb_nodes[mtg][ig][jg].as_ref().unwrap().path_counter.clone();
                            }
                            // println!("[{}][{}][{}] final growing.len(): {}, string_count: {}", mt, i, j, growing.len(), string_count);
                            self.fb_nodes[mt][i][j].as_mut().unwrap().string_count = string_count;
                        }, None => { }
                    }
                }
            }
        }
        self.clear_path_counter();
        // println!("starting_nodes: {:?}", starting_nodes);
        self.starting_nodes = starting_nodes;
    }

    /// add a single error estimate for each left starting point
    pub fn benchmark_once<F>(&mut self, rng: &mut impl RngCore, mut decode: F)
            where F: FnMut(Vec<(usize, usize, usize, Either<Either<ErrorType, CorrelatedErrorType>, ()>)>, &BTreeSet<(usize, usize, usize)>, usize) -> bool {
            // Vec<(te, ie, je, pauli_or_erasure)>, string_d
        for starting_nodes_idx in 0..self.starting_nodes.len() {
            self.benchmark_starting_node(rng, &mut decode, starting_nodes_idx);
        }
    }

    pub fn benchmark_random_starting_node<F>(&mut self, rng: &mut impl RngCore, mut decode: F)
            where F: FnMut(Vec<(usize, usize, usize, Either<Either<ErrorType, CorrelatedErrorType>, ()>)>, &BTreeSet<(usize, usize, usize)>, usize) -> bool {
        let mut smallest_k = None;
        let mut smallest_idx = None;
        for i in 0..self.starting_nodes.len() {
            let (mts, is, js) = self.starting_nodes[i];
            let fb_node = self.fb_nodes[mts][is][js].as_ref().unwrap();
            if smallest_k.is_none() || fb_node.sampling_k < smallest_k.unwrap() {
                smallest_k = Some(fb_node.sampling_k);
                smallest_idx = Some(i);
            }
        }
        // let mut smallest_indices = Vec::new();
        // for i in 0..self.starting_nodes.len() {
        //     let (mts, is, js) = self.starting_nodes[i];
        //     let fb_node = self.fb_nodes[mts][is][js].as_ref().unwrap();
        //     if fb_node.sampling_k == smallest_k.unwrap() {
        //         smallest_indices.push(i);
        //     }
        // }
        // let starting_nodes_idx = *smallest_indices.choose(rng).unwrap();
        let starting_nodes_idx = smallest_idx.unwrap();
        self.benchmark_starting_node(rng, &mut decode, starting_nodes_idx)
    }

    /// run benchmark for a
    pub fn benchmark_starting_node<F>(&mut self, rng: &mut impl RngCore, decode: &mut F, starting_nodes_idx: usize)
            where F: FnMut(Vec<(usize, usize, usize, Either<Either<ErrorType, CorrelatedErrorType>, ()>)>, &BTreeSet<(usize, usize, usize)>, usize) -> bool {
            // Vec<(te, ie, je, pauli_or_erasure)>, string_d
        let (mts, is, js) = self.starting_nodes[starting_nodes_idx];
        let use_weighted_path_sampling = self.use_weighted_path_sampling;
        let use_weighted_assignment_sampling = self.use_weighted_assignment_sampling;
        assert!(self.assignment_sampling_amount >= 1, "at least one sampling is required");
        // sample a path from (mts, is, js) to any end point, whether weighted sample or not
        let hop = self.fb_nodes[mts][is][js].as_ref().unwrap().hop.unwrap();
        let mut sampled_string: Vec<(usize, usize, usize)> = Vec::new();
        let mut sampled_string_ps = rug::Float::with_val(self.rug_precision, 1.);
        let mut selection = Vec::new();
        selection.push(((mts, is, js), 1.));
        for required_hop in (0..hop).rev() {  // note: hop-1, ..., 1, 0
            // randomly choose one based on the weight
            let &((mtc, ic, jc), weight_c) = selection.choose_weighted(rng, |item| item.1).unwrap();
            sampled_string_ps *= weight_c;
            sampled_string.push((mtc, ic, jc));
            // find next selection
            if required_hop == 0 {  // no next selection, stop here
                break
            }
            selection.clear();
            let mut weight_sum = 0.;  // to normalize weight so that large code distance wouldn't hit bound of f64 type
            for (&(mtp, ip, jp), possible_match) in self.fb_nodes[mtc][ic][jc].as_ref().unwrap().matches.iter() {
                let fb_node_peer = self.fb_nodes[mtp][ip][jp].as_ref().expect("exist");
                match fb_node_peer.hop.clone() {
                    Some(hop_peer) => {
                        if hop_peer == required_hop {
                            // use sqrt so that   1. in Pauli case it's more realistic because half of them having error is dominant and that is sqrt(p_all)
                            //                    2. in erasure case although all of them having error is dominant, it's not bad to use a more averaged case sqrt(p_all)
                            // in any case, the weight of the sampling should not have any impact on the result for infinite size simulation, but will only affect the speed of convergence
                            let weight = if use_weighted_path_sampling { possible_match.joint_probability.sqrt() } else { 1. };
                            selection.push(((mtp, ip, jp), weight));
                            weight_sum += weight;
                        }
                    }, None => { }
                }
            }
            let weight_avr = weight_sum / selection.len() as f64;
            for e in selection.iter_mut() {
                e.1 /= weight_avr;
            }
            // println!("{:?}", selection);
        }
        assert!(sampled_string.len() > 1, "why would sample string have no more than 2 nodes?");
        // build clearance region of this sampled string, so that additional errors should not happen at these positions
        let clearance_region = {
            let mut clearance_region = BTreeSet::<(usize, usize, usize)>::new();
            {  // first consider starting boundary
                let fb_node = self.fb_nodes[mts][is][js].as_ref().unwrap();
                for possible_erasure in fb_node.erasure_boundaries.iter() {
                    clearance_region.insert(possible_erasure.erasure_position);
                }
                for possible_pauli in fb_node.pauli_boundaries.iter() {
                    clearance_region.insert(possible_pauli.pauli_position);
                }
            }
            for idx in 0..sampled_string.len()-1 {
                let (mt1, i1, j1) = sampled_string[idx];
                let (mt2, i2, j2) = sampled_string[idx + 1];
                let fb_node = self.fb_nodes[mt1][i1][j1].as_ref().unwrap();
                let possible_match = fb_node.matches.get(&(mt2, i2, j2)).unwrap();
                for possible_erasure in possible_match.erasure_matches.iter() {
                    clearance_region.insert(possible_erasure.erasure_position);
                }
                for possible_pauli in possible_match.pauli_matches.iter() {
                    clearance_region.insert(possible_pauli.pauli_position);
                }
            }
            {   // finally consider ending boundary
                let idx = sampled_string.len() - 1;
                let (mte, ie, je) = sampled_string[idx];
                let fb_node = self.fb_nodes[mte][ie][je].as_ref().unwrap();
                for possible_erasure in fb_node.erasure_boundaries.iter() {
                    clearance_region.insert(possible_erasure.erasure_position);
                }
                for possible_pauli in fb_node.pauli_boundaries.iter() {
                    clearance_region.insert(possible_pauli.pauli_position);
                }
            }
            // println!("clearance_region: {:?}", clearance_region);
            clearance_region
        };
        // build full erasure and selection, which is a fixed vec containing all possible erasure positions
        let (full_erasure_selection, full_pauli_selection) = {
            let mut erasure_weight_sum = 0.;
            let mut full_erasure_selection = Vec::<(usize, StringElementType, f64, f64)>::new();  // idx, string_element_type, weight, type_joint_probability
            let mut pauli_weight_sum = 0.;
            let mut full_pauli_selection = Vec::<(usize, StringElementType, f64, f64)>::new();  // idx, string_element_type, weight, type_joint_probability
            {  // first consider starting boundary
                let fb_node = self.fb_nodes[mts][is][js].as_ref().unwrap();
                if !fb_node.erasure_boundaries.is_empty() {
                    let weight = if use_weighted_assignment_sampling { fb_node.erasure_boundaries_joint_probability } else { 1. };
                    full_erasure_selection.push((0, StringElementType::Boundary, weight, fb_node.erasure_boundaries_joint_probability));
                    erasure_weight_sum += weight;
                }
                if !fb_node.pauli_boundaries.is_empty() {
                    let weight = if use_weighted_assignment_sampling { fb_node.pauli_boundaries_joint_probability } else { 1. };
                    // println!("fb_node.pauli_boundaries_joint_probability: {}", fb_node.pauli_boundaries_joint_probability);
                    full_pauli_selection.push((0, StringElementType::Boundary, weight, fb_node.pauli_boundaries_joint_probability));
                    pauli_weight_sum += weight;
                }
            }
            for idx in 0..sampled_string.len()-1 {
                let (mt1, i1, j1) = sampled_string[idx];
                let (mt2, i2, j2) = sampled_string[idx + 1];
                let fb_node = self.fb_nodes[mt1][i1][j1].as_ref().unwrap();
                let possible_match = fb_node.matches.get(&(mt2, i2, j2)).unwrap();
                if !possible_match.erasure_matches.is_empty() {
                    let weight = if use_weighted_assignment_sampling { possible_match.erasure_joint_probability } else { 1. };
                    full_erasure_selection.push((idx, StringElementType::MatchNext, weight, possible_match.erasure_joint_probability));
                    erasure_weight_sum += weight;
                }
                if !possible_match.pauli_matches.is_empty() {
                    let weight = if use_weighted_assignment_sampling { possible_match.pauli_joint_probability } else { 1. };
                    // println!("possible_match.pauli_joint_probability: {}", possible_match.pauli_joint_probability);
                    full_pauli_selection.push((idx, StringElementType::MatchNext, weight, possible_match.pauli_joint_probability));
                    pauli_weight_sum += weight;
                }
            }
            {   // finally consider ending boundary
                let idx = sampled_string.len() - 1;
                let (mte, ie, je) = sampled_string[idx];
                let fb_node = self.fb_nodes[mte][ie][je].as_ref().unwrap();
                if !fb_node.erasure_boundaries.is_empty() {
                    let weight = if use_weighted_assignment_sampling { fb_node.erasure_boundaries_joint_probability } else { 1. };
                    full_erasure_selection.push((idx, StringElementType::Boundary, weight, fb_node.erasure_boundaries_joint_probability));
                    erasure_weight_sum += weight;
                }
                if !fb_node.pauli_boundaries.is_empty() {
                    let weight = if use_weighted_assignment_sampling { fb_node.pauli_boundaries_joint_probability } else { 1. };
                    full_pauli_selection.push((idx, StringElementType::Boundary, weight, fb_node.pauli_boundaries_joint_probability));
                    pauli_weight_sum += weight;
                }
            }
            let erasure_weight_avr = erasure_weight_sum / full_erasure_selection.len() as f64;
            for e in full_erasure_selection.iter_mut() {
                e.2 /= erasure_weight_avr;
            }
            // println!("full_erasure_selection: {:?}", full_erasure_selection);
            let pauli_weight_avr = pauli_weight_sum / full_pauli_selection.len() as f64;
            for e in full_pauli_selection.iter_mut() {
                e.2 /= pauli_weight_avr;
            }
            // println!("full_pauli_selection: {:?}", full_pauli_selection);
            (full_erasure_selection, full_pauli_selection)
        };
        // given the sampled string, now calculate the logical error rate on this string by iterating over all possible combinations
        let mut string_logical_error_rate = ErrorRateAccumulator::new_use_simple_sum(self.use_simple_sum, self.rug_precision);
        for erasure_count in 0..full_erasure_selection.len()+1 {
            for pauli_count in 0..full_pauli_selection.len()+1 {
                if erasure_count + pauli_count > hop + 1 || (erasure_count == 0 && pauli_count == 0) {
                    continue  // no need to actually sample, it's impossible to have this amount of errors simultaneously
                }
                if erasure_count + 2 * pauli_count < hop {  // this kind of error confuses estimator when adding more errors randomly
                    continue
                }
                let mut combinatorial_of_selection = binomial(full_erasure_selection.len(), erasure_count);
                if erasure_count <= full_pauli_selection.len() && pauli_count <= full_pauli_selection.len() - erasure_count {
                    combinatorial_of_selection *= binomial(full_pauli_selection.len() - erasure_count, pauli_count);
                }
                // println!("erasure_count: {}, pauli_count: {}, combinatorial_of_selection: {}", erasure_count, pauli_count, combinatorial_of_selection);
                let mut assignment_sampling_s = 0;
                let mut assignment_sampling_sum_ps = rug::Float::with_val(self.rug_precision, 0.);
                let mut assignment_sampling_sum_elements = rug::Float::with_val(self.rug_precision, 0.);
                for _assignment_idx in 0..self.assignment_sampling_amount {
                    let mut assignment = Vec::<(usize, StringElementType, AssignmentElementType, f64)>::new();
                    let mut sampling_ps = rug::Float::with_val(self.rug_precision, 1.);
                    // first sample multiple erasure errors
                    let mut erasure_selected_set = BTreeSet::new();
                    if erasure_count > 0 {
                        let erasure_selection = full_erasure_selection.choose_multiple_weighted(rng, erasure_count, |item| item.2).unwrap().collect::<Vec::<&(usize, StringElementType, f64, f64)>>();
                        for &&(idx, string_element_type, weight, typed_joint_probability) in erasure_selection.iter() {
                            sampling_ps = sampling_ps.clone() * weight;
                            assignment.push((idx, string_element_type, AssignmentElementType::Erasure, typed_joint_probability));
                            erasure_selected_set.insert((idx, string_element_type));
                        }
                    }
                    // then sample multiple pauli errors at the remaining positions
                    if pauli_count > 0 {
                        let mut partial_pauli_selection = full_pauli_selection.clone();
                        partial_pauli_selection.retain(|&(idx, string_element_type, _weight, _typed_joint_probability)| !erasure_selected_set.contains(&(idx, string_element_type)));
                        let pauli_selection = match partial_pauli_selection.choose_multiple_weighted(rng, pauli_count, |item| item.2) {
                            Ok(pauli_selection) => pauli_selection.collect::<Vec::<&(usize, StringElementType, f64, f64)>>(),
                            Err(_) => { continue }
                        };
                        for &&(idx, string_element_type, weight, typed_joint_probability) in pauli_selection.iter() {
                            sampling_ps *= weight;
                            assignment.push((idx, string_element_type, AssignmentElementType::Pauli, typed_joint_probability));
                        }
                    }
                    let assignment = assignment;  // make it immutable
                    // println!("    assignment: {:?}", assignment);
                    let has_logical_error = {
                        let mut errors = Vec::new();
                        for &(idx, string_element_type, assignment_element_type, _typed_joint_probability) in assignment.iter() {
                            let (mt1, i1, j1) = sampled_string[idx];
                            let fb_node = self.fb_nodes[mt1][i1][j1].as_ref().unwrap();
                            match string_element_type {
                                StringElementType::Boundary => {
                                    match assignment_element_type {
                                        AssignmentElementType::Erasure => {
                                            let possible_erasure = fb_node.erasure_boundaries.choose_weighted(rng, |item| item.probability).unwrap();
                                            let (te, ie, je) = possible_erasure.erasure_position;
                                            errors.push((te, ie, je, Either::Right(())));
                                        }
                                        AssignmentElementType::Pauli => {
                                            let possible_pauli = fb_node.pauli_boundaries.choose_weighted(rng, |item| item.probability).unwrap();
                                            let (te, ie, je) = possible_pauli.pauli_position;
                                            errors.push((te, ie, je, Either::Left(possible_pauli.pauli_type.clone())));
                                        }
                                    }
                                },
                                StringElementType::MatchNext => {
                                    let (mt2, i2, j2) = sampled_string[idx + 1];
                                    let possible_match = fb_node.matches.get(&(mt2, i2, j2)).unwrap();
                                    match assignment_element_type {
                                        AssignmentElementType::Erasure => {
                                            let possible_erasure = possible_match.erasure_matches.choose_weighted(rng, |item| item.probability).unwrap();
                                            let (te, ie, je) = possible_erasure.erasure_position;
                                            errors.push((te, ie, je, Either::Right(())));
                                        }
                                        AssignmentElementType::Pauli => {
                                            let possible_pauli = possible_match.pauli_matches.choose_weighted(rng, |item| item.probability).unwrap();
                                            let (te, ie, je) = possible_pauli.pauli_position;
                                            errors.push((te, ie, je, Either::Left(possible_pauli.pauli_type.clone())));
                                        }
                                    }
                                },
                            }
                        }
                        let string_d = hop + 1;
                        decode(errors, &clearance_region, string_d)  // run real decoding
                    };
                    assignment_sampling_s += 1;
                    assignment_sampling_sum_ps += sampling_ps.clone();
                    if has_logical_error {
                        let mut physical_error_rate = rug::Float::with_val(self.rug_precision, 1.);
                        assert!(assignment.len() > 0, "should have some errors");
                        for (_idx, _string_element_type, _assignment_element_type, typed_joint_probability) in assignment.iter() {
                            // println!("typed_joint_probability: {}", typed_joint_probability);
                            physical_error_rate *= typed_joint_probability;
                        }
                        assignment_sampling_sum_elements += physical_error_rate / sampling_ps;
                    }
                }
                if assignment_sampling_s > 0 {  // sometimes it's impossible to sample, just ignore this case
                    // println!("    assignment_sampling_s: {}", assignment_sampling_s);
                    let error_rate = ErrorRateAccumulator::new_use_simple_sum(self.use_simple_sum, self.rug_precision).accumulate_multiple(
                        combinatorial_of_selection, assignment_sampling_sum_ps * assignment_sampling_sum_elements
                            / rug::Integer::u_pow_u(assignment_sampling_s, 2).complete()).error_rate.clone();
                    // println!("pauli_count: {}, error_rate: {}, combinatorial_of_selection: {}, sub: {}", pauli_count, error_rate, combinatorial_of_selection, assignment_sampling_sum_ps * assignment_sampling_sum_elements / (assignment_sampling_s as f64).powi(2));
                    string_logical_error_rate.accumulate(error_rate);
                }
            }
        }
        // update state
        let fb_node = self.fb_nodes[mts][is][js].as_mut().unwrap();
        fb_node.sampling_k += 1;
        fb_node.sampling_sum_ps = sampled_string_ps.clone() + fb_node.sampling_sum_ps.clone();  // note: this will use the left-hand precision
        fb_node.sampling_sum_elements = string_logical_error_rate.error_rate / sampled_string_ps.clone()
            + fb_node.sampling_sum_elements.clone();  // note: this will use the left-hand precision
        // println!("sampled_string_ps: {}, sampled_string: {:?}", sampled_string_ps, sampled_string);
    }

    /// get estimated result of logical error rate
    pub fn logical_error_rate(&self) -> rug::Float {
        let mut left_logical_error_rate = ErrorRateAccumulator::new_use_simple_sum(self.use_simple_sum, self.rug_precision);
        let mut back_logical_error_rate = ErrorRateAccumulator::new_use_simple_sum(self.use_simple_sum, self.rug_precision);
        let starting_nodes = &self.starting_nodes;
        let mut not_sampled_count = 0;
        for &(mts, is, js) in starting_nodes.iter() {
            // sample a path from (mts, is, js) to any end point, whether weighted sample or not
            let fb_node = self.fb_nodes[mts][is][js].as_ref().unwrap();
            if fb_node.sampling_k > 0 {
                let acc_p = fb_node.sampling_sum_ps.clone() * fb_node.sampling_sum_elements.clone() / (fb_node.sampling_k as f64).powi(2);
                // println!("[{}][{}][{}]: {} {} {} {} {}", mts, is, js, fb_node.sampling_sum_ps, fb_node.sampling_sum_elements, fb_node.sampling_k, fb_node.string_count, acc_p);
                match fb_node.boundary_candidate {
                    Some(BoundaryCandidate::Left) => {
                        left_logical_error_rate.accumulate_multiple(fb_node.string_count.clone(), acc_p.clone());
                    }
                    Some(BoundaryCandidate::Back) => {
                        back_logical_error_rate.accumulate_multiple(fb_node.string_count.clone(), acc_p.clone());
                    }
                    _ => unreachable!("boundary candidate must be left or back"),
                }
            } else {
                not_sampled_count += 1;
            }
        }
        if not_sampled_count > 0 {
            // println!("not_sampled_count: {}", not_sampled_count);
            let left_divivded_error_rate = left_logical_error_rate.error_rate.clone() / rug::Float::with_val(self.rug_precision, starting_nodes.len() - not_sampled_count);
            left_logical_error_rate = ErrorRateAccumulator::new_use_simple_sum(self.use_simple_sum, self.rug_precision);
            left_logical_error_rate.accumulate_multiple(rug::Integer::from(starting_nodes.len()), left_divivded_error_rate);
            let back_divivded_error_rate = back_logical_error_rate.error_rate.clone() / rug::Float::with_val(self.rug_precision, starting_nodes.len() - not_sampled_count);
            back_logical_error_rate = ErrorRateAccumulator::new_use_simple_sum(self.use_simple_sum, self.rug_precision);
            back_logical_error_rate.accumulate_multiple(rug::Integer::from(starting_nodes.len()), back_divivded_error_rate);
        }
        // println!("left: {}, back: {}", left_logical_error_rate.error_rate, back_logical_error_rate.error_rate);
        // let float_1 = rug::Float::with_val(self.rug_precision, 1.);
        // float_1.clone() - (float_1.clone() - left_logical_error_rate.error_rate) * (float_1.clone() - back_logical_error_rate.error_rate)
        // the above equation, although correct, has untolerant rounding error; use the equivalent equation below
        left_logical_error_rate.error_rate.clone() + back_logical_error_rate.error_rate.clone() -
            left_logical_error_rate.error_rate.clone() * back_logical_error_rate.error_rate.clone()
    }

    pub fn debug_print(&self) {
        for (mt, array) in self.fb_nodes.iter().enumerate() {
            for (i, array) in array.iter().enumerate() {
                for (j, element) in array.iter().enumerate() {
                    match element {
                        Some(ref fb_node) => {
                            // let t = Index::from_measurement_idx(mt, 0, 0).t;
                            println!("[{}][{}][{}] hop={:?} boundary_candidate={:?}", mt, i, j, fb_node.hop, fb_node.boundary_candidate);
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
                                    for possible_erasure in possible_match.erasure_matches.iter() {
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
            pauli_boundaries_joint_probability: 0.,
            erasure_boundaries: Vec::new(),
            erasure_boundaries_joint_probability: 0.,
            hop: None,
            // internal state
            boundary_candidate: None,
            string_count: rug::Integer::from(0),
            sampling_k: 0,
            sampling_sum_ps: rug::Float::with_val(2, 0.),  // it doesn't matter, will later take the higher precision one
            sampling_sum_elements: rug::Float::with_val(2, 0.),
            // temporary state
            path_counter: rug::Integer::from(0),
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
            pauli_joint_probability: 0.,
            erasure_matches: Vec::new(),
            erasure_joint_probability: 0.,
            joint_probability: 0.,
        }
    }
}

/// fake decoding will always succeed when error is less than half (or more precisely 2s + t < d given s pauli errors and t erasure errors)
pub fn fake_decoding(errors: Vec<(usize, usize, usize, Either<Either<ErrorType, CorrelatedErrorType>, ()>)>
        , _clearance_region: &BTreeSet<(usize, usize, usize)>, string_d: usize) -> bool {
    let mut erasure_count = 0;
    let mut pauli_count = 0;
    for (_te, _ie, _je, pauli_or_erasure) in errors.iter() {
        match pauli_or_erasure {
            Either::Left(_) => {
                pauli_count += 1;
            }
            Either::Right(_) => {
                erasure_count += 1;
            }
        }
    }
    if erasure_count + 2 * pauli_count > string_d {
        true
    } else if erasure_count + 2 * pauli_count < string_d {
        false
    } else {
        rand::random()  // fail half of the time
    }
}

pub struct ErrorRateAccumulator {
    /// the accumulated error rate
    pub error_rate: rug::Float,
    /// configure to use simple sum
    pub use_simple_sum: bool,
    // internal state
    rug_precision: u32,
}

impl ErrorRateAccumulator {
    pub fn new(rug_precision: u32) -> Self {
        Self::new_use_simple_sum(false, rug_precision)
    }

    pub fn new_use_simple_sum(use_simple_sum: bool, rug_precision: u32) -> Self {
        Self {
            error_rate: rug::Float::with_val(rug_precision, 0.),
            use_simple_sum: use_simple_sum,
            rug_precision: rug_precision,
        }
    }

    pub fn accumulate(&mut self, p: rug::Float) -> &mut Self {
        if self.use_simple_sum {
            self.error_rate = self.error_rate.clone() + p;
        } else {
            self.error_rate = self.error_rate.clone() * (1. - p.clone()) + p * (1. - self.error_rate.clone());
        }
        self
    }

    pub fn accumulate_multiple(&mut self, n: rug::Integer, p: rug::Float) -> &mut Self {
        if self.use_simple_sum {
            self.error_rate = self.error_rate.clone() + rug::Float::with_val(self.rug_precision, n) * p.clone();
        } else {
            // need to optimize when doing the sum, beucase n is generally 2^d and we want a lower complexity with d
            let mut acc_p = rug::Float::with_val(self.rug_precision, 0.);
            let mut p_i = p.clone();
            let mut n = n;
            loop {
                if n == 0 {  // stop when n == 0
                    break
                }
                if n.get_bit(0) {
                    acc_p = acc_p.clone() * (1. - p_i.clone()) + p_i.clone() * (1. - acc_p.clone());
                }
                p_i = 2. * p_i.clone() * (1. - p_i.clone());
                n = n.clone() >> 1;
            }
            self.error_rate = self.error_rate.clone() * (1. - acc_p.clone()) + acc_p.clone() * (1. - self.error_rate.clone());
        }
        self
    }
}

fn binomial(n: usize, k: usize) -> rug::Integer {
    rug::Integer::binomial_u(n as u32, k as u32).complete()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::rand::prelude::*;

    // use `cargo test fast_benchmark_1 -- --nocapture` to run specific test

    #[test]
    fn fast_benchmark_1() {
        let d = 5;
        let p = 0.01;
        let pe = 0.02;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(d, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(p/3., p/3., p/3., pe);
        let mut fast_benchmark = model.build_graph_fast_benchmark(ftqec::weight_autotune, true).unwrap();
        fast_benchmark.assignment_sampling_amount = 3;
        fast_benchmark.prepare();
        // fast_benchmark.debug_print();
        model.optimize_correction_pattern();
        model.build_exhausted_path();
        // run benchmark
        let mut rng = thread_rng();
        fast_benchmark.benchmark_once(&mut rng, fake_decoding);
    }

    #[test]
    fn fast_benchmark_2() {
        let d = 5;
        let p = 0.002;
        let pe = 0.0;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code(d, d);
        model.set_individual_error_with_perfect_initialization_with_erasure(p/3., p/3., p/3., pe);
        let mut fast_benchmark = model.build_graph_fast_benchmark(ftqec::weight_autotune, true).unwrap();
        fast_benchmark.assignment_sampling_amount = 10;
        fast_benchmark.prepare();
        // fast_benchmark.debug_print();
        model.optimize_correction_pattern();
        model.build_exhausted_path();
        // run benchmark
        let mut rng = thread_rng();
        for _ in 0..100 {
            fast_benchmark.benchmark_once(&mut rng, fake_decoding);
        }
        println!("estimated logical error rate: {}", fast_benchmark.logical_error_rate());
    }

    #[test]
    fn fast_benchmark_error_rate_accumulator() {
        // let mut accumulator = ErrorRateAccumulator::new();
        let rug_precision = 128;
        println!("{}", ErrorRateAccumulator::new(rug_precision).accumulate_multiple(
            rug::Integer::from(100), rug::Float::with_val(rug_precision, 0.00001)).error_rate);

        println!("{}", ErrorRateAccumulator::new(rug_precision).accumulate_multiple(
            rug::Integer::from(100), rug::Float::with_val(rug_precision, 0.001)).error_rate);
        println!("{}", ErrorRateAccumulator::new_use_simple_sum(true, rug_precision).accumulate_multiple(
            rug::Integer::from(100), rug::Float::with_val(rug_precision, 0.001)).error_rate);
        
        println!("{}", ErrorRateAccumulator::new(rug_precision).accumulate_multiple(
            rug::Integer::from(100), rug::Float::with_val(rug_precision, 0.01)).error_rate);
        println!("{}", ErrorRateAccumulator::new_use_simple_sum(true, rug_precision).accumulate_multiple(
            rug::Integer::from(100), rug::Float::with_val(rug_precision, 0.01)).error_rate);
    }

    #[test]
    fn fast_benchmark_high_precision_float() {
        let mut a = rug::Float::with_val(53, 10.5);
        for _ in 0..10000 {
            a = a * 1e-40;
        }
        println!("{:.16e}", a);
        // test small sum
        for &precision in [50u32, 99, 150, 200, 1000, 2000].iter() {  // precision in bits
            let b = rug::Float::with_val(precision, 10.5);
            let c = rug::Float::with_val(precision, 1e-100);
            let mut d = b.clone() + c;
            d -= b;
            println!("{}: {:.16e}", precision, d);
        }
        // test precision sum
        let a = rug::Float::with_val(4, 3.1415926);
        let b = rug::Float::with_val(12, 3.1415926);
        println!("a: {}", a);
        println!("b: {}", b);
        println!("a + b: {}", a.clone() + b.clone());
        println!("b + a: {}", b.clone() + a.clone());
    }

}
