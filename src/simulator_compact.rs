//! A compact simulator that tracks all the error sources globally and cache the defect measurements it generates.
//!

use super::either::Either;
use super::noise_model::*;
use super::reproducible_rand::Xoroshiro128StarStar;
use super::simulator::*;
use super::types::*;
use super::util_macros::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use ErrorType::*;

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulatorCompact {
    /// each error source is an independent probabilistic Pauli or erasure error
    pub error_sources: Vec<ErrorSource>,
    /// use embedded random number generator
    #[serde(skip)]
    pub rng: Xoroshiro128StarStar,
    /// the actual happening errors
    #[serde(skip)]
    errors: BTreeMap<Position, ErrorType>,
    /// the desired correction of the actual error
    #[serde(skip)]
    corrections: BTreeMap<Position, ErrorType>,
    /// the measured defects
    #[serde(skip)]
    defects: BTreeSet<Position>,
    /// optional simulator for the purpose of validate the correction
    #[serde(skip)]
    simulator: Option<Simulator>,
}

impl Clone for SimulatorCompact {
    fn clone(&self) -> Self {
        Self {
            error_sources: self.error_sources.clone(),
            rng: Xoroshiro128StarStar::new(),
            errors: BTreeMap::new(),
            corrections: BTreeMap::new(),
            defects: BTreeSet::new(),
            simulator: self.simulator.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSource {
    Pauli {
        p: f64,
        defects: Vec<Position>,
        errors: Vec<(Position, ErrorType)>,
        correction: Vec<(Position, ErrorType)>,
    },
}

impl ErrorSource {
    pub fn get_error_t(&self) -> usize {
        match self {
            Self::Pauli { errors, .. } => {
                let t = errors[0].0.t;
                debug_assert!(
                    errors.iter().all(|(position, _)| position.t == t),
                    "an error source cannot happen at multiple time point..."
                );
                t
            }
        }
    }
    pub fn get_correction_t(&self) -> usize {
        match self {
            Self::Pauli { correction, .. } => {
                let t = correction[0].0.t;
                debug_assert!(
                    correction.iter().all(|(position, _)| position.t == t),
                    "an correction cannot happen at multiple time point..."
                );
                t
            }
        }
    }
    pub fn shift_error_t(&mut self, delta_t: usize) {
        match self {
            Self::Pauli {
                defects, errors, ..
            } => {
                for position in defects.iter_mut() {
                    position.t += delta_t;
                }
                for (position, _) in errors.iter_mut() {
                    position.t += delta_t;
                }
            }
        }
    }
    pub fn shift_correction_t(&mut self, delta_height: usize) {
        match self {
            Self::Pauli { correction, .. } => {
                for (position, _) in correction.iter_mut() {
                    position.t += delta_height;
                }
            }
        }
    }
}

#[cfg(feature = "python_binding")]
bind_trait_simulator_generics! {SimulatorCompact}

impl SimulatorGenerics for SimulatorCompact {
    fn generate_random_errors(&mut self, _noise_model: &NoiseModel) -> (usize, usize) {
        self.clear();
        let mut rng = self.rng.clone(); // avoid mutable borrow
        let mut error_count = 0;
        for error_source in self.error_sources.iter() {
            match error_source {
                ErrorSource::Pauli {
                    p,
                    errors,
                    defects,
                    correction,
                } => {
                    let random_value = rng.next_f64();
                    if random_value < *p {
                        // apply error
                        for (position, error) in errors.iter() {
                            if let Some(existing_error) = self.errors.get_mut(position) {
                                if *existing_error != I {
                                    error_count -= 1;
                                }
                                *existing_error = existing_error.multiply(error);
                                if *existing_error != I {
                                    error_count += 1;
                                }
                            } else {
                                self.errors.insert(position.clone(), *error);
                                if *error != I {
                                    error_count += 1;
                                }
                            }
                        }
                        // apply perfect correction
                        for (position, correct_pauli) in correction.iter() {
                            if let Some(existing_correct_pauli) = self.corrections.get_mut(position)
                            {
                                *existing_correct_pauli =
                                    existing_correct_pauli.multiply(correct_pauli);
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
        self.rng = rng; // save the random number generator
        (error_count, 0) // doesn't support erasure errors yet
    }
    fn generate_sparse_detected_erasures(&self) -> SparseErasures {
        SparseErasures::new() // doesn't support erasure errors yet
    }
    fn generate_sparse_error_pattern(&self) -> SparseErrorPattern {
        SparseErrorPattern::new_map(self.errors.clone())
    }
    fn generate_sparse_measurement(&self) -> SparseMeasurement {
        SparseMeasurement::new_set(self.defects.clone())
    }
    fn validate_correction(&mut self, correction: &SparseCorrection) -> (bool, bool) {
        assert!(
            self.simulator.is_some(),
            "a simulator must be provided to validate a correction"
        );
        let simulator = self.simulator.as_mut().unwrap();
        let top_t = simulator.height - 1;
        simulator_iter_mut_real!(simulator, position, node, t => top_t, {  // only clear propagated errors on top later
            node.propagated = I;
        });
        // set the desired correction, which is the result of the final propagated errors
        for (position, correct_pauli) in self.corrections.iter() {
            let mut position = position.clone();
            position.t = top_t; // shift down
            let node: &mut SimulatorNode = simulator.get_node_mut_unwrap(&position);
            node.propagated = node.propagated.multiply(correct_pauli);
        }
        let mut shifted_correction = SparseCorrection::new();
        for (position, correct_pauli) in correction.iter() {
            let mut position = position.clone();
            position.t = top_t; // shift down
            shifted_correction.add(position, *correct_pauli);
        }
        simulator.validate_correction(&shifted_correction)
    }
}

impl SimulatorCompact {
    pub fn from_simulator(
        mut simulator: Simulator,
        noise_model: Arc<NoiseModel>,
        parallel: usize,
    ) -> Self {
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
                let t_start = parallel_idx * interval; // included
                let mut t_end = (parallel_idx + 1) * interval; // excluded
                if parallel_idx == parallel - 1 {
                    t_end = simulator.height; // to make sure every part is included
                }
                let noise_model = Arc::clone(&noise_model);
                handlers.push(std::thread::spawn(move || {
                    let mut instance = instance.lock().unwrap();
                    instance.build_error_sources_region(
                        &mut simulator,
                        noise_model,
                        t_start,
                        t_end,
                    );
                }));
            }
            for handler in handlers.drain(..) {
                handler.join().unwrap();
            }
            // move the data from instances (without additional large memory allocation)
            for parallel_idx in 0..parallel {
                let mut instance = instances[parallel_idx].lock().unwrap();
                simulator_compact
                    .error_sources
                    .append(&mut instance.error_sources);
            }
        }
        simulator_compact.simulator = Some(simulator);
        simulator_compact
    }

    fn build_error_sources_region(
        &mut self,
        simulator: &mut Simulator,
        noise_model: Arc<NoiseModel>,
        t_start: usize,
        t_end: usize,
    ) {
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
                continue;
            }
            let noise_model_node = noise_model.get_node_unwrap(position);
            // whether it's possible to have erasure error at this node
            let possible_erasure_error = noise_model_node.erasure_error_rate > 0.
                || noise_model_node.correlated_erasure_error_rates.is_some()
                || {
                    let node = simulator.get_node_unwrap(position);
                    if let Some(gate_peer) = node.gate_peer.as_ref() {
                        let peer_noise_model_node = noise_model.get_node_unwrap(gate_peer);
                        if let Some(correlated_erasure_error_rates) =
                            &peer_noise_model_node.correlated_erasure_error_rates
                        {
                            correlated_erasure_error_rates.error_probability() > 0.
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };
            assert!(!possible_erasure_error, "not implemented");
            for error in all_possible_errors.iter() {
                let p = match error {
                    Either::Left(error_type) => {
                        noise_model_node.pauli_error_rates.error_rate(error_type)
                    }
                    Either::Right(error_type) => {
                        match &noise_model_node.correlated_pauli_error_rates {
                            Some(correlated_pauli_error_rates) => {
                                correlated_pauli_error_rates.error_rate(error_type)
                            }
                            None => 0.,
                        }
                    }
                }; // probability of this error to occur
                if p > 0. {
                    // simulate the error and measure it
                    let mut sparse_errors = SparseErrorPattern::new();
                    match error {
                        Either::Left(error_type) => {
                            sparse_errors.add(position.clone(), error_type.clone());
                        }
                        Either::Right(error_type) => {
                            sparse_errors.add(position.clone(), error_type.my_error());
                            let node = simulator.get_node_unwrap(position);
                            let gate_peer = node
                                .gate_peer
                                .as_ref()
                                .expect("correlated error must corresponds to a two-qubit gate");
                            sparse_errors.add((**gate_peer).clone(), error_type.peer_error());
                        }
                    }
                    let sparse_errors = Arc::new(sparse_errors); // make it immutable and shared
                    let (sparse_correction, sparse_measurement_real, _sparse_measurement_virtual) =
                        simulator.fast_measurement_given_few_errors(&sparse_errors);
                    let sparse_measurement_real = sparse_measurement_real.to_vec();
                    if sparse_measurement_real.len() == 0 {
                        // no way to detect it, ignore
                        continue;
                    }
                    self.error_sources.push(ErrorSource::Pauli {
                        p,
                        defects: sparse_measurement_real,
                        correction: sparse_correction.to_vec(),
                        errors: sparse_errors.to_vec(),
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

    pub fn assert_eq(&self, other: &Self) -> Result<(), String> {
        if self.error_sources.len() != other.error_sources.len() {
            return Err(format!(
                "the length differs {} != {}",
                self.error_sources.len(),
                other.error_sources.len()
            ));
        }
        for index in 0..self.error_sources.len() {
            if self.error_sources[index] != other.error_sources[index] {
                return Err(format!(
                    "the {}-th error source differs: {:?} != {:?}",
                    index, self.error_sources[index], other.error_sources[index]
                ));
            }
        }
        debug_assert_eq!(self, other, "up to this step, they should be equal");
        Ok(())
    }
}

impl PartialEq for SimulatorCompact {
    fn eq(&self, other: &Self) -> bool {
        self.error_sources == other.error_sources
    }
}

/// this is a compressed version of compact simulator, by not expanding all the layers and only dynamically generate the layers
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulatorCompactCompressed {
    /// the extender, without ever expanding it
    pub extender: SimulatorCompactExtender,
    /// the real noisy measurement
    pub noisy_measurements: usize,
}

impl Clone for SimulatorCompactCompressed {
    fn clone(&self) -> Self {
        Self {
            extender: self.extender.clone(),
            noisy_measurements: self.noisy_measurements,
        }
    }
}

impl SimulatorCompactCompressed {
    pub fn new(extender: SimulatorCompactExtender, noisy_measurements: usize) -> Self {
        Self {
            extender,
            noisy_measurements,
        }
    }

    pub fn clear(&mut self) {
        self.extender.base.clear();
    }
}

#[cfg(feature = "python_binding")]
bind_trait_simulator_generics! {SimulatorCompactCompressed}

impl SimulatorGenerics for SimulatorCompactCompressed {
    fn generate_random_errors(&mut self, _noise_model: &NoiseModel) -> (usize, usize) {
        self.clear();
        let mut rng = self.extender.base.rng.clone(); // avoid mutable borrow
        let mut error_count = 0;
        let mut base_errors = self.extender.base.errors.clone();
        let mut base_corrections: BTreeMap<Position, ErrorType> =
            self.extender.base.corrections.clone();
        let mut base_defects = self.extender.base.defects.clone();
        for (error_source, delta_t) in self.extender.iter(self.noisy_measurements) {
            match &error_source {
                ErrorSource::Pauli {
                    p,
                    errors,
                    defects,
                    correction,
                } => {
                    let random_value = rng.next_f64();
                    if random_value < *p {
                        // apply error
                        for (position, error) in errors.iter() {
                            let mut position = position.clone();
                            position.t += delta_t;
                            if let Some(existing_error) = base_errors.get_mut(&position) {
                                if *existing_error != I {
                                    error_count -= 1;
                                }
                                *existing_error = existing_error.multiply(error);
                                if *existing_error != I {
                                    error_count += 1;
                                }
                            } else {
                                base_errors.insert(position.clone(), *error);
                                if *error != I {
                                    error_count += 1;
                                }
                            }
                        }
                        // apply perfect correction
                        for (position, correct_pauli) in correction.iter() {
                            if let Some(existing_correct_pauli) = base_corrections.get_mut(position)
                            {
                                *existing_correct_pauli =
                                    existing_correct_pauli.multiply(correct_pauli);
                            } else {
                                base_corrections.insert(position.clone(), *correct_pauli);
                            }
                        }
                        // apply defect measurements
                        for position in defects.iter() {
                            let mut position = position.clone();
                            position.t += delta_t;
                            if base_defects.contains(&position) {
                                base_defects.remove(&position);
                            } else {
                                base_defects.insert(position);
                            }
                        }
                    }
                }
            }
        }
        self.extender.base.errors = base_errors;
        self.extender.base.corrections = base_corrections;
        self.extender.base.defects = base_defects;
        self.extender.base.rng = rng; // save the random number generator
        (error_count, 0) // doesn't support erasure errors yet
    }
    fn generate_sparse_detected_erasures(&self) -> SparseErasures {
        self.extender.base.generate_sparse_detected_erasures()
    }
    fn generate_sparse_error_pattern(&self) -> SparseErrorPattern {
        self.extender.base.generate_sparse_error_pattern()
    }
    fn generate_sparse_measurement(&self) -> SparseMeasurement {
        self.extender.base.generate_sparse_measurement()
    }
    fn validate_correction(&mut self, correction: &SparseCorrection) -> (bool, bool) {
        self.extender.base.validate_correction(correction)
    }
}

/// The extender takes two `SimulatorCompact` as input, assuming the first one has T and the second one has T+1 noisy measurement rounds.
/// It works by finding an efficient representation that can generate a `SimulatorCompact` for arbitrarily large T.
/// Usually `noisy_measurements = 4` is good enough, e.g., when `measurement_cycle = 6`, the whole graph is 0<=t<=24, and the repeated region is 12<=t<=18.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulatorCompactExtender {
    pub base: SimulatorCompact,
    /// the number of noisy_measurements of the first
    pub noisy_measurements: usize,
    /// repeat region, indices of the repeat region
    pub repeat_region: (usize, usize),
    /// measurement cycle, `t` will be biased by repeat * measurement_cycle
    pub measurement_cycle: usize,
}

pub struct SimulatorCompactExtenderIter<'a> {
    extender: &'a SimulatorCompactExtender,
    /// target noisy measurement
    noisy_measurements: usize,
    /// the current index
    error_source_index: usize,
}

impl<'a> Iterator for SimulatorCompactExtenderIter<'a> {
    // to avoid memory allocation & deallocation if just iterating them.
    // profiling shows malloc and free takes 90% of the simulation time
    type Item = (&'a ErrorSource, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let error_source_index = self.error_source_index;
        self.error_source_index += 1;
        let extender = &self.extender;
        let base = &extender.base;
        if self.noisy_measurements == extender.noisy_measurements {
            if error_source_index < base.error_sources.len() {
                return Some((&base.error_sources[error_source_index], 0));
            } else {
                return None;
            }
        }
        let (repeat_start, repeat_end) = extender.repeat_region;
        let repeat: usize = self.noisy_measurements - extender.noisy_measurements;
        if error_source_index < repeat_end {
            return Some((&base.error_sources[error_source_index], 0));
        }
        if error_source_index >= repeat_end
            && error_source_index < repeat_end + repeat * (repeat_end - repeat_start)
        {
            let i = (error_source_index - repeat_start) / (repeat_end - repeat_start);
            debug_assert!(i >= 1 && i <= repeat);
            let index: usize =
                (error_source_index - repeat_start) % (repeat_end - repeat_start) + repeat_start;
            return Some((&base.error_sources[index], i * extender.measurement_cycle));
        }
        if error_source_index < base.error_sources.len() + repeat * (repeat_end - repeat_start) {
            let index = error_source_index - repeat * (repeat_end - repeat_start);
            return Some((
                &base.error_sources[index],
                repeat * extender.measurement_cycle,
            ));
        }
        None
    }
}

impl SimulatorCompactExtender {
    pub fn new(
        first: SimulatorCompact,
        second: SimulatorCompact,
        noisy_measurements: usize,
    ) -> Self {
        // find a rule
        // first check how many error sources differed by the two
        assert!(
            second.error_sources.len() > first.error_sources.len(),
            "must differ"
        );
        let error_sources_differ = second.error_sources.len() - first.error_sources.len();
        // then check whether the error sources are monotonic, this is essential for finding a rule
        #[derive(Debug)]
        struct RangeT {
            t: usize,
            start: usize, // index, include
            end: usize,   // index, exclude
        }
        let mut first_t_ranges: Vec<RangeT> = vec![];
        let mut second_t_ranges: Vec<RangeT> = vec![];
        for (t_ranges, simulator_compact) in [
            (&mut first_t_ranges, &first),
            (&mut second_t_ranges, &second),
        ] {
            let mut last_t = simulator_compact.error_sources[0].get_error_t();
            let mut start = 0;
            for (index, error_source) in simulator_compact.error_sources.iter().enumerate() {
                let t = error_source.get_error_t();
                assert!(t >= last_t, "t must be monotonically increasing");
                if t != last_t {
                    t_ranges.push(RangeT {
                        t,
                        start,
                        end: index,
                    });
                    start = index;
                    last_t = t;
                }
            }
            t_ranges.push(RangeT {
                t: last_t,
                start,
                end: simulator_compact.error_sources.len(),
            });
        }
        let measurement_cycle =
            second_t_ranges.last().unwrap().t - first_t_ranges.last().unwrap().t;
        assert!(measurement_cycle > 0);
        let repeat_t_start =
            (first_t_ranges.last().unwrap().t / measurement_cycle + 1) / 2 * measurement_cycle;
        let mut repeat_start = usize::MAX;
        let mut repeat_end = 0;
        for range in first_t_ranges.iter() {
            if range.t >= repeat_t_start && range.t < repeat_t_start + measurement_cycle {
                repeat_start = repeat_start.min(range.start);
                repeat_end = repeat_end.max(range.end);
            }
        }
        assert!(
            repeat_start != usize::MAX,
            "no error sources found in t range of [{}, {})",
            repeat_t_start,
            repeat_t_start + measurement_cycle
        );
        assert_eq!(
            repeat_end - repeat_start,
            error_sources_differ,
            "the repeat region should have the differed number of error sources"
        );
        let extender: SimulatorCompactExtender = Self {
            base: first,
            noisy_measurements,
            repeat_region: (repeat_start, repeat_end),
            measurement_cycle,
        };
        // use the second simulator to verify the correctness (partially)
        second
            .assert_eq(&extender.generate(noisy_measurements + 1))
            .unwrap();
        // return the verified extender
        extender
    }

    pub fn generate(&self, noisy_measurements: usize) -> SimulatorCompact {
        let mut simulator_compact = self.base.clone();
        simulator_compact.error_sources = self
            .iter(noisy_measurements)
            .map(|(error_source, delta_t)| {
                let mut error_source = error_source.clone();
                error_source.shift_error_t(delta_t);
                error_source.shift_correction_t(
                    (noisy_measurements - self.noisy_measurements) * self.measurement_cycle,
                );
                error_source
            })
            .collect();
        simulator_compact
    }

    pub fn iter(&self, noisy_measurements: usize) -> SimulatorCompactExtenderIter {
        assert!(noisy_measurements >= self.noisy_measurements);
        SimulatorCompactExtenderIter {
            extender: self,
            noisy_measurements,
            error_source_index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_builder::*;
    use crate::noise_model_builder::*;

    #[test]
    fn simulator_compact_extender() {
        // cargo test simulator_compact_extender -- --nocapture
        let di = 3;
        let dj = 3;
        let p = 0.001;
        let build_simulator =
            |noisy_measurements: usize| -> (Simulator, NoiseModel, SimulatorCompact) {
                let mut simulator = Simulator::new(
                    CodeType::RotatedPlanarCode,
                    CodeSize::new(noisy_measurements, di, dj),
                );
                let mut noise_model = NoiseModel::new(&simulator);
                NoiseModelBuilder::StimNoiseModel.apply(
                    &mut simulator,
                    &mut noise_model,
                    &json!({}),
                    p,
                    0.5,
                    0.,
                );
                code_builder_sanity_check(&simulator).unwrap();
                noise_model_sanity_check(&simulator, &noise_model).unwrap();
                let simulator_compact = SimulatorCompact::from_simulator(
                    simulator.clone(),
                    Arc::new(noise_model.clone()),
                    1,
                );
                (simulator, noise_model, simulator_compact)
            };
        let noisy_measurements = 4;
        let (_, _, first) = build_simulator(noisy_measurements);
        let (_, _, second) = build_simulator(noisy_measurements + 1);
        let extender = SimulatorCompactExtender::new(first, second, noisy_measurements);
        println!("extender built successfully");
        // test a larger instance
        let test_noisy_measurement = 7;
        let generated = extender.generate(test_noisy_measurement);
        let (_, _, ground_truth) = build_simulator(test_noisy_measurement);
        generated.assert_eq(&ground_truth).unwrap();
    }
}
