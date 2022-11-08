//! General purpose Pauli group simulator optimized for surface code
//! 
#[cfg(feature="python_binding")]
use pyo3::prelude::*;
use std::cmp::Ordering;
use super::types::*;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Visitor, MapAccess};
use serde::ser::SerializeMap;
use super::code_builder::*;
use super::util_macros::*;
use super::reproducible_rand::Xoroshiro128StarStar;
use super::error_model::*;
use ErrorType::*;
use std::sync::Arc;
use std::collections::{HashMap, HashSet, BTreeSet, BTreeMap};
use super::serde_hashkey;
use super::erasure_graph::*;


/// general simulator for two-dimensional code with circuit-level implementation of stabilizer measurements
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct Simulator {
    /// information of the preferred code
    // #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub code_type: CodeType,
    /// the information fields of CodeType
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub builtin_code_information: BuiltinCodeInformation,
    /// size of the snapshot, where `nodes` is ensured to be a cube of `height` * `vertical` * `horizontal`
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub height: usize,
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub vertical: usize,
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub horizontal: usize,
    /// nodes array, because some rotated code can easily have more than half of the nodes non-existing, existing nodes are stored on heap
    pub nodes: Vec::< Vec::< Vec::< Option<Box <SimulatorNode> > > > >,
    /// use embedded random number generator
    pub rng: Xoroshiro128StarStar,
    /// how many cycles is there a round of measurements; default to 1
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub measurement_cycles: usize,
}

/// when plotting, t is the time axis; looking at the direction of `t=-∞`, the top-left corner is `i=j=0`;
/// `i` is vertical position, which increases when moving from top to bottom;
/// `j` is horizontal position, which increases when moving from left to right
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct Position {
    // pub index: [usize; 3],
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub t: usize,
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub i: usize,
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub j: usize,
}

/// each node represents a location `[i][j]` at a specific time point `[t]`, this location has some probability of having Pauli error or erasure error.
/// we could have single-qubit or two-qubit gate in a node, and errors are added **after applying this gate** (e.g. if the gate is measurement, then 
/// errors at this node will have no impact on the measurement because errors are applied after the measurement).
/// we also maintain "virtual nodes" at the boundary of a code, these virtual nodes are missing stabilizers at the boundary of a open-boundary surface code.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct SimulatorNode {
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub qubit_type: QubitType,
    /// single-qubit or two-qubit gate applied 
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub gate_type: GateType,
    pub gate_peer: Option<Arc<Position>>,
    /// simulation data
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub error: ErrorType,
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub has_erasure: bool,
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub propagated: ErrorType,
    /// Virtual qubit doesn't physically exist, which means they will never have errors themselves.
    /// Real qubit errors can propagate to virtual qubits, but errors will never propagate to real qubits.
    /// Virtual qubits can be understood as perfect stabilizers that only absorb propagated errors and never propagate them.
    /// They're useful in tailored surface code decoding, and also to represent virtual boundaries
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub is_virtual: bool,
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub is_peer_virtual: bool,
    /// miscellaneous information, should be static, e.g. decoding assistance information
    pub miscellaneous: Option<Arc<serde_json::Value>>,
}

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pymethods)]
impl SimulatorNode {
    /// create a new simulator node
    #[cfg_attr(feature = "python_binding", new)]
    pub fn new(qubit_type: QubitType, gate_type: GateType, gate_peer: Option<Position>) -> Self {
        Self {
            qubit_type: qubit_type,
            gate_type: gate_type,
            gate_peer: gate_peer.map(Arc::new),
            error: I,
            has_erasure: false,
            propagated: I,
            is_virtual: false,
            is_peer_virtual: false,
            miscellaneous: None,
        }
    }
    #[cfg_attr(feature="python_binding", setter)]
    pub fn set_gate_peer(&mut self, pos: Position){
        self.gate_peer = Option::Some(pos).map(Arc::new);
    }
    #[cfg_attr(feature="python_binding", getter)]
    pub fn get_gate_peer(&self) -> Position{
       (**self.gate_peer.as_ref().unwrap()).clone()
    }
    /// set error with sanity check
    pub fn set_error_check(&mut self, _error_model: &ErrorModel, error: &ErrorType) {
        debug_assert!(!self.is_virtual || error == &I, "should not add errors at virtual nodes");
        // TODO: in debug build, check if this error is valid given the error rates
        self.error = *error;
    }
    pub fn set_error_temp(&mut self, error: &ErrorType){
        debug_assert!(!self.is_virtual || error == &I, "should not add errors at virtual nodes");
        // TODO: in debug build, check if this error is valid given the error rates
        self.error = *error;
    }
}

impl SimulatorNode{
    /// quick initialization function to set virtual bits (if there is any)
    pub fn set_virtual(mut self, is_virtual: bool, is_peer_virtual: bool) -> Self {
        self.is_virtual = is_virtual;
        self.is_peer_virtual = is_peer_virtual;
        self
    }

    /// quick initialization to set miscellaneous information
    pub fn with_miscellaneous(mut self, miscellaneous: Option<serde_json::Value>) -> Self {
        self.miscellaneous = miscellaneous.map(|x| Arc::new(x));
        self
    }
}

/// single-qubit and two-qubit gate type
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Copy)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub enum GateType {
    /// initialize in $|0\rangle$ state which is the eigenstate of $\hat{Z}$
    InitializeZ,
    /// initialize in $|+\rangle$ state which is the eigenstate of $\hat{X}$
    InitializeX,
    /// CX gate or CNOT gate, the control qubit
    CXGateControl,
    /// CX gate or CNOT gate, the target qubit
    CXGateTarget,
    /// CY gate or controlled-Y gate, the control qubit
    CYGateControl,
    /// CY gate or controlled-Y gate, the target qubit
    CYGateTarget,
    /// CZ gate or CPHASE gate, it's symmetric so no need to distinguish control and target
    CZGate,
    /// measurement in $\hat{Z}$ basis, only sensitive to $\hat{X}$ or $\hat{Y}$ errors
    MeasureZ,
    /// measurement in $\hat{X}$ basis, only sensitive to $\hat{Z}$ or $\hat{Y}$ errors
    MeasureX,
    /// no gate at this position, or idle. note that if the peer of virtual node, this position is also considered idle
    /// because the gate with virtual peer is non-existing physically.
    None,
}

