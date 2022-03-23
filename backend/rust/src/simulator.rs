
#![allow(unused_imports)]
#![allow(dead_code)]

use super::ndarray;
use super::petgraph;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use super::blossom_v;
use std::sync::{Arc};
use super::types::{QubitType, ErrorType, CorrelatedErrorType, ErrorModel, PauliErrorRates, CorrelatedPauliErrorRates, CorrelatedErasureErrorRates};
use super::union_find_decoder;
use super::either::Either;
use super::serde_json;
use std::time::Instant;
use super::fast_benchmark::FastBenchmark;
use serde::{Serialize, Deserialize};
use super::util;
use super::util::simple_hasher::SimpleHasher;
use super::union_find_decoder::UnionFind;
use super::code_builder::*;
use super::util_macros::*;


/// general simulator for two-dimensional code with circuit-level implementation of stabilizer measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Simulator {
    /// information of the preferred code
    pub code_type: CodeType,
    /// size of the snapshot, where `nodes` is ensured to be a cube of `height` * `vertical` * `horizontal`
    pub height: usize,
    pub vertical: usize,
    pub horizontal: usize,
    pub nodes: Vec::< Vec::< Vec::< Option<SimulatorNode> > > >,
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
    pub correlated_pauli_error_rates: Option<CorrelatedPauliErrorRates>,
    #[serde(rename = "corr_pe")]
    pub correlated_erasure_error_rates: Option<CorrelatedErasureErrorRates>,
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum GateType {
    /// initialize in $|0\rangle$ state which is the eigenstate of $\hat{Z}$
    /// , also this will randomly apply $\hat{Z}$ which is irrelevant if measuring in the correct basis
    /// (this will not be exported as error but just an internal behavior)
    InitializeZ,
    /// initialize in $|+\rangle$ state which is the eigenstate of $\hat{X}$
    /// , also this will randomly apply $\hat{X}$ which is irrelevant if measuring in the correct basis
    /// (this will not be exported as error but just an internal behavior)
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
    pub fn is_single_qubit_gate(&self) -> bool {
        self.is_initialization() || self.is_measurement() || self == &GateType::None
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
        };
        build_code(&mut simulator);
        simulator
    }

    /// this will generate an **isolated** iterator, not taking the reference of the simulator instance.
    /// you must check if the position is valid using `is_valid_position`
    pub fn position_iter(&self) -> SimulatorPositionIterator {
        SimulatorPositionIterator::new(self.height, self.vertical, self.horizontal)
    }

    /// iterate every position given specific `t`
    pub fn position_iter_t(&self, t: usize) -> SimulatorPositionIterator {
        if t >= self.height {  // null iterator
            return SimulatorPositionIterator::new(0, 0, 0);
        }
        let mut iterator = SimulatorPositionIterator::new(t + 1, self.vertical, self.horizontal);
        iterator.next_position.t = t;
        iterator
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
                node.correlated_pauli_error_rates = Some(CorrelatedPauliErrorRates::default());
            }
            if node.correlated_erasure_error_rates.is_none() {
                node.correlated_erasure_error_rates = Some(CorrelatedErasureErrorRates::default());
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
            if node.correlated_erasure_error_rates.is_none() {
                if node.correlated_erasure_error_rates.as_ref().unwrap().error_probability() == 0. {
                    node.correlated_erasure_error_rates = None;
                }
            }
        });
    }
}

/// this iterator removes the lifetime dependency of `Simulator`, offering more flexible iteration over the nodes
pub struct SimulatorPositionIterator {
    next_position: Position,
    height: usize,
    vertical: usize,
    horizontal: usize,
}

impl SimulatorPositionIterator {
    pub fn new(height: usize, vertical: usize, horizontal: usize) -> Self {
        let mut ret = Self {
            next_position: Position::new(0, 0, 0),
            height: height,
            vertical: vertical,
            horizontal: horizontal,
        };
        if height == 0 || vertical == 0 || horizontal == 0 {
            // if no iterations at all, set `next_position` to an invalid height so that it returns `None`
            ret.next_position = Position::new(height + 1, 0, 0);
        }
        ret
    }
}

impl Iterator for SimulatorPositionIterator {
    // We can refer to this type using Self::Item
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.next_position;
        if ret.t >= self.height {  // invalid position, stop here
            return None;
        }
        // update `next_position`
        self.next_position.j += 1;
        if self.next_position.j >= self.horizontal {
            self.next_position.j = 0;
            self.next_position.i += 1;
        }
        if self.next_position.i >= self.vertical {
            self.next_position.i = 0;
            self.next_position.t += 1;
        }
        Some(ret)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulator_invalid_position() {  // cargo test simulator_invalid_position -- --nocapture
        let di = 5;
        let dj = 5;
        let noisy_measurements = 5;
        let simulator = Simulator::new(CodeType::StandardPlanarCode { noisy_measurements, di, dj });
        let invalid_position = Position::new(100, 100, 100);
        assert!(!simulator.is_valid_position(&invalid_position), "invalid position");
        let nonexisting_position = Position::new(0, 0, 0);
        assert!(simulator.is_valid_position(&nonexisting_position), "valid position");
        assert!(!simulator.is_node_exist(&nonexisting_position), "nonexisting position");

    }

}
