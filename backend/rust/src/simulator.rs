use std::cmp::Ordering;
use super::types::{QubitType, ErrorType, PauliErrorRates, CorrelatedPauliErrorRates, CorrelatedErasureErrorRates};
use serde::{Serialize, Deserialize};
use super::code_builder::*;
use super::util_macros::*;
use super::reproducible_rand::Xoroshiro128StarStar;
use ErrorType::*;


/// general simulator for two-dimensional code with circuit-level implementation of stabilizer measurements
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Simulator {
    /// information of the preferred code
    pub code_type: CodeType,
    /// size of the snapshot, where `nodes` is ensured to be a cube of `height` * `vertical` * `horizontal`
    pub height: usize,
    pub vertical: usize,
    pub horizontal: usize,
    pub nodes: Vec::< Vec::< Vec::< Option<SimulatorNode> > > >,
    /// use embedded random number generator
    pub rng: Xoroshiro128StarStar,
}

impl Simulator {
    pub fn clone(&self) -> Self {
        Self {
            code_type: self.code_type,
            height: self.height,
            vertical: self.vertical,
            horizontal: self.horizontal,
            nodes: self.nodes.clone(),
            rng: Xoroshiro128StarStar::new(),  // do not copy random number generator, otherwise parallel simulation may give same result
        }
    }
}

/// when plotting, t is the time axis; looking at the direction of `t=-∞`, the top-left corner is `i=j=0`;
/// `i` is vertical position, which increases when moving from top to bottom;
/// `j` is horizontal position, which increases when moving from left to right
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    // pub index: [usize; 3],
    pub t: usize,
    pub i: usize,
    pub j: usize,
}

/// each node represents a location `[i][j]` at a specific time point `[t]`, this location has some probability of having Pauli error or erasure error.
/// we could have single-qubit or two-qubit gate in a node, and errors are added **after applying this gate** (e.g. if the gate is measurement, then 
/// errors at this node will have no impact on the measurement because errors are applied after the measurement).
/// we also maintain "virtual nodes" at the boundary of a code, these virtual nodes are missing stabilizers at the boundary of a open-boundary surface code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimulatorNode {
    /// the position of current node
    pub position: Position,
    pub qubit_type: QubitType,
    /// single-qubit or two-qubit gate applied 
    pub gate_type: GateType,
    pub gate_peer: Option<Position>,
    /// without losing generality, errors are applied after the gate
    #[serde(rename = "pp")]
    pub pauli_error_rates: PauliErrorRates,
    #[serde(rename = "pe")]
    pub erasure_error_rate: f64,
    #[serde(rename = "corr_pp")]
    pub correlated_pauli_error_rates: Option<Box<CorrelatedPauliErrorRates>>,
    #[serde(rename = "corr_pe")]
    pub correlated_erasure_error_rates: Option<Box<CorrelatedErasureErrorRates>>,
    /// simulation data
    #[serde(skip)]
    pub error: ErrorType,
    #[serde(skip)]
    pub has_erasure: bool,
    #[serde(skip)]
    pub propagated: ErrorType,
    /// Virtual qubit doesn't physically exist, which means they will never have errors themselves.
    /// Real qubit errors can propagate to virtual qubits, but errors will never propagate to real qubits.
    /// Virtual qubits can be understood as perfect stabilizers that only absorb propagated errors and never propagate them.
    /// They're useful in tailored surface code decoding, and also to represent virtual boundaries
    pub is_virtual: bool,
    pub is_peer_virtual: bool,
}

impl SimulatorNode {
    pub fn new(position: Position, qubit_type: QubitType, gate_type: GateType, gate_peer: Option<Position>) -> Self {
        Self {
            position: position,
            qubit_type: qubit_type,
            gate_type: gate_type,
            gate_peer: gate_peer,
            pauli_error_rates: PauliErrorRates::default(),
            erasure_error_rate: 0.,
            correlated_pauli_error_rates: None,
            correlated_erasure_error_rates: None,
            error: ErrorType::I,
            has_erasure: false,
            propagated: ErrorType::I,
            is_virtual: false,
            is_peer_virtual: false,
        }
    }
    pub fn set_virtual(mut self, is_virtual: bool, is_peer_virtual: bool) -> Self {
        self.is_virtual = is_virtual;
        self.is_peer_virtual = is_peer_virtual;
        self
    }
    pub fn is_noiseless(&self) -> bool {
        if self.pauli_error_rates.error_probability() > 0. {
            return false
        }
        if self.erasure_error_rate > 0. {
            return false
        }
        if self.correlated_pauli_error_rates.is_some() && self.correlated_pauli_error_rates.as_ref().unwrap().error_probability() > 0. {
            return false
        }
        if self.correlated_erasure_error_rates.is_some() && self.correlated_erasure_error_rates.as_ref().unwrap().error_probability() > 0. {
            return false
        }
        true
    }