#[cfg_attr(feature = "python_binding", pymethods)]
impl GateType {
    pub fn is_initialization(&self) -> bool {
        self == &GateType::InitializeZ || self == &GateType::InitializeX
    }
    pub fn is_measurement(&self) -> bool {
        self == &GateType::MeasureZ || self == &GateType::MeasureX
    }
    /// given a propagated error, check if stabilizer measurement output is +1 (true) or -1 (false)
    pub fn stabilizer_measurement(&self, propagated: &ErrorType) -> bool {
        match self {
            // not sensitive to Z
            GateType::MeasureZ => { if matches!(propagated, X | Y) { true } else { false } }
            // not sensitive to X
            GateType::MeasureX => { if matches!(propagated, Z | Y) { true } else { false } }
            _ => { panic!("stabilizer measurement behavior not specified") }
        }
    }
    /// single-qubit gate doesn't have peer, including idle gate
    pub fn is_single_qubit_gate(&self) -> bool {
        self.is_initialization() || self.is_measurement() || self == &GateType::None
    }
    /// two-qubit gate must have peer
    pub fn is_two_qubit_gate(&self) -> bool {
        !self.is_single_qubit_gate()
    }
    /// only two-qubit gate will propagate to peer
    pub fn propagate_peer(&self, propagated: &ErrorType) -> ErrorType {
        match self {
            // cx control not sensitive to Z, propagate as X
            GateType::CXGateControl => { if matches!(propagated, X | Y) { X } else { I } }
            // cx target not sensitive to X, propagate as Z
            GateType::CXGateTarget => { if matches!(propagated, Z | Y) { Z } else { I } }
            // cy control not sensitive to Z, propagate as Y
            GateType::CYGateControl => { if matches!(propagated, X | Y) { Y } else { I } }
            // cy target not sensitive to Y, propagate as Z
            GateType::CYGateTarget => { if matches!(propagated, Z | X) { Z } else { I } }
            // cz not sensitive to Z, propagate as Z
            GateType::CZGate => { if matches!(propagated, X | Y) { Z } else { I } }
            _ => { panic!("gate propagation behavior not specified") }
        }
    }
    /// check if a measurement gate is corresponding to the initialization
    pub fn is_corresponding_initialization(&self, other: &GateType) -> bool {
        if self == &GateType::MeasureX && other == &GateType::InitializeX { return true }
        if self == &GateType::MeasureZ && other == &GateType::InitializeZ { return true }
        false
    }
    /// the expected gate type of peer if this is a two-qubit gate, otherwise return `GateType::None`.
    /// for example, the peer gate type of a `GateType::CXGateControl` is `GateType::CXGateTarget`
    /// , because a [CXGate](https://qiskit.org/documentation/stubs/qiskit.circuit.library.CXGate.html)
    /// consists of a control and target.
    pub fn peer_gate(&self) -> GateType {
        match self {
            GateType::CXGateControl => GateType::CXGateTarget,
            GateType::CXGateTarget => GateType::CXGateControl,
            GateType::CYGateControl => GateType::CYGateTarget,
            GateType::CYGateTarget => GateType::CYGateControl,
            GateType::CZGate => GateType::CZGate,
            _ => GateType::None,
        }
    }
}

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pymethods)]
impl Simulator {
    /// given builtin code type, this will automatically build the code structure
    #[cfg_attr(feature = "python_binding", new)]
    pub fn new(code_type: CodeType, builtin_code_information: BuiltinCodeInformation) -> Self {
        let mut simulator = Self {
            code_type: code_type,
            builtin_code_information: builtin_code_information,
            height: 0,
            vertical: 0,
            horizontal: 0,
            nodes: Vec::new(),
            rng: Xoroshiro128StarStar::new(),
            measurement_cycles: 1,
        };
        build_code(&mut simulator);
        simulator
    }

    pub fn set_nodes(&mut self, position: Position, error: ErrorType){
        let node = self.get_node_mut_unwrap(&position);
        node.set_error_temp(&error);
    }

    pub fn clone(&self) -> Self {
        Self {
            code_type: self.code_type.clone(),
            builtin_code_information: self.builtin_code_information.clone(),
            height: self.height,
            vertical: self.vertical,
            horizontal: self.horizontal,
            nodes: self.nodes.clone(),
            rng: Xoroshiro128StarStar::new(),  // do not copy random number generator, otherwise parallel simulation may give same result
            measurement_cycles: self.measurement_cycles,
        }
    }

    pub fn volume(&self) -> usize {
        self.height * self.vertical * self.horizontal
    }

    /// judge if `[t][i][j]` is valid index of `self.nodes`
    #[inline]
    pub fn is_valid_position(&self, position: &Position) -> bool {
        position.t < self.height && position.i < self.vertical && position.j < self.horizontal
    }

    /// judge if `self.nodes[t][i][j]` is `Some(_)`
    #[inline]
    pub fn is_node_exist(&self, position: &Position) -> bool {
        self.is_valid_position(position) && self.get_node(position).is_some()
    }

    /// check if this node is a real node, i.e. physically exist in the simulation
    #[inline]
    pub fn is_node_real(&self, position: &Position) -> bool {
        self.is_node_exist(position) && self.get_node_unwrap(position).is_virtual == false
    }

    /// check if this node is a virtual node, i.e. non-existing but just work as a virtual boundary
    /// (they can be viewed as the missing stabilizers on the boundary)
    #[inline]
    pub fn is_node_virtual(&self, position: &Position) -> bool {
        self.is_node_exist(position) && self.get_node_unwrap(position).is_virtual == true
    }

