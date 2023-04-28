//! A compact simulator that tracks all the error sources globally and cache the defect measurements it generates.
//! 

use super::simulator::*;
use super::util_macros::*;
use std::collections::{BTreeMap, BTreeSet};
use super::either::Either;
use super::types::*;
use super::noise_model::*;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use ErrorType::*;
use super::reproducible_rand::Xoroshiro128StarStar;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorCompact {
    /// each error source is an independent probabilistic Pauli or erasure error
    pub error_sources: Vec<ErrorSource>,
    /// use embedded random number generator
    pub rng: Xoroshiro128StarStar,
    /// the actual happening errors
    errors: BTreeMap<Position, ErrorType>,
    /// the desired correction of the actual error
    corrections: BTreeMap<Position, ErrorType>,
    /// the measured defects
    defects: BTreeSet<Position>,
    /// optional simulator for the purpose of validate the correction
    #[serde(skip)]
    simulator: Option<Simulator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSource {
    Pauli {
        p: f64,
        defects: Vec<Position>,
        errors: Vec<(Position, ErrorType)>,
        correction: Vec<(Position, ErrorType)>,
    },
}

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pymethods)]
impl SimulatorGenerics for SimulatorCompact {
    fn generate_random_errors(&mut self, _noise_model: &NoiseModel) -> (usize, usize) {
        self.clear();
        let mut rng = self.rng.clone();  // avoid mutable borrow
        let mut error_count = 0;
        for error_source in self.error_sources.iter() {
            match error_source {
                ErrorSource::Pauli { p, errors, defects, correction } => {
                    let random_value = rng.next_f64();
                    if random_value < *p {
                        // apply error
                        for (position, error) in errors.iter() {
                            if let Some(existing_error) = self.errors.get_mut(position) {
                                if *existing_error != I { error_count -= 1; }
                                *existing_error = existing_error.multiply(error);
                                if *existing_error != I { error_count += 1; }
                            } else {
                                self.errors.insert(position.clone(), *error);
                                if *error != I { error_count += 1; }
                            }
                        }
                        // apply perfect correction
                        for (position, correct_pauli) in correction.iter() {
                            if let Some(existing_correct_pauli) = self.corrections.get_mut(position) {
                                *existing_correct_pauli = existing_correct_pauli.multiply(correct_pauli);
                            } else {
                                self.corrections.insert(position.clone(), *correct_pauli);
                            }
                        }
                        // apply defect measurements
                        for position in defects.iter() {
                            if self.defects.contains(position) {
                                self.defects.remove(position);
                            } else {
                                self.defects.insert(position.clone());
                            }
                        }
                    }
                }
            }
        }
        self.rng = rng;  // save the random number generator
        (error_count, 0)  // doesn't support erasure errors yet
    }
    fn generate_sparse_detected_erasures(&self) -> SparseErasures {
        SparseErasures::new()  // doesn't support erasure errors yet
    }
    fn generate_sparse_error_pattern(&self) -> SparseErrorPattern {
        SparseErrorPattern::new_map(self.errors.clone())
    }
    fn generate_sparse_measurement(&self) -> SparseMeasurement {
        SparseMeasurement::new_set(self.defects.clone())
    }
    fn validate_correction(&mut self, correction: &SparseCorrection) -> (bool, bool) {
        assert!(self.simulator.is_some(), "a simulator must be provided to validate a correction");
        let simulator = self.simulator.as_mut().unwrap();
        let top_t = simulator.height - 1;
        simulator_iter_mut_real!(simulator, position, node, t => top_t, {  // only clear propagated errors on top later
            node.propagated = I;
        });
        // set the desired correction, which is the result of the final propagated errors
        for (position, correct_pauli) in self.corrections.iter() {
            assert_eq!(position.t, top_t, "correction pattern must only be at top layer");
            let node = simulator.get_node_mut_unwrap(position);
            node.propagated = node.propagated.multiply(correct_pauli);
        }
        simulator.validate_correction(correction)
    }
}

