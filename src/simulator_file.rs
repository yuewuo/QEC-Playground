//! Simulator that reads from file
//!

use super::noise_model::*;
use super::reproducible_rand::Xoroshiro128StarStar;
use super::simulator::*;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulatorVec {
    /// the error patterns are periodically constructed
    pub error_patterns: Vec<SparseErrorPattern>,
    /// index
    pub index: usize,
    /// original simulator
    #[serde(skip)]
    simulator: Option<Simulator>,
}

impl Clone for SimulatorVec {
    fn clone(&self) -> Self {
        Self {
            error_patterns: self.error_patterns.clone(),
            index: self.index,
            simulator: self.simulator.clone(),
        }
    }
}

#[cfg(feature = "python_binding")]
bind_trait_simulator_generics! {SimulatorVec}

impl SimulatorGenerics for SimulatorVec {
    fn set_rng(&mut self, _rng: Xoroshiro128StarStar) {}

    fn generate_random_errors(&mut self, noise_model: &NoiseModel) -> (usize, usize) {
        self.index = (self.index + 1) % self.error_patterns.len();
        let sparse_error_pattern = self.generate_sparse_error_pattern();
        let simulator = self.simulator.as_mut().unwrap();
        simulator
            .load_sparse_error_pattern(&sparse_error_pattern, noise_model)
            .unwrap();
        simulator.propagate_errors();
        (sparse_error_pattern.len(), 0)
    }
    fn generate_sparse_detected_erasures(&self) -> SparseErasures {
        SparseErasures::new() // doesn't support erasure errors yet
    }
    fn generate_sparse_error_pattern(&self) -> SparseErrorPattern {
        self.error_patterns[self.index].clone()
    }
    fn generate_sparse_measurement(&self) -> SparseMeasurement {
        self.simulator.as_ref().unwrap().generate_sparse_measurement()
    }
    fn validate_correction(&mut self, correction: &SparseCorrection) -> (bool, bool) {
        self.simulator.as_mut().unwrap().validate_correction(correction)
    }
}

impl SimulatorVec {
    pub fn from_simulator(simulator: Simulator, error_patterns: Vec<SparseErrorPattern>) -> Self {
        assert!(!error_patterns.is_empty());
        Self {
            error_patterns,
            index: 0,
            simulator: Some(simulator),
        }
    }
}