    /// check if this node is a virtual node, i.e. non-existing but just work as a virtual boundary
    pub fn set_error_rates(&mut self, error_model: &mut ErrorModel, px: f64, py: f64, pz: f64, pe: f64) {
        assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
        assert!(pe <= 1. && pe >= 0.);
        if self.measurement_cycles == 1 {
            println!("[warning] setting error rates of unknown code, no perfect measurement protection is enabled");
        }
        let mut error_model_node = ErrorModelNode::new();
        error_model_node.pauli_error_rates.error_rate_X = px;
        error_model_node.pauli_error_rates.error_rate_Y = py;
        error_model_node.pauli_error_rates.error_rate_Z = pz;
        error_model_node.erasure_error_rate = pe;
        let error_model_node = Arc::new(error_model_node);
        for t in 0 .. self.height - self.measurement_cycles {
            simulator_iter_mut_real!(self, position, _node, t => t, {  // only add errors on real node
                error_model.set_node(position, Some(error_model_node.clone()));
            });
        }
    }

    /// expand the correlated error rates, useful when exporting the data structure for other applications to modify
    pub fn expand_error_rates(&mut self, error_model: &mut ErrorModel) {
        simulator_iter_mut!(self, position, _node, {
            let mut error_model_node: ErrorModelNode = error_model.get_node_unwrap(position).clone();
            if error_model_node.correlated_pauli_error_rates.is_none() {
                error_model_node.correlated_pauli_error_rates = Some(CorrelatedPauliErrorRates::default());
            }
            if error_model_node.correlated_erasure_error_rates.is_none() {
                error_model_node.correlated_erasure_error_rates = Some(CorrelatedErasureErrorRates::default());
            }
            error_model.set_node(position, Some(Arc::new(error_model_node)));
        });
    }

    /// compress error rates by trying to find same error rates and use the same pointer to reduce memory usage and improve memory coherence.
    /// note that when building error model with rust, one should directly use this optimization, so that this function execute fast.
    /// when taking error model data from other programs, this function will have to hash every one of them and it might take a while to do so.
    #[inline(never)]
    pub fn compress_error_rates(&mut self, error_model: &mut ErrorModel) {
        let mut arc_set: HashSet<*const ErrorModelNode> = HashSet::new();
        // since f64 typed error rates are not hashable by default, here I first serialize the them and then use OrderedFloatPolicy for hashing
        let mut node_map: HashMap<serde_hashkey::Key<serde_hashkey::OrderedFloatPolicy>, Arc<ErrorModelNode>> = HashMap::new();
        simulator_iter_mut!(self, position, _node, {
            let node_arc: Arc<ErrorModelNode> = error_model.get_node_unwrap_arc(position);
            let node_pointer: *const ErrorModelNode = Arc::as_ptr(&node_arc);
            if arc_set.contains(&node_pointer) {  // already found this pointer, good!
                continue
            }
            // find in hash map
            let hash_key = serde_hashkey::to_key_with_ordered_float(&node_arc).expect("hash");
            match node_map.get(&hash_key) {
                Some(existing_arc) => {
                    // println!("found same error model node, compressing it...");
                    error_model.set_node(position, Some(existing_arc.clone()));
                    continue
                }, None => { }
            }
            // if not found, this is a new value
            arc_set.insert(node_pointer);
            node_map.insert(hash_key, node_arc.clone());
            // println!("found new error model and added");
        });
    }

    /// generate random errors according to the given error rates
    #[inline(never)]
    pub fn generate_random_errors(&mut self, error_model: &ErrorModel) -> (usize, usize) {
        // this size is small compared to the simulator itself
        let allocate_size = self.height * self.vertical * self.horizontal;
        let mut pending_pauli_errors = Vec::<(Position, ErrorType)>::with_capacity(allocate_size);
        let mut pending_erasure_errors = Vec::<Position>::with_capacity(allocate_size);
        // let mut pending_pauli_errors = Vec::<(Position, ErrorType)>::new();
        // let mut pending_erasure_errors = Vec::<Position>::new();
        let mut rng = self.rng.clone();  // avoid mutable borrow
        let mut error_count = 0;
        let mut erasure_count = 0;
        // first apply single-qubit errors
        simulator_iter_mut!(self, position, node, {
            let error_model_node = error_model.get_node_unwrap(position);
            let random_pauli = rng.next_f64();
            if random_pauli < error_model_node.pauli_error_rates.error_rate_X {
                node.set_error_check(error_model, &X);
                // println!("X error at {} {} {}",node.i, node.j, node.t);
            } else if random_pauli < error_model_node.pauli_error_rates.error_rate_X + error_model_node.pauli_error_rates.error_rate_Z {
                node.set_error_check(error_model, &Z);
                // println!("Z error at {} {} {}",node.i, node.j, node.t);
            } else if random_pauli < error_model_node.pauli_error_rates.error_probability() {
                node.set_error_check(error_model, &Y);
                // println!("Y error at {} {} {}",node.i, node.j, node.t);
            } else {
                node.set_error_check(error_model, &I);
            }
            if node.error != I {
                error_count += 1;
            }
            let random_erasure = rng.next_f64();
            node.has_erasure = false;
            node.propagated = I;  // clear propagated errors
            if random_erasure < error_model_node.erasure_error_rate {
                pending_erasure_errors.push(position.clone());
            }
            match &error_model_node.correlated_pauli_error_rates {
                Some(correlated_pauli_error_rates) => {
                    let random_pauli = rng.next_f64();
                    let correlated_pauli_error_type = correlated_pauli_error_rates.generate_random_error(random_pauli);
                    let my_error = correlated_pauli_error_type.my_error();
                    if my_error != I {
                        pending_pauli_errors.push((position.clone(), my_error));
                    }
                    let peer_error = correlated_pauli_error_type.peer_error();
                    if peer_error != I {
                        let gate_peer = node.gate_peer.as_ref().expect("correlated pauli error must corresponds to a two-qubit gate");
                        pending_pauli_errors.push(((**gate_peer).clone(), peer_error));
                    }
                },
                None => { },
            }
            match &error_model_node.correlated_erasure_error_rates {
                Some(correlated_erasure_error_rates) => {
                    let random_erasure = rng.next_f64();
                    let correlated_erasure_error_type = correlated_erasure_error_rates.generate_random_erasure_error(random_erasure);
                    let my_error = correlated_erasure_error_type.my_error();
                    if my_error {
                        pending_erasure_errors.push(position.clone());
                    }
                    let peer_error = correlated_erasure_error_type.peer_error();
                    if peer_error {
                        let gate_peer = node.gate_peer.as_ref().expect("correlated erasure error must corresponds to a two-qubit gate");
                        pending_erasure_errors.push((**gate_peer).clone());
                    }
                },
                None => { },
            }
        });
        // apply pending pauli errors
        for (position, peer_error) in pending_pauli_errors.iter() {
            let node = self.get_node_mut_unwrap(&position);
            if node.error != I {
                error_count -= 1;
            }
            node.set_error_check(error_model, &node.error.multiply(&peer_error));
            if node.error != I {
                error_count += 1;
            }
        }
        // apply pending erasure errors, amd generate random pauli error
        for position in pending_erasure_errors.iter() {
            let mut node = self.get_node_mut_unwrap(&position);
            if !node.has_erasure {  // only counts new erasures; there might be duplicated pending erasure
                erasure_count += 1;
            }
            node.has_erasure = true;
            if node.error != I {
                error_count -= 1;
            }
            let random_erasure = rng.next_f64();
            node.set_error_check(error_model, &(if random_erasure < 0.25 { X }
                else if random_erasure < 0.5 { Z }
                else if random_erasure < 0.75 { Y }
                else { I }
            ));
            if node.error != I {
                error_count += 1;
            };
        }
        debug_assert!({  // the above code avoids iterating the code multiple times when error rate is low (~1%), check correctness in debug mode
            let sparse_error_pattern = self.generate_sparse_error_pattern();
            sparse_error_pattern.len() == error_count
        });
        debug_assert!({
            let sparse_detected_erasures = self.generate_sparse_detected_erasures();
            sparse_detected_erasures.len() == erasure_count
        });
        self.rng = rng;  // save the random number generator
        self.propagate_errors();
        (error_count, erasure_count)
    }

