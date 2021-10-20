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
//! We not only consider a single error chain connecting the two opposite boundaries, but also randomize errors at other positions so that it's more accurate especially when the decoder far from optimal.
//! For example, union-find decoder has exactly the same decoding accuracy with MWPM decoder when there is only a single error chain, and the decoding accuracy only shows up when we have 2D or 3D randomized errors.
//! This randomizing sub-routine opens opportunities to reveal the difference between decoders, together with the generality beyond simple coefficient fitting.
//!


#![allow(non_snake_case)]
#![allow(dead_code)]

// use super::ftqec;

pub struct PossiblePath {

}

pub struct FBNode {
    pub t: usize,
    pub i: usize,
    pub j: usize,
    // connection information
    // pub connections: Ve
    // internal static information
    pub hop_right: usize,  // how many hops to left boundary
    pub hop_back: usize,  // how many hops to back boundary

}

pub struct FastBenchmark {
    
}