impl SimulatorCompact {
    pub fn from_simulator(mut simulator: Simulator, noise_model: Arc<NoiseModel>, parallel: usize) -> Self {
        let mut simulator_compact = Self {
            error_sources: vec![],
            rng: Xoroshiro128StarStar::new(),
            errors: BTreeMap::new(),
            corrections: BTreeMap::new(),
            defects: BTreeSet::new(),
            simulator: None,
        };
        if parallel <= 1 {
            let height = simulator.height;
            simulator_compact.build_error_sources_region(&mut simulator, noise_model, 0, height);
        } else {
            let mut handlers = Vec::new();
            let mut instances = Vec::new();
            let interval = simulator.height / parallel;
            for parallel_idx in 0..parallel {
                let instance = Arc::new(Mutex::new(simulator_compact.clone()));
                let mut simulator = simulator.clone();
                instances.push(Arc::clone(&instance));
                let t_start = parallel_idx * interval;  // included
                let mut t_end = (parallel_idx + 1) * interval;  // excluded
                if parallel_idx == parallel - 1 {
                    t_end = simulator.height;  // to make sure every part is included
                }
                let noise_model = Arc::clone(&noise_model);
                handlers.push(std::thread::spawn(move || {
                    let mut instance = instance.lock().unwrap();
                    instance.build_error_sources_region(&mut simulator, noise_model, t_start, t_end);
                }));
            }
            for handler in handlers.drain(..) {
                handler.join().unwrap();
            }
            // move the data from instances (without additional large memory allocation)
            for parallel_idx in 0..parallel {
                let mut instance = instances[parallel_idx].lock().unwrap();
                simulator_compact.error_sources.append(&mut instance.error_sources);
            }
        }
        simulator_compact.simulator = Some(simulator);
        simulator_compact
    }

    fn build_error_sources_region(&mut self, simulator: &mut Simulator, noise_model: Arc<NoiseModel>, t_start: usize, t_end: usize) {
        // calculate all possible errors to be iterated
        let mut all_possible_errors: Vec<Either<ErrorType, CorrelatedPauliErrorType>> = Vec::new();
        for error_type in ErrorType::all_possible_errors().drain(..) {
            all_possible_errors.push(Either::Left(error_type));
        }
        for correlated_error_type in CorrelatedPauliErrorType::all_possible_errors().drain(..) {
            all_possible_errors.push(Either::Right(correlated_error_type));
        }
        // clear the states in simulator including pauli, erasure errors and propagated errors
        simulator.clear_all_errors();
        // iterate over all possible errors at all possible positions
        simulator_iter!(simulator, position, {
            if position.t < t_start || position.t >= t_end {
                continue
            }
            let noise_model_node = noise_model.get_node_unwrap(position);
            // whether it's possible to have erasure error at this node
            let possible_erasure_error = noise_model_node.erasure_error_rate > 0. || noise_model_node.correlated_erasure_error_rates.is_some() || {
                let node = simulator.get_node_unwrap(position);
                if let Some(gate_peer) = node.gate_peer.as_ref() {
                    let peer_noise_model_node = noise_model.get_node_unwrap(gate_peer);
                    if let Some(correlated_erasure_error_rates) = &peer_noise_model_node.correlated_erasure_error_rates {
                        correlated_erasure_error_rates.error_probability() > 0.
                    } else { false }
                } else { false }
            };
            assert!(!possible_erasure_error, "not implemented");
            for error in all_possible_errors.iter() {
                let p = match error {
                    Either::Left(error_type) => {
                        noise_model_node.pauli_error_rates.error_rate(error_type)
                    },
                    Either::Right(error_type) => {
                        match &noise_model_node.correlated_pauli_error_rates {
                            Some(correlated_pauli_error_rates) => {
                                correlated_pauli_error_rates.error_rate(error_type)
                            },
                            None => 0.,
                        }
                    },
                }; // probability of this error to occur
                if p > 0. {
                    // simulate the error and measure it
                    let mut sparse_errors = SparseErrorPattern::new();
                    match error {
                        Either::Left(error_type) => {
                            sparse_errors.add(position.clone(), error_type.clone());
                        },
                        Either::Right(error_type) => {
                            sparse_errors.add(position.clone(), error_type.my_error());
                            let node = simulator.get_node_unwrap(position);
                            let gate_peer = node.gate_peer.as_ref().expect("correlated error must corresponds to a two-qubit gate");
                            sparse_errors.add((**gate_peer).clone(), error_type.peer_error());
                        },
                    }
                    let sparse_errors = Arc::new(sparse_errors);  // make it immutable and shared
                    let (sparse_correction, sparse_measurement_real, _sparse_measurement_virtual) = simulator.fast_measurement_given_few_errors(&sparse_errors);
                    let sparse_measurement_real = sparse_measurement_real.to_vec();
                    if sparse_measurement_real.len() == 0 {  // no way to detect it, ignore
                        continue
                    }
                    self.error_sources.push(ErrorSource::Pauli {
                        p, defects: sparse_measurement_real, correction: sparse_correction.to_vec(), errors: sparse_errors.to_vec(),
                    })
                }
            }
        });
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.corrections.clear();
        self.defects.clear();
    }

}