    /// clear all pauli and erasure errors and also propagated errors, returning to a clean state
    pub fn clear_all_errors(&mut self) {
        simulator_iter_mut!(self, position, node, {
            node.error = I;
            node.has_erasure = false;
            node.propagated = I;
        });
    }

    /// must be called before `propagate_errors` to ensure correctness, note that `generate_random_errors` already does this
    #[allow(dead_code)]
    pub fn clear_propagate_errors(&mut self) {
        simulator_iter_mut!(self, position, node, {
            node.propagated = I;
        });
    }

    /// this will be automatically called after `generate_random_errors`, but if user modified the error, they need to call this function again
    #[inline(never)]
    pub fn propagate_errors(&mut self) {
        debug_assert!({
            let mut propagated_clean = true;
            simulator_iter!(self, position, node, {
                if node.propagated != I {
                    propagated_clean = false;
                }
            });
            if !propagated_clean {
                println!("[warning] propagate state must be clean before calling `propagate_errors`");
                println!("    note that `generate_random_errors` automatically cleared it, otherwise you need to manually call `clear_propagate_errors`");
            }
            propagated_clean
        });
        for t in 0..self.height - 1 {
            simulator_iter!(self, position, _node, t => t, {
                self.propagate_error_from(position);
            });
        }
    }

    /// calculate propagated errors at one position. in order to correctly propagate every error, the order of propagation must be ascending in `t`s.
    /// note that errors are propagated to the next time, i.e. `t + 1`.
    /// when a error (other than Identity) propagates to the peer, it returns the position of the peer.
    #[inline]
    pub fn propagate_error_from(&mut self, position: &Position) -> Option<Position> {
        debug_assert!(position.t < self.height - 1, "propagate error from final layer is meaningless, because it doesn't have any next layer");
        let node = self.get_node_unwrap(position);
        // propagation from virtual to real is forbidden
        let propagate_to_peer_forbidden = node.is_virtual && !node.is_peer_virtual;
        // error will propagated to itself at `t+1`, this will initialize `propagated` at `t+1`
        let node_propagated = node.propagated.clone();
        let node_gate_peer = node.gate_peer.clone();
        let propagate_to_next = node.error.multiply(&node_propagated);
        let gate_type = node.gate_type.clone();
        let next_position = &mut position.clone();
        next_position.t += 1;
        let next_node = self.get_node_mut_unwrap(next_position);
        next_node.propagated = next_node.propagated.multiply(&propagate_to_next);  // multiply the propagated error
        if gate_type.is_initialization() {
            next_node.propagated = I;  // no error after initialization
        }
        // propagate error to gate peer
        if !propagate_to_peer_forbidden && gate_type.is_two_qubit_gate() {
            let propagate_to_peer = gate_type.propagate_peer(&node_propagated);
            if propagate_to_peer != I {
                let mut next_peer_position: Position = (*node_gate_peer.unwrap()).clone();
                next_peer_position.t += 1;
                let peer_node = self.get_node_mut_unwrap(&next_peer_position);
                peer_node.propagated = peer_node.propagated.multiply(&propagate_to_peer);
                return Some(next_peer_position)
            }
        }
        None
    }

    /// use sparse measurement to efficiently iterate over non-trivial measurements
    #[inline(never)]
    pub fn generate_sparse_measurement(&self) -> SparseMeasurement {
        let mut sparse_measurement = SparseMeasurement::new();
        for t in (self.measurement_cycles..self.height).step_by(self.measurement_cycles) {
            // only iterate over real stabilizers, excluding those non-existing virtual stabilizers
            simulator_iter_real!(self, position, node, t => t, {
                if node.gate_type.is_measurement() {
                    let this_result = node.gate_type.stabilizer_measurement(&node.propagated);
                    let mut previous_position = position.clone();
                    loop {  // usually this loop execute only once because the previous measurement is found immediately
                        debug_assert!(previous_position.t >= self.measurement_cycles, "cannot find the previous measurement cycle");
                        previous_position.t -= self.measurement_cycles;
                        let previous_node = self.get_node_unwrap(&previous_position);
                        if previous_node.gate_type.is_measurement() {  // found previous measurement
                            let previous_result = previous_node.gate_type.stabilizer_measurement(&previous_node.propagated);
                            if this_result != previous_result {
                                sparse_measurement.insert_nontrivial_measurement(position);
                            }
                            break
                        }
                        println!("[warning] no measurement found in previous round, continue searching...")
                    }
                }
            });
        }
        sparse_measurement
    }