    /// check if this position is physically idle: either no gate or a two-qubit gate with a virtual peer
    pub fn is_physically_idle(&self) -> bool {
        self.gate_type == GateType::None || self.is_peer_virtual
    }
}

/// single-qubit and two-qubit gate type
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Copy)]
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
    /// because the gate with virtual peer is non-existing physically. in order to check if a position is physically idle,
    /// use [`SimulatorNode::is_physically_idle`].
    None,
}

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
            GateType::CXGateControl => { if matches!(propagated, X | Y) { X } else { ErrorType::I } }
            // cx target not sensitive to X, propagate as Z
            GateType::CXGateTarget => { if matches!(propagated, Z | Y) { Z } else { ErrorType::I } }
            // cy control not sensitive to Z, propagate as Y
            GateType::CYGateControl => { if matches!(propagated, X | Y) { Y } else { ErrorType::I } }
            // cy target not sensitive to Y, propagate as Z
            GateType::CYGateTarget => { if matches!(propagated, Z | X) { Z } else { ErrorType::I } }
            // cz not sensitive to Z, propagate as Z
            GateType::CZGate => { if matches!(propagated, X | Y) { Z } else { ErrorType::I } }
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

impl Simulator {
    /// given builtin code type, this will automatically build the code structure
    pub fn new(code_type: CodeType) -> Self {
        let mut simulator = Self {
            code_type: code_type,
            height: 0,
            vertical: 0,
            horizontal: 0,
            nodes: Vec::new(),
            rng: Xoroshiro128StarStar::new(),
        };
        build_code(&mut simulator);
        simulator
    }

    /// judge if `[t][i][j]` is valid index of `self.nodes`
    pub fn is_valid_position(&self, position: &Position) -> bool {
        position.t < self.height && position.i < self.vertical && position.j < self.horizontal
    }

    /// judge if `self.nodes[t][i][j]` is `Some(_)`
    pub fn is_node_exist(&self, position: &Position) -> bool {
        self.is_valid_position(position) && self.get_node(position).is_some()
    }