    /// including virtual measurements in the result as an extension to [`Simulator::generate_sparse_measurement`]
    #[inline(never)]
    pub fn generate_sparse_measurement_virtual(&self) -> SparseMeasurement {
        let mut sparse_measurement_virtual = SparseMeasurement::new();
        for t in (self.measurement_cycles..self.height).step_by(self.measurement_cycles) {
            // only iterate over virtual stabilizers, excluding those real stabilizers
            simulator_iter_virtual!(self, position, node, t => t, {
                if node.gate_type.is_measurement() {
                    let this_result = node.gate_type.stabilizer_measurement(&node.propagated);
                    let mut previous_position = position.clone();
                    loop {  // usually this loop execute only once because the previous measurement is found immediately
                        debug_assert!(previous_position.t >= self.measurement_cycles, "cannot find the previous measurement cycle");
                        previous_position.t -= self.measurement_cycles;
                        let previous_node = self.get_node_unwrap(&previous_position);
                        if previous_node.gate_type.is_measurement() {  // found previous measurement
                            let previous_result = previous_node.gate_type.stabilizer_measurement(&previous_node.propagated);
                            if this_result != previous_result {
                                sparse_measurement_virtual.insert_nontrivial_measurement(position);
                            }
                            break
                        }
                        println!("[warning] no measurement found in previous round, continue searching...")
                    }
                }
            });
        }
        sparse_measurement_virtual
    }

    /// generate detected erasures
    #[inline(never)]
    pub fn generate_sparse_detected_erasures(&self) -> SparseDetectedErasures {
        let mut sparse_detected_erasures = SparseDetectedErasures::new();
        simulator_iter_real!(self, position, node, {
            if node.has_erasure {
                sparse_detected_erasures.erasures.insert(position.clone());
            }
        });
        sparse_detected_erasures
    }

    #[inline(never)]
    pub fn fast_measurement_given_few_errors(&mut self, sparse_errors: &SparseErrorPattern) -> (SparseCorrection, SparseMeasurement, SparseMeasurement) {
        if sparse_errors.len() == 0 {
            println!("[warning] why calling fast measurement given no error?");
            return (SparseCorrection::new(), SparseMeasurement::new(), SparseMeasurement::new())
        }
        debug_assert!({  // fast measurement requires no errors at first
            let mut dirty = false;
            simulator_iter!(self, position, node, {
                if node.error != I || node.propagated != I || node.has_erasure {
                    dirty = true;
                }
            });
            !dirty
        });
        let mut interested_region: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut min_t = self.height - 1;
        let mut max_t = 0;
        for (position, error) in sparse_errors.iter() {
            let node = self.get_node_mut_unwrap(position);
            node.error = *error;
            interested_region.insert((position.i, position.j));
            if position.t < min_t {
                min_t = position.t;
            }
            if position.t > max_t {
                max_t = position.t;
            }
        }
        // println!("min_t: {}, max_t: {}, interested_region: {:?}", min_t, max_t, interested_region);
        // propagate error if until no measurement errors are observed
        let mut sparse_measurement_real = SparseMeasurement::new();
        let mut sparse_measurement_virtual = SparseMeasurement::new();
        let mut accumulated_clean_measurements = 0;
        let early_break_accumulated_clean_measurements = 2;  // 1 is not enough, consider increasing this if still not enough
        for t in min_t + 1 .. self.height {
            let mut pending_interested_region = Vec::new();
            for &(i, j) in interested_region.iter() {
                let propagated_neighbor = self.propagate_error_from(&pos!(t - 1, i, j));
                match propagated_neighbor {
                    Some(peer) => pending_interested_region.push((peer.i, peer.j)),
                    None => { },
                }
            }
            for (i, j) in pending_interested_region.drain(..) {
                interested_region.insert((i, j));
            }
            if t != 0 && t % self.measurement_cycles == 0 {  // it's a layer of measurement
                if t > max_t {  // accumulate only after the reaching the original max_t
                    accumulated_clean_measurements += 1;
                }
                for &(i, j) in interested_region.iter() {
                    let position = &pos!(t, i, j);
                    let node = self.get_node_unwrap(position);
                    if node.gate_type.is_measurement() {
                        let this_result = node.gate_type.stabilizer_measurement(&node.propagated);
                        let mut previous_position = position.clone();
                        loop {  // usually this loop execute only once because the previous measurement is found immediately
                            debug_assert!(previous_position.t >= self.measurement_cycles, "cannot find the previous measurement cycle");
                            previous_position.t -= self.measurement_cycles;
                            let previous_node = self.get_node_unwrap(&previous_position);
                            if previous_node.gate_type.is_measurement() {  // found previous measurement
                                let previous_result = previous_node.gate_type.stabilizer_measurement(&previous_node.propagated);
                                if this_result != previous_result {
                                    if node.is_virtual {
                                        sparse_measurement_virtual.insert_nontrivial_measurement(position);
                                    } else {
                                        sparse_measurement_real.insert_nontrivial_measurement(position);
                                    }
                                    accumulated_clean_measurements = 0;
                                }
                                break
                            }
                            println!("[warning] no measurement found in previous round, continue searching...")
                        }
                    }
                }
            }
            if t > max_t {
                max_t = t;
                // if no more non-trivial measurements, break early
                if accumulated_clean_measurements >= early_break_accumulated_clean_measurements {
                    break
                }
            }
        }
        // create sparse correction
        let mut sparse_correction = SparseCorrection::new();
        simulator_iter!(self, position, node, t => max_t, {
            if node.propagated != I && node.qubit_type == QubitType::Data {
                let mut correction_position = position.clone();
                correction_position.t = self.height - 1;  // sparse correction always has t = self.height - 1, top layer
                sparse_correction.add(correction_position, node.propagated);
            }
        });
        // println!("min_t: {}, max_t: {}, interested_region: {:?}, sparse_measurement_real: {:?}", min_t, max_t, interested_region, sparse_measurement_real);
        // clear errors in interested region
        for t in min_t .. max_t + 1 {
            for &(i, j) in interested_region.iter() {
                let node = self.get_node_mut_unwrap(&pos!(t, i, j));
                node.error = ErrorType::I;
                node.propagated = ErrorType::I;
            }
        }
        debug_assert!({  // fast measurement should recover the errors before return
            let mut dirty = false;
            simulator_iter!(self, position, node, {
                if node.error != I || node.propagated != I || node.has_erasure {
                    dirty = true;
                }
            });
            !dirty
        });
        // in debug mode, check the early break indeed works
        debug_assert!({
            for (position, error) in sparse_errors.iter() {
                let node = self.get_node_mut_unwrap(position);
                node.error = *error;
            }
            self.propagate_errors();
            let standard_measurements_real = self.generate_sparse_measurement();
            let standard_measurements_virtual = self.generate_sparse_measurement_virtual();
            let standard_correction = self.generate_sparse_correction();
            self.clear_all_errors();
            // println!("sparse_measurement_real: {:?}, standard_measurements_real: {:?}", sparse_measurement_real, standard_measurements_real);
            // println!("sparse_measurement_virtual: {:?}, standard_measurements_virtual: {:?}", sparse_measurement_virtual, standard_measurements_virtual);
            let mut measurements_equal = sparse_measurement_real.nontrivial.len() == standard_measurements_real.nontrivial.len()
                && sparse_measurement_virtual.nontrivial.len() == standard_measurements_virtual.nontrivial.len();
            if measurements_equal {  // further check for each element
                for position in standard_measurements_real.nontrivial.iter() {
                    if !sparse_measurement_real.nontrivial.contains(position) {
                        measurements_equal = false;
                        println!("[error] nontrivial measurement happens at {} but optimized code doesn't correctly detect it", position);
                    }
                }
                for position in standard_measurements_virtual.nontrivial.iter() {
                    if !sparse_measurement_virtual.nontrivial.contains(position) {
                        measurements_equal = false;
                        println!("[error] nontrivial measurement happens at {} but optimized code doesn't correctly detect it", position);
                    }
                }
            }
            // println!("sparse_correction: {:?}, standard_correction: {:?}", sparse_correction, standard_correction);
            let mut correction_equal = sparse_correction.len() == standard_correction.len();
            if correction_equal {  // further check for each element
                for (position, error) in standard_correction.iter() {
                    if sparse_correction.get(position) != Some(error) {
                        correction_equal = false;
                        println!("[error] sparse correction not equal at {}, {} != {:?}", position, error, sparse_correction.get(position));
                    }
                }
            }
            measurements_equal && correction_equal
        });
        (sparse_correction, sparse_measurement_real, sparse_measurement_virtual)
    }

    /// generate error pattern
    pub fn generate_sparse_error_pattern(&self) -> SparseErrorPattern {
        let mut sparse_error_pattern = SparseErrorPattern::new();
        simulator_iter!(self, position, node, {
            if node.error != I {
                sparse_error_pattern.add(position.clone(), node.error);
            }
        });
        sparse_error_pattern
    }

    /// generate correction pattern using errors only at the top layer
    pub fn generate_sparse_correction(&self) -> SparseCorrection {
        let mut sparse_correction = SparseCorrection::new();
        simulator_iter!(self, position, node, t => self.height - 1, {
            if node.propagated != I && node.qubit_type == QubitType::Data {
                sparse_correction.add(position.clone(), node.propagated);
            }
        });
        sparse_correction
    }

    /// test if correction successfully recover the logical information
    #[inline(never)]
    pub fn validate_correction(&mut self, correction: &SparseCorrection) -> (bool, bool) {
        if let Some((logical_i, logical_j)) = code_builder_validate_correction(self, correction) {
            return (logical_i, logical_j)
        }
        unimplemented!("correction validation method not found for this code");
    }

}

impl Simulator{
    /// get `self.nodes[t][i][j]` without position check when compiled in release mode
    #[inline]
    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<Box<SimulatorNode>> {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        &self.nodes[position.t][position.i][position.j]
    }