    /// get `self.nodes[t][i][j]` without position check when compiled in release mode
    pub fn get_node(&'_ self, position: &Position) -> &'_ Option<SimulatorNode> {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        &self.nodes[position.t][position.i][position.j]
    }

    /// get `self.nodes[t][i][j]` and then unwrap without position check when compiled in release mode
    pub fn get_node_unwrap(&'_ self, position: &Position) -> &'_ SimulatorNode {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        debug_assert!(self.is_node_exist(position), "position {} does not exist in the simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        self.get_node(position).as_ref().unwrap()
    }

    /// get mutable `self.nodes[t][i][j]` without position check when compiled in release mode
    pub fn get_node_mut(&'_ mut self, position: &Position) -> &'_ mut Option<SimulatorNode> {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        &mut self.nodes[position.t][position.i][position.j]
    }

    /// get mutable `self.nodes[t][i][j]` and unwrap without position check when compiled in release mode
    pub fn get_node_mut_unwrap(&'_ mut self, position: &Position) -> &'_ mut SimulatorNode {
        debug_assert!(self.is_valid_position(position), "position {} is invalid in a simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        debug_assert!(self.is_node_exist(position), "position {} does not exist in the simulator with size [{}][{}][{}]"
            , position, self.height, self.vertical, self.horizontal);
        self.get_node_mut(position).as_mut().unwrap()
    }

    /// check if this node is a real node, i.e. physically exist in the simulation
    pub fn is_node_real(&self, position: &Position) -> bool {
        self.is_node_exist(position) && self.get_node_unwrap(position).is_virtual == false
    }

    /// check if this node is a virtual node, i.e. non-existing but just work as a virtual boundary
    /// (they can be viewed as the missing stabilizers on the boundary)
    pub fn is_node_virtual(&self, position: &Position) -> bool {
        self.is_node_exist(position) && self.get_node_unwrap(position).is_virtual == true
    }

    /// check if this node is a virtual node, i.e. non-existing but just work as a virtual boundary
    pub fn set_error_rates(&mut self, px: f64, py: f64, pz: f64, pe: f64) {
        assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
        assert!(pe <= 1. && pe >= 0.);
        let measurement_cycles = match self.code_type.builtin_code_information() {
            Some(BuiltinCodeInformation{ measurement_cycles, .. }) => {
                measurement_cycles
            },
            _ => {
                println!("[warning] setting error rates of unknown code, no perfect measurement protection is enabled");
                0
            }
        };
        for t in 0 .. self.height - measurement_cycles {
            simulator_iter_mut_real!(self, position, node, t => t, {  // only add errors on real node
                node.pauli_error_rates.error_rate_X = px;
                node.pauli_error_rates.error_rate_Y = py;
                node.pauli_error_rates.error_rate_Z = pz;
                node.erasure_error_rate = pe;
            });
        }
    }

    /// check if error rates are not zero at perfect measurement ranges or at virtual nodes
    pub fn error_rate_sanity_check(&self) -> Result<(), String> {
        match self.code_type.builtin_code_information() {
            Some(BuiltinCodeInformation{ measurement_cycles, noisy_measurements, .. }) => {
                // check that no errors present in the final perfect measurement rounds
                let expected_height = measurement_cycles * (noisy_measurements + 1) + 1;
                if self.height != expected_height {
                    return Err(format!("height {} is not expected {}, don't know where is perfect measurement", self.height, expected_height))
                }
                for t in self.height - measurement_cycles .. self.height {
                    simulator_iter!(self, position, node, t => t, {
                        if !node.is_noiseless() {
                            return Err(format!("detected noisy position {} within final perfect measurement", position))
                        }
                    });
                }
                // check all no error rate at virtual nodes
                simulator_iter_virtual!(self, position, node, {  // only check for virtual nodes
                    if !node.is_noiseless() {
                        return Err(format!("detected noisy position {} which is virtual node", position))
                    }
                });
            }, _ => {println!("[warning] code doesn't provide enough information for sanity check") }
        }
        Ok(())
    }

    /// expand the correlated error rates, useful when exporting the data structure for other applications to modify
    pub fn expand_error_rates(&mut self) {
        simulator_iter_mut!(self, position, node, {
            if node.correlated_pauli_error_rates.is_none() {
                node.correlated_pauli_error_rates = Some(Box::new(CorrelatedPauliErrorRates::default()));
            }
            if node.correlated_erasure_error_rates.is_none() {
                node.correlated_erasure_error_rates = Some(Box::new(CorrelatedErasureErrorRates::default()));
            }
        });
    }

    /// compress the correlated error rates, useful when importing modified data structure from other applications
    pub fn compress_error_rates(&mut self) {
        simulator_iter_mut!(self, position, node, {
            if node.correlated_pauli_error_rates.is_some() {
                if node.correlated_pauli_error_rates.as_ref().unwrap().error_probability() == 0. {
                    node.correlated_pauli_error_rates = None;
                }
            }
            if node.correlated_erasure_error_rates.is_some() {
                if node.correlated_erasure_error_rates.as_ref().unwrap().error_probability() == 0. {
                    node.correlated_erasure_error_rates = None;
                }
            }
        });
    }

    /// generate random errors according to the given error rates
    #[inline(never)]
    pub fn generate_random_errors(&mut self) -> usize {
        // this size is small compared to the simulator itself
        let allocate_size = self.height * self.vertical * self.horizontal;
        let mut pending_pauli_errors = Vec::<(Position, ErrorType)>::with_capacity(allocate_size);
        let mut pending_erasure_errors = Vec::<Position>::with_capacity(allocate_size);
        // let mut pending_pauli_errors = Vec::<(Position, ErrorType)>::new();
        // let mut pending_erasure_errors = Vec::<Position>::new();
        let mut rng = self.rng.clone();  // avoid mutable borrow
        let mut error_count = 0;
        // first apply single-qubit errors
        simulator_iter_mut!(self, position, node, {
            let random_pauli = rng.next_f64();
            if random_pauli < node.pauli_error_rates.error_rate_X {
                node.error = X;
                // println!("X error at {} {} {}",node.i, node.j, node.t);
            } else if random_pauli < node.pauli_error_rates.error_rate_X + node.pauli_error_rates.error_rate_Z {
                node.error = Z;
                // println!("Z error at {} {} {}",node.i, node.j, node.t);
            } else if random_pauli < node.pauli_error_rates.error_probability() {
                node.error = Y;
                // println!("Y error at {} {} {}",node.i, node.j, node.t);
            } else {
                node.error = ErrorType::I;
            }
            if node.error != ErrorType::I {
                error_count += 1;
            }
            let random_erasure = rng.next_f64();
            node.has_erasure = false;
            if random_erasure < node.erasure_error_rate {
                pending_erasure_errors.push(*position);
            }
            match &node.correlated_pauli_error_rates {
                Some(correlated_pauli_error_rates) => {
                    let random_pauli = rng.next_f64();
                    let correlated_pauli_error_type = correlated_pauli_error_rates.generate_random_error(random_pauli);
                    let my_error = correlated_pauli_error_type.my_error();
                    if my_error != ErrorType::I {
                        pending_pauli_errors.push((*position, my_error));
                    }
                    let peer_error = correlated_pauli_error_type.peer_error();
                    if peer_error != ErrorType::I {
                        let gate_peer = node.gate_peer.as_ref().expect("correlated pauli error must corresponds to a two-qubit gate");
                        pending_pauli_errors.push((*gate_peer, peer_error));
                    }
                },
                None => { },
            }
            match &node.correlated_erasure_error_rates {
                Some(correlated_erasure_error_rates) => {
                    let random_erasure = rng.next_f64();
                    let correlated_erasure_error_type = correlated_erasure_error_rates.generate_random_erasure_error(random_erasure);
                    let my_error = correlated_erasure_error_type.my_error();
                    if my_error {
                        pending_erasure_errors.push(*position);
                    }
                    let peer_error = correlated_erasure_error_type.peer_error();
                    if peer_error {
                        let gate_peer = node.gate_peer.as_ref().expect("correlated erasure error must corresponds to a two-qubit gate");
                        pending_erasure_errors.push(*gate_peer);
                    }
                },
                None => { },
            }
        });
        // apply pending pauli errors
        for (position, peer_error) in pending_pauli_errors.iter() {
            let mut node = self.get_node_mut_unwrap(&position);
            if node.error != ErrorType::I {
                error_count -= 1;
            }
            node.error = node.error.multiply(&peer_error);
            if node.error != ErrorType::I {
                error_count += 1;
            }
        }
        // apply pending erasure errors, amd generate random pauli error
        for position in pending_erasure_errors.iter() {
            let mut node = self.get_node_mut_unwrap(&position);
            node.has_erasure = true;
            if node.error != ErrorType::I {
                error_count -= 1;
            }
            let random_erasure = rng.next_f64();
            node.error = if random_erasure < 0.25 { X }
                else if random_erasure < 0.5 { Z }
                else if random_erasure < 0.75 { Y }
                else { ErrorType::I };
            if node.error != ErrorType::I {
                error_count += 1;
            }
        }
        debug_assert!({  // the above code avoids iterating the code multiple times when error rate is low (~1%), check correctness in debug mode
            let mut real_error_count = 0;
            simulator_iter!(self, position, node, {
                if node.error != ErrorType::I {
                    real_error_count += 1;
                }
            });
            real_error_count == error_count
        });
        self.rng = rng;  // save the random number generator
        self.propagate_errors();
        error_count
    }

    /// clear all pauli and erasure errors and also propagated errors, returning to a clean state
    #[allow(dead_code)]
    pub fn clear_all_errors(&mut self) {
        simulator_iter_mut!(self, position, node, {
            node.error = ErrorType::I;
            node.has_erasure = false;
            node.propagated = ErrorType::I;
        });
    }

    /// must be called before `propagate_errors` to ensure correctness, note that `generate_random_errors` already does this
    #[allow(dead_code)]
    pub fn clear_propagate_errors(&mut self) {
        simulator_iter_mut!(self, position, node, {
            node.propagated = ErrorType::I;
        });
    }

    /// this will be automatically called after `generate_random_errors`, but if user modified the error, they need to call this function again
    #[inline(never)]
    pub fn propagate_errors(&mut self) {
        debug_assert!({
            let mut propagated_clean = true;
            simulator_iter!(self, position, node, {
                if node.propagated != ErrorType::I {
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
    /// note that errors are propagated to the next time, i.e. `t + 1`
    pub fn propagate_error_from(&mut self, position: &Position) -> Option<Position> {
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
            next_node.propagated = ErrorType::I;  // no error after initialization
        }
        // propagate error to gate peer
        if !propagate_to_peer_forbidden && gate_type.is_two_qubit_gate() {
            let propagate_to_peer = gate_type.propagate_peer(&node_propagated);
            if propagate_to_peer != ErrorType::I {
                let mut next_peer_position = node_gate_peer.unwrap();
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
        let measurement_cycles = match self.code_type.builtin_code_information() {
            Some(BuiltinCodeInformation{ measurement_cycles, .. }) => {
                measurement_cycles
            },
            _ => {
                println!("[warning] generate measurement of unknown code, fall back to slower speed");
                1
            }
        };
        for t in (measurement_cycles..self.height).step_by(measurement_cycles) {
            // only iterate over real stabilizers, excluding those non-existing virtual stabilizers
            simulator_iter_real!(self, position, node, t => t, {
                if node.gate_type.is_measurement() {
                    let this_result = node.gate_type.stabilizer_measurement(&node.propagated);
                    let mut previous_position = position.clone();
                    loop {  // usually this loop execute only once because the previous measurement is found immediately
                        previous_position.t -= measurement_cycles;
                        let previous_node = self.get_node_unwrap(&previous_position);
                        if previous_node.gate_type.is_measurement() {  // found previous measurement
                            let previous_result = previous_node.gate_type.stabilizer_measurement(&previous_node.propagated);
                            if this_result != previous_result {
                                sparse_measurement.insert_nontrivial_measurement(position);
                            }
                            break
                        }
                    }
                }
            });
        }
        sparse_measurement
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

impl Position {
    pub fn new(t: usize, i: usize, j: usize) -> Self {
        Self {
            t: t,
            i: i,
            j: j,
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}][{}][{}]", self.t, self.i, self.j)
    }
}

impl std::fmt::Display for SimulatorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SimulatorNode{}{} {{ qubit_type: {:?}, gate_type: {:?}, gate_peer: {:?} }}", self.position
            , if self.is_virtual{ "(virtual)" } else { "" }, self.qubit_type, self.gate_type, self.gate_peer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseMeasurement {
    pub nontrivial: std::collections::BTreeSet<Position>,
}

impl SparseMeasurement {
    pub fn new() -> Self {
        Self {
            nontrivial: std::collections::BTreeSet::new(),
        }
    }
    /// return false if this nontrivial measurement is already present
    pub fn insert_nontrivial_measurement(&mut self, position: &Position) -> bool {
        self.nontrivial.insert(position.clone())
    }
    /// convert to vector in ascending order
    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<Position> {
        self.nontrivial.iter().map(|position| *position).collect()
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
        let simulator = Simulator::new(CodeType::StandardPlanarCode { noisy_measurements, di, dj });
        let invalid_position = pos!(100, 100, 100);
        assert!(!simulator.is_valid_position(&invalid_position), "invalid position");
        let nonexisting_position = pos!(0, 0, 0);
        assert!(simulator.is_valid_position(&nonexisting_position), "valid position");
        assert!(!simulator.is_node_exist(&nonexisting_position), "nonexisting position");
        if std::mem::size_of::<SimulatorNode>() > 128 {  // ArmV8 data cache line is 64 bytes
            panic!("std::mem::size_of::<SimulatorNode>() = {} which is unexpectedly large, check if anything wrong", std::mem::size_of::<SimulatorNode>());
        }
    }

}