    /// get mutable `self.nodes[t][i][j]` without position check when compiled in release mode
    #[inline]
    pub fn get_node_mut(&'_ mut self, position: &Position) -> &'_ mut Option<Box<SimulatorNode>> {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        &mut self.nodes[position.t][position.i][position.j]
    }

    /// get mutable `self.nodes[t][i][j]` and unwrap without position check when compiled in release mode
    #[inline]
    pub fn get_node_mut_unwrap(&'_ mut self, position: &Position) -> &'_ mut SimulatorNode {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        debug_assert!(self.is_node_exist(position), "position {} does not exist in the simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        self.get_node_mut(position).as_mut().unwrap()
    }

    /// get `self.nodes[t][i][j]` and then unwrap without position check when compiled in release mode
    #[inline]
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ SimulatorNode {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        debug_assert!(self.is_node_exist(position), "position {} does not exist in the simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        self.get_node(position).as_ref().unwrap()
    }

    /// load detected erasures back to the simulator
    pub fn load_sparse_detected_erasures(&mut self, sparse_detected_erasures: &SparseDetectedErasures) -> Result<(), String> {
        for position in sparse_detected_erasures.iter() {
            if !self.is_node_exist(position) {
                return Err(format!("invalid erasure at position {}", position))
            }
        }
        simulator_iter_mut!(self, position, node, {
            node.has_erasure = sparse_detected_erasures.contains(position);
        });
        Ok(())
    }
    
    /// load an error pattern
    pub fn load_sparse_error_pattern(&mut self, sparse_error_pattern: &SparseErrorPattern) -> Result<(), String> {
        for (position, _error) in sparse_error_pattern.iter() {
            if !self.is_node_exist(position) {
                return Err(format!("invalid error at position {}", position))
            }
        }
        simulator_iter_mut!(self, position, node, {
            node.error = I;
            if let Some(error) = sparse_error_pattern.get(position) {
                node.error = *error;
            }
        });
        Ok(())
    }

    /// create json object for debugging and viewing
    pub fn to_json(&self, error_model: &ErrorModel) -> serde_json::Value {
        json!({
            "code_type": self.code_type,
            "height": self.height,
            "vertical": self.vertical,
            "horizontal": self.horizontal,
            "nodes": (0..self.height).map(|t| {
                (0..self.vertical).map(|i| {
                    (0..self.horizontal).map(|j| {
                        let position = &pos!(t, i, j);
                        if self.is_node_exist(position) {
                            let node = self.get_node_unwrap(position);
                            let error_model_node = error_model.get_node_unwrap(position);
                            Some(json!({
                                "position": position,
                                "qubit_type": node.qubit_type,
                                "gate_type": node.gate_type,
                                "gate_peer": node.gate_peer,
                                "is_virtual": node.is_virtual,
                                "is_peer_virtual": node.is_peer_virtual,
                                "error_model": error_model_node,
                            }))
                        } else {
                            None
                        }
                    }).collect::<Vec<Option<serde_json::Value>>>()
                }).collect::<Vec<Vec<Option<serde_json::Value>>>>()
            }).collect::<Vec<Vec<Vec<Option<serde_json::Value>>>>>()
        })
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            t: usize::MAX,
            i: usize::MAX,
            j: usize::MAX,
        }
    }
}

impl Ord for Position {
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

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pymethods)]
impl Position {
    #[cfg_attr(feature = "python_binding", new)]
    pub fn new(t: usize, i: usize, j: usize) -> Self {
        Self {
            t: t,
            i: i,
            j: j,
        }
    }
    pub fn distance(&self, other: &Self) -> usize {
        ((self.t as isize - other.t as isize).abs() + (self.i as isize - other.i as isize).abs() + (self.j as isize - other.j as isize).abs()) as usize
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}][{}][{}]", self.t, self.i, self.j)
    }
}

impl Serialize for Position {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
        serializer.serialize_str(format!("[{}][{}][{}]", self.t, self.i, self.j).as_str())
    }
}

impl<'de> Visitor<'de> for Position {
    type Value = Position;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", r#"position should look like "[0][10][13]""#)
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
        if s.get(0..1) != Some("[") {
            return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self))
        }
        if s.get(s.len()-1..s.len()) != Some("]") {
            return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self))
        }
        let splitted = s.get(1..s.len()-1).unwrap().split("][").collect::<Vec<&str>>();
        if splitted.len() != 3 {
            return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self))
        }
        let t = match splitted[0].to_string().parse::<usize>() {
            Ok(t) => t,
            Err(_) => { return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self)) }
        };
        let i = match splitted[1].to_string().parse::<usize>() {
            Ok(t) => t,
            Err(_) => { return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self)) }
        };
        let j = match splitted[2].to_string().parse::<usize>() {
            Ok(t) => t,
            Err(_) => { return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &self)) }
        };
        Ok(Position::new(t, i, j))
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de>, {
        // the new-ed position just works like a helper type that implements Visitor trait, not optimized for efficiency
        deserializer.deserialize_str(Position::new(usize::MAX, usize::MAX, usize::MAX))
    }
}

impl std::fmt::Display for SimulatorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SimulatorNode{} {{ qubit_type: {:?}, gate_type: {:?}, gate_peer: {:?} }}"
            , if self.is_virtual{ "(virtual)" } else { "" }, self.qubit_type, self.gate_type, self.gate_peer)
    }
}

/// in most cases non-trivial measurements are rare, this sparse structure use `BTreeSet` to store them
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct SparseMeasurement {
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub nontrivial: BTreeSet<Position>,
}

// #[cfg_attr(feature = "python_binding", pymethods)]
impl SparseMeasurement {
    /// create a new clean measurement without nontrivial measurements
    // #[cfg_attr(feature = "python_binding", new)]
    pub fn new() -> Self {
        Self {
            nontrivial: BTreeSet::new(),
        }
    }
    /// return false if this nontrivial measurement is already present
    #[inline]
    pub fn insert_nontrivial_measurement(&mut self, position: &Position) -> bool {
        self.nontrivial.insert(position.clone())
    }
    /// iterator
    pub fn iter<'a>(&'a self) -> std::collections::btree_set::Iter<'a, Position> {
        self.nontrivial.iter()
    }
    /// convert to vector in ascending order
    pub fn to_vec(&self) -> Vec<Position> {
        self.iter().map(|position| (*position).clone()).collect()
    }
    /// convert vector to sparse measurement
    pub fn from_vec(nontrivial: &Vec<Position>) -> Self {
        let mut sparse_measurement = Self::new();
        for position in nontrivial.iter() {
            debug_assert!(!sparse_measurement.nontrivial.contains(position), "duplicate nontrivial measurement forbidden");
            sparse_measurement.insert_nontrivial_measurement(position);
        }
        sparse_measurement
    }
    /// the length of non-trivial measurements
    pub fn len(&self) -> usize {
        self.nontrivial.len()
    }
}

/// detected erasures along with its effected edges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct SparseDetectedErasures {
    /// the position of the erasure errors
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub erasures: BTreeSet<Position>,
}

impl SparseDetectedErasures {
    /// create a new clean measurement without nontrivial measurements
    pub fn new() -> Self {
        Self {
            erasures: BTreeSet::new(),
        }
    }
    /// iterator
    pub fn iter<'a>(&'a self) -> std::collections::btree_set::Iter<'a, Position> {
        self.erasures.iter()
    }
    /// the length of non-trivial measurements
    pub fn len(&self) -> usize {
        self.erasures.len()
    }
    /// contains element
    pub fn contains(&self, key: &Position) -> bool {
        self.erasures.contains(key)
    }
    /// compute the edges that are re-weighted to 0 because of these erasures
    pub fn get_erasure_edges(&self, erasure_graph: &ErasureGraph) -> Vec<ErasureEdge> {
        let mut erasure_edges = Vec::<ErasureEdge>::new();
        for erasure in self.erasures.iter() {
            let erasure_node = erasure_graph.get_node_unwrap(erasure);
            for erasure_edge in erasure_node.erasure_edges.iter() {
                erasure_edges.push(erasure_edge.clone());
            }
        }
        erasure_edges
    }
}

/// in most cases errors are rare, this sparse structure use `BTreeMap` to store them
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct SparseErrorPattern {
    /// error happening at position: Position (t, i, j)
    #[cfg_attr(feature = "python_binding", pyo3(get, set))]
    pub errors: BTreeMap<Position, ErrorType>,
}

impl SparseErrorPattern {
    /// create an empty error pattern
    pub fn new() -> Self {
        Self {
            errors: BTreeMap::new(),
        }
    }
    /// extend an error pattern using another error pattern
    #[allow(dead_code)]
    pub fn extend(&mut self, next: &Self) {
        for (position, error) in next.iter() {
            self.add(position.clone(), *error);
        }
    }
    /// add an error at some position, if an error already presents, then multiply them
    pub fn add(&mut self, position: Position, error: ErrorType) {
        if let Some(node_error) = self.errors.get_mut(&position) {
            *node_error = node_error.multiply(&error);
        } else {
            self.errors.insert(position, error);
        }
    }
    /// iterator
    pub fn iter<'a>(&'a self) -> std::collections::btree_map::Iter<'a, Position, ErrorType> {
        self.errors.iter()
    }
    /// length
    pub fn len(&self) -> usize {
        self.errors.len()
    }
    /// get element
    pub fn get(&self, key: &Position) -> Option<&ErrorType> {
        self.errors.get(key)
    }
}

impl Serialize for SparseErrorPattern {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
        let mut map = serializer.serialize_map(Some(self.len()))?;  // known length
        for (position, error) in self.iter() {
            map.serialize_entry(position, error)?;
        }
        map.end()
    }
}

impl<'de> Visitor<'de> for SparseErrorPattern {
    type Value = SparseErrorPattern;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", r#"sparse error pattern like {"[0][10][13]":"Z","[0][10][7]":"X","[0][10][8]":"Y"}"#)
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error> where M: MapAccess<'de>, {
        let mut error_pattern = SparseErrorPattern::new();
        while let Some((key, value)) = access.next_entry()? {
            error_pattern.add(key, value);
        }
        Ok(error_pattern)
    }
}

impl<'de> Deserialize<'de> for SparseErrorPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de>, {
        // the new-ed error pattern just works like a helper type that implements Visitor trait, not optimized for efficiency
        deserializer.deserialize_map(SparseErrorPattern::new())
    }
}

/// share methods with [`SparseErrorPattern`] but records **propagated** errors of **data qubits** on **top layer**
/// , thus in principle it's incompatible with [`SparseErrorPattern`] which records individual errors
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct SparseCorrection(SparseErrorPattern);

impl SparseCorrection {
    /// create an empty correction
    pub fn new() -> Self {
        Self(SparseErrorPattern::new())
    }
    /// extend an error pattern using another error pattern
    pub fn extend(&mut self, next: &Self) {
        for (position, error) in next.0.iter() {
            self.add(position.clone(), *error);
        }
    }
    /// add an correction Pauli operator at some position, if an error already presents, then multiply them
    pub fn add(&mut self, position: Position, operator: ErrorType) {
        debug_assert!({  // check `t` are the same
            let mut check_passed = true;
            for (key, _value) in self.0.iter() {
                if key.t != position.t {
                    println!("correction should also have the same `t`, violating: {} and {}", key, position);
                    check_passed = false;
                }
                break  // no need to iterate them all, because every call to this function will be checked
            }
            check_passed
        }, "correction must have the same t");
        self.0.add(position, operator);
    }
    /// iterator
    pub fn iter<'a>(&'a self) -> std::collections::btree_map::Iter<'a, Position, ErrorType> {
        self.0.iter()
    }
    /// length
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// get element
    pub fn get(&self, key: &Position) -> Option<&ErrorType> {
        self.0.get(key)
    }
}

impl Serialize for SparseCorrection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer, {
        Serialize::serialize(&self.0, serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulator_basics() {  // cargo test simulator_basics -- --nocapture
        let di = 5;
        let dj = 5;
        let noisy_measurements = 5;
        let simulator = Simulator::new(CodeType::StandardPlanarCode, BuiltinCodeInformation::new(noisy_measurements, di, dj));
        let invalid_position = pos!(100, 100, 100);
        assert!(!simulator.is_valid_position(&invalid_position), "invalid position");
        let nonexisting_position = pos!(0, 0, 0);
        assert!(simulator.is_valid_position(&nonexisting_position), "valid position");
        assert!(!simulator.is_node_exist(&nonexisting_position), "nonexisting position");
        println!("std::mem::size_of::<SimulatorNode>() = {}", std::mem::size_of::<SimulatorNode>());
        println!("std::mem::size_of::<ErrorModelNode>() = {}", std::mem::size_of::<ErrorModelNode>());
        if std::mem::size_of::<SimulatorNode>() > 32 {  // ArmV8 data cache line is 64 bytes
            panic!("SimulatorNode which is unexpectedly large, check if anything wrong");
        }
    }

}

#[cfg(feature="python_binding")]
#[pyfunction]
pub(crate) fn register(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Simulator>()?;
    m.add_class::<SimulatorNode>()?;
    m.add_class::<Position>()?;
    m.add_class::<GateType>()?;
    m.add_class::<SparseMeasurement>()?;
    m.add_class::<SparseDetectedErasures>()?;
    m.add_class::<SparseErrorPattern>()?;
    m.add_class::<SparseCorrection>()?;
    Ok(())
}
