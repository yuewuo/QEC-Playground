//! # Code Builder
//!
//! Given known a `code_type: CodeType` for a simulator, this will build the proper code.
//! It will ignore `CodeType::Customized` and leave it to user
//!
//! TODO: add svg picture to show example of different code types, see <https://docs.rs/embed-doc-image-showcase/latest/embed_doc_image_showcase/>
//! for how to embed picture in cargo doc
//! 

#[cfg(feature="python_binding")]
use super::pyo3::prelude::*;
use super::simulator::*;
use serde::{Serialize, Deserialize};
use super::types::*;
use super::util_macros::*;
use super::clap::{PossibleValue};
use ErrorType::*;


/// commonly used code type that has built-in functions to automatically build up the simulator.
/// other type of code type is also feasible, but one needs to implement the generation of code patch.
#[cfg_attr(feature = "python_binding", pyclass)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CodeType {
    ///noisy measurement rounds (excluding the final perfect measurement cap), vertical code dsitance, horizontal code distance
    StandardPlanarCode,
    /// noisy meausrement rounds (excluding the final perfect emasurement cap), +i+j axis code distance, +i-j axis code dsitance
    RotatedPlanarCode,
    /// noisy measurement rounds (excluding the final perfect measurement cap), vertical code distance, horizontal code distance
    StandardXZZXCode,
    /// noisy measurement rounds (excluding the final perfect measurement cap), +i+j axis code distance, +i-j axis code distance
    RotatedXZZXCode,
    /// noisy measurement rounds (excluding the final perfect measurement cap), vertical code distance, horizontal code distance
    StandardTailoredCode,
    /// noisy measurement rounds (excluding the final perfect measurement cap), +i+j axis code distance, +i-j axis code distance
    RotatedTailoredCode,
    /// periodic boundary condition of rotated tailored surface code, code distances must be even number
    PeriodicRotatedTailoredCode,
    /// unknown code type, user must provide necessary information and build circuit-level implementation
    Customized,
}

/// built-in code types' information
#[cfg_attr(feature = "python_binding", pyclass)]
#[derive(Debug, Serialize, Clone)]
pub struct BuiltinCodeInformation {
    pub noisy_measurements: usize,
    pub di: usize,
    pub dj: usize,
}

#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pymethods)]
impl BuiltinCodeInformation {
    #[cfg_attr(feature = "python_binding", new)]
    pub fn new(noisy_measurements: usize, di: usize, dj: usize) -> Self{
        BuiltinCodeInformation{
            noisy_measurements: noisy_measurements,
            di: di,
            dj: dj,
        }
    }
}

#[cfg_attr(feature = "python_binding", pymethods)]
impl CodeType {
    // pub fn builtin_code_information(&self) -> Option<BuiltinCodeInformation> {
    //     match &self {
    //         &CodeType::StandardPlanarCode { noisy_measurements, di, dj } | &CodeType::RotatedPlanarCode { noisy_measurements, dp: di, dn: dj } |
    //         &CodeType::StandardXZZXCode { noisy_measurements, di, dj } | &CodeType::RotatedXZZXCode { noisy_measurements, dp: di, dn: dj } |
    //         &CodeType::StandardTailoredCode { noisy_measurements, di, dj } | &CodeType::RotatedTailoredCode { noisy_measurements, dp: di, dn: dj } |
    //         &CodeType::PeriodicRotatedTailoredCode { noisy_measurements, dp: di, dn: dj } => {
    //             Some(BuiltinCodeInformation {
    //                 noisy_measurements: *noisy_measurements,
    //                 di: *di,
    //                 dj: *dj,
    //             })
    //         },
    //         _ => None
    //     }
    // }

    /// get position on the left of (i, j), note that this position may be invalid for open-boundary code if it doesn't exist
    pub fn get_left(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        match self {
            &CodeType::RotatedTailoredCode => {
                if j > 0 {
                    (i, j - 1)
                } else {
                    (i, usize::MAX)
                }
            },
            &CodeType::PeriodicRotatedTailoredCode => {
                let dp = builtin_code_information.di;
                let dn = builtin_code_information.dj;
                let (di, dj) = (dp-1, dn-1);
                if i + j == dj {
                    (i + (di + 1), j + di)
                } else if i == j + dj + 1 {
                    (i - (dj + 1), j + dj)
                } else {
                    (i, j - 1)
                }
            },
            _ => unimplemented!("left position not implemented for this code type, please fill the implementation")
        }
    }

    /// get position up the position (i, j), note that this position may be invalid for open-boundary code if it doesn't exist
    pub fn get_up(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        match self {
            &CodeType::RotatedTailoredCode => {
                if i > 0 {
                    (i - 1, j)
                } else {
                    (usize::MAX, j)
                }
            },
            &CodeType::PeriodicRotatedTailoredCode => {
                let dp = builtin_code_information.di;
                let dn = builtin_code_information.dj;
                let (di, dj) = (dp-1, dn-1);
                if i == 0 && j == dj {
                    (di + dj + 1, di)
                } else if i + j == dj {
                    (i + di, j + (di + 1))
                } else if j == i + dj {
                    (i + dj, j - (dj + 1))
                } else {
                    (i - 1, j)
                }
            },
            _ => unimplemented!("left position not implemented for this code type, please fill the implementation")
        }
    }

    /// get position on the right of (i, j), note that this position may be invalid for open-boundary code if it doesn't exist
    pub fn get_right(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        match self {
            &CodeType::RotatedTailoredCode => {
                (i, j + 1)
            },
            &CodeType::PeriodicRotatedTailoredCode => {
                let dp = builtin_code_information.di;
                let dn = builtin_code_information.dj;
                let (di, dj) = (dp-1, dn-1);
                if i + j == 2 * di + dj + 1 {
                    (i - (di + 1), j - di)
                } else if j == i + dj {
                    (i + (dj + 1), j - dj)
                } else {
                    (i, j + 1)
                }
            },
            _ => unimplemented!("left position not implemented for this code type, please fill the implementation")
        }
    }

    /// get position down the position (i, j), note that this position may be invalid for open-boundary code if it doesn't exist
    pub fn get_down(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        match self {
            &CodeType::RotatedTailoredCode => {
                (i + 1, j)
            },
            &CodeType::PeriodicRotatedTailoredCode => {
                let dp = builtin_code_information.di;
                let dn = builtin_code_information.dj;
                let (di, dj) = (dp-1, dn-1);
                if i == di + dj + 1 && j == di {
                    (0, dj)
                } else if i + j == 2 * di + dj + 1 {
                    (i - di, j - (di + 1))
                } else if i == j + dj + 1 {
                    (i - dj, j + (dj + 1))
                } else {
                    (i + 1, j)
                }
            },
            _ => unimplemented!("left position not implemented for this code type, please fill the implementation")
        }
    }

    /// convenient call to get diagonal neighbor on the left up
    pub fn get_left_up(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        let (i, j) = self.get_left(i, j, builtin_code_information);
        self.get_up(i, j, builtin_code_information)
    }

    /// convenient call to get diagonal neighbor on the left down
    pub fn get_left_down(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        let (i, j) = self.get_left(i, j, builtin_code_information);
        self.get_down(i, j, builtin_code_information)
    }

    /// convenient call to get diagonal neighbor on the right up
    pub fn get_right_up(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        let (i, j) = self.get_right(i, j, builtin_code_information);
        self.get_up(i, j, builtin_code_information)
    }

    /// convenient call to get diagonal neighbor on the left down
    pub fn get_right_down(&self, i: usize, j: usize, builtin_code_information: &BuiltinCodeInformation) -> (usize, usize) {
        let (i, j) = self.get_right(i, j, builtin_code_information);
        self.get_down(i, j, builtin_code_information)
    }
}


impl CodeType{
    pub fn new(code_type: &String) -> Self {
        match code_type.as_str() {
            "StandardPlanarCode" => Self::StandardPlanarCode,
            "RotatedPlanarCode" => Self::RotatedPlanarCode,
            "StandardTailoredCode" => Self::StandardTailoredCode,
            "RotatedTailoredCode" => Self::RotatedTailoredCode,
            "PeriodicRotatedTailoredCode" => Self::PeriodicRotatedTailoredCode,
            "StandardXZZXCode" => Self::StandardXZZXCode,
            "RotatedXZZXCode" => Self::RotatedXZZXCode,
            _ => unimplemented!()
        }
    }    

    pub fn possible_values<'a>() -> impl Iterator<Item = PossibleValue<'a>> {
        static VARIANTS: &'static [&str] = &[
            "StandardPlanarCode", "RotatedPlanarCode", "StandardTailoredCode", "RotatedTailoredCode", "PeriodicRotatedTailoredCode", "StandardXZZXCode", "RotatedXZZXCode"
        ];
        VARIANTS.iter().map(|x| PossibleValue::new(x))
    }
}

pub fn build_code(simulator: &mut Simulator) {
    let code_type = &simulator.code_type;
    let builtin_code_information = &simulator.builtin_code_information;
    match code_type {
        &CodeType::StandardPlanarCode| &CodeType::RotatedPlanarCode => {
            let di = builtin_code_information.di;
            let dj = builtin_code_information.dj;
            let noisy_measurements = builtin_code_information.noisy_measurements;
            simulator.measurement_cycles = 6;
            assert!(di > 0, "code distance must be positive integer");
            assert!(dj > 0, "code distance must be positive integer");
            let is_rotated = matches!(code_type, CodeType::RotatedPlanarCode { .. });
            if is_rotated {
                assert!(di % 2 == 1, "code distance must be odd integer, current: di = {}", di);
                assert!(dj % 2 == 1, "code distance must be odd integer, current: dj = {}", dj);
            }
            // println!("noisy_measurements: {}, di: {}, dj: {}, is_rotated: {}", noisy_measurements, di, dj, is_rotated);
            let (vertical, horizontal) = if is_rotated {
                (di + dj + 1, di + dj + 1)
            } else {
                (2 * di + 1, 2 * dj + 1)
            };
            let height = simulator.measurement_cycles * (noisy_measurements + 1) + 1;
            // each measurement takes 6 time steps
            let mut nodes = Vec::with_capacity(height);
            let is_real = |i: usize, j: usize| -> bool {
                if is_rotated {
                    let is_real_dj = |pi, pj| { pi + pj < dj || (pi + pj == dj && pi % 2 == 0 && pi > 0) };
                    let is_real_di = |pi, pj| { pi + pj < di || (pi + pj == di && pj % 2 == 0 && pj > 0) };
                    if i <= dj && j <= dj {
                        is_real_dj(dj - i, dj - j)
                    } else if i >= di && j >= di {
                        is_real_dj(i - di, j - di)
                    } else if i >= dj && j <= di {
                        is_real_di(i - dj, di - j)
                    } else if i <= di && j >= dj {
                        is_real_di(di - i, j - dj)
                    } else {
                        unreachable!()
                    }
                } else {
                    i > 0 && j > 0 && i < vertical - 1 && j < horizontal - 1
                }
            };
            let is_virtual = |i: usize, j: usize| -> bool {
                if is_rotated {
                    let is_virtual_dj = |pi, pj| { pi + pj == dj && (pi % 2 == 1 || pi == 0) };
                    let is_virtual_di = |pi, pj| { pi + pj == di && (pj % 2 == 1 || pj == 0) };
                    if i <= dj && j <= dj {
                        is_virtual_dj(dj - i, dj - j)
                    } else if i >= di && j >= di {
                        is_virtual_dj(i - di, j - di)
                    } else if i >= dj && j <= di {
                        is_virtual_di(i - dj, di - j)
                    } else if i <= di && j >= dj {
                        is_virtual_di(di - i, j - dj)
                    } else {
                        unreachable!()
                    }
                } else {
                    if i == 0 || i == vertical - 1 {
                        j % 2 == 1
                    } else if j == 0 || j == horizontal - 1 {
                        i % 2 == 1
                    } else {
                        false
                    }
                }
            };
            let is_present = |i: usize, j: usize| -> bool {
                let is_this_real = is_real(i, j);
                let is_this_virtual = is_virtual(i, j);
                assert!(!(is_this_real && is_this_virtual), "a position cannot be both real and virtual");
                is_this_real || is_this_virtual
            };
            for t in 0..height {
                let mut row_i = Vec::with_capacity(vertical);
                for i in 0..vertical {
                    let mut row_j = Vec::with_capacity(horizontal);
                    for j in 0..horizontal {
                        if is_present(i, j) {
                            let qubit_type = if (i + j) % 2 == 0 {
                                assert!(is_real(i, j), "data qubits should not be virtual");
                                QubitType::Data
                            } else { if i % 2 == 1 { QubitType::StabZ } else { QubitType::StabX } };
                            let mut gate_type = GateType::None;
                            let mut gate_peer = None;
                            match t % simulator.measurement_cycles {
                                1 => {  // initialization
                                    match qubit_type {
                                        QubitType::StabZ => { gate_type = GateType::InitializeZ; }
                                        QubitType::StabX => { gate_type = GateType::InitializeX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                2 => {  // gate 1
                                    if qubit_type == QubitType::Data {
                                        if i+1 < vertical && is_present(i+1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateTarget } else { GateType::CXGateControl };
                                            gate_peer = Some(pos!(t, i+1, j));
                                        }
                                    } else {
                                        if i >= 1 && is_present(i-1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateControl } else { GateType::CXGateTarget };
                                            gate_peer = Some(pos!(t, i-1, j));
                                        }
                                    }
                                },
                                3 => {  // gate 2
                                    if j % 2 == 1 {  // operate with right
                                        if is_present(i, j+1) {
                                            gate_type = GateType::CXGateControl;
                                            gate_peer = Some(pos!(t, i, j+1));
                                        }
                                    } else {  // operate with left
                                        if j >= 1 && is_present(i, j-1) {
                                            gate_type = GateType::CXGateTarget;
                                            gate_peer = Some(pos!(t, i, j-1));
                                        }
                                    }
                                },
                                4 => {  // gate 3
                                    if j % 2 == 1 {  // operate with left
                                        if j >= 1 && is_present(i, j-1) {
                                            gate_type = GateType::CXGateControl;
                                            gate_peer = Some(pos!(t, i, j-1));
                                        }
                                    } else {  // operate with right
                                        if is_present(i, j+1) {
                                            gate_type = GateType::CXGateTarget;
                                            gate_peer = Some(pos!(t, i, j+1));
                                        }
                                    }
                                },
                                5 => {  // gate 4
                                    if qubit_type == QubitType::Data {
                                        if i >= 1 && is_present(i-1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateTarget } else { GateType::CXGateControl };
                                            gate_peer = Some(pos!(t, i-1, j));
                                        }
                                    } else {
                                        if i+1 < vertical && is_present(i+1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateControl } else { GateType::CXGateTarget };
                                            gate_peer = Some(pos!(t, i+1, j));
                                        }
                                    }
                                },
                                0 => {  // measurement
                                    match qubit_type {
                                        QubitType::StabZ => { gate_type = GateType::MeasureZ; }
                                        QubitType::StabX => { gate_type = GateType::MeasureX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                _ => unreachable!()
                            }
                            row_j.push(Some(Box::new(SimulatorNode::new(qubit_type, gate_type, gate_peer.clone()).set_virtual(
                                is_virtual(i, j), gate_peer.map_or(false, |peer| is_virtual(peer.i, peer.j))))));
                        } else {
                            row_j.push(None);
                        }
                    }
                    row_i.push(row_j);
                }
                nodes.push(row_i)
            }
            simulator.vertical = vertical;
            simulator.horizontal = horizontal;
            simulator.height = height;
            simulator.nodes = nodes;
        },
        &CodeType::StandardTailoredCode | &CodeType::RotatedTailoredCode => {
            let di = builtin_code_information.di;
            let dj = builtin_code_information.dj;
            let noisy_measurements = builtin_code_information.noisy_measurements;
            simulator.measurement_cycles = 6;
            assert!(di > 0, "code distance must be positive integer");
            assert!(dj > 0, "code distance must be positive integer");
            let is_rotated = matches!(code_type, CodeType::RotatedTailoredCode { .. });
            if is_rotated {
                assert!(di % 2 == 1, "code distance must be odd integer, current: di = {}", di);
                assert!(dj % 2 == 1, "code distance must be odd integer, current: dj = {}", dj);
            }
            // println!("noisy_measurements: {}, di: {}, dj: {}, is_rotated: {}", noisy_measurements, di, dj, is_rotated);
            let (vertical, horizontal) = if is_rotated {
                (di + dj + 1, di + dj + 1)
            } else {
                (2 * di + 1, 2 * dj + 1)
            };
            let height = simulator.measurement_cycles * (noisy_measurements + 1) + 1;
            // each measurement takes 6 time steps
            let mut nodes = Vec::with_capacity(height);
            let is_real = |i: usize, j: usize| -> bool {
                if is_rotated {
                    let is_real_dj = |pi, pj| { pi + pj < dj || (pi + pj == dj && pi % 2 == 0 && pi > 0) };
                    let is_real_di = |pi, pj| { pi + pj < di || (pi + pj == di && pj % 2 == 0 && pj > 0) };
                    if i <= dj && j <= dj {
                        is_real_dj(dj - i, dj - j)
                    } else if i >= di && j >= di {
                        is_real_dj(i - di, j - di)
                    } else if i >= dj && j <= di {
                        is_real_di(i - dj, di - j)
                    } else if i <= di && j >= dj {
                        is_real_di(di - i, j - dj)
                    } else {
                        unreachable!()
                    }
                } else {
                    i > 0 && j > 0 && i < vertical - 1 && j < horizontal - 1
                }
            };
            let is_virtual = |i: usize, j: usize| -> bool {
                if is_rotated {
                    let is_virtual_dj = |pi, pj| { pi + pj == dj && (pi % 2 == 1 || pi == 0) };
                    let is_virtual_di = |pi, pj| { pi + pj == di && (pj % 2 == 1 || pj == 0) };
                    if i <= dj && j <= dj {
                        is_virtual_dj(dj - i, dj - j)
                    } else if i >= di && j >= di {
                        is_virtual_dj(i - di, j - di)
                    } else if i >= dj && j <= di {
                        is_virtual_di(i - dj, di - j)
                    } else if i <= di && j >= dj {
                        is_virtual_di(di - i, j - dj)
                    } else {
                        unreachable!()
                    }
                } else {
                    if i == 0 || i == vertical - 1 {
                        j % 2 == 1
                    } else if j == 0 || j == horizontal - 1 {
                        i % 2 == 1
                    } else {
                        false
                    }
                }
            };
            let is_present = |i: usize, j: usize| -> bool {
                let is_this_real = is_real(i, j);
                let is_this_virtual = is_virtual(i, j);
                assert!(!(is_this_real && is_this_virtual), "a position cannot be both real and virtual");
                is_this_real || is_this_virtual
            };
            for t in 0..height {
                let mut row_i = Vec::with_capacity(vertical);
                for i in 0..vertical {
                    let mut row_j = Vec::with_capacity(horizontal);
                    for j in 0..horizontal {
                        if is_present(i, j) {
                            let qubit_type = if (i + j) % 2 == 0 {
                                assert!(is_real(i, j), "data qubits should not be virtual");
                                QubitType::Data
                            } else { if i % 2 == 1 { QubitType::StabY } else { QubitType::StabX } };
                            let mut gate_type = GateType::None;
                            let mut gate_peer = None;
                            // see residual decoding of https://journals.aps.org/prl/abstract/10.1103/PhysRevLett.124.130501
                            let (is_corner, peer_corner): (bool, Option<Position>) = if is_rotated {
                                if i == 0 && j == dj {
                                    (true, Some(pos!(t, 1, dj+1)))
                                } else if j == 0 && i == dj {
                                    (true, Some(pos!(t, dj-1, 1)))
                                } else if i == vertical-1 && j == di {
                                    (true, Some(pos!(t, vertical-2, di-1)))
                                } else if i == di && j == vertical-1 {
                                    (true, Some(pos!(t, di+1, vertical-2)))
                                } else {
                                    (false, None)
                                }
                            } else {
                                if i == 0 && j == 1 {
                                    (true, Some(pos!(t, 1, 0)))
                                } else if i == 1 && j == horizontal-1 {
                                    (true, Some(pos!(t, 0, horizontal-2)))
                                } else if i == vertical-2 && j == 0 {
                                    (true, Some(pos!(t, vertical-1, 1)))
                                } else if i == vertical-1 && j == horizontal-2 {
                                    (true, Some(pos!(t, vertical-2, horizontal-1)))
                                } else {
                                    (false, None)
                                }
                            };
                            match t % simulator.measurement_cycles {
                                1 => {  // initialization
                                    match qubit_type {
                                        QubitType::StabY => { gate_type = GateType::InitializeX; }
                                        QubitType::StabX => { gate_type = GateType::InitializeX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                2 => {  // gate 1
                                    if qubit_type == QubitType::Data {
                                        if i+1 < vertical && is_present(i+1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateTarget } else { GateType::CYGateTarget };
                                            gate_peer = Some(pos!(t, i+1, j));
                                        }
                                    } else {
                                        if i >= 1 && is_present(i-1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateControl } else { GateType::CYGateControl };
                                            gate_peer = Some(pos!(t, i-1, j));
                                        }
                                    }
                                },
                                3 => {  // gate 2
                                    if j % 2 == 1 {  // operate with right
                                        if is_present(i, j+1) {
                                            gate_type = if qubit_type == QubitType::Data { GateType::CYGateTarget } else { GateType::CXGateControl };
                                            gate_peer = Some(pos!(t, i, j+1));
                                        }
                                    } else {  // operate with left
                                        if j >= 1 && is_present(i, j-1) {
                                            gate_type = if qubit_type == QubitType::Data { GateType::CXGateTarget } else { GateType::CYGateControl };
                                            gate_peer = Some(pos!(t, i, j-1));
                                        }
                                    }
                                },
                                4 => {  // gate 3
                                    if j % 2 == 1 {  // operate with left
                                        if j >= 1 && is_present(i, j-1) {
                                            gate_type = if qubit_type == QubitType::Data { GateType::CYGateTarget } else { GateType::CXGateControl };
                                            gate_peer = Some(pos!(t, i, j-1));
                                        }
                                    } else {  // operate with right
                                        if is_present(i, j+1) {
                                            gate_type = if qubit_type == QubitType::Data { GateType::CXGateTarget } else { GateType::CYGateControl };
                                            gate_peer = Some(pos!(t, i, j+1));
                                        }
                                    }
                                },
                                5 => {  // gate 4
                                    if qubit_type == QubitType::Data {
                                        if i >= 1 && is_present(i-1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateTarget } else { GateType::CYGateTarget };
                                            gate_peer = Some(pos!(t, i-1, j));
                                        }
                                    } else {
                                        if i+1 < vertical && is_present(i+1, j) {
                                            gate_type = if j % 2 == 1 { GateType::CXGateControl } else { GateType::CYGateControl };
                                            gate_peer = Some(pos!(t, i+1, j));
                                        }
                                    }
                                },
                                0 => {  // measurement
                                    match qubit_type {
                                        QubitType::StabY => { gate_type = GateType::MeasureX; }
                                        QubitType::StabX => { gate_type = GateType::MeasureX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                _ => unreachable!()
                            }
                            row_j.push(Some(Box::new(SimulatorNode::new(qubit_type, gate_type, gate_peer.clone())
                                .set_virtual(is_virtual(i, j), gate_peer.map_or(false, |peer| is_virtual(peer.i, peer.j)))
                                .with_miscellaneous(if is_corner { Some(json!({ "is_corner": true, "peer_corner": peer_corner.unwrap() })) } else { None }))));
                        } else {
                            row_j.push(None);
                        }
                    }
                    row_i.push(row_j);
                }
                nodes.push(row_i)
            }
            simulator.vertical = vertical;
            simulator.horizontal = horizontal;
            simulator.height = height;
            simulator.nodes = nodes;
        },
        &CodeType::PeriodicRotatedTailoredCode => {
            let dp = builtin_code_information.di;
            let dn = builtin_code_information.dj;
            let noisy_measurements = builtin_code_information.noisy_measurements;
            simulator.measurement_cycles = 6;
            assert!(dp > 0, "code distance must be positive integer");
            assert!(dn > 0, "code distance must be positive integer");
            assert!(dp % 2 == 0, "code distance must be even integer, current: dp = {}", dp);
            assert!(dn % 2 == 0, "code distance must be even integer, current: dn = {}", dn);
            // println!("noisy_measurements: {}, dp: {}, dn: {}, is_rotated: {}", noisy_measurements, dp, dn);
            let di = dp - 1;  // to use previously developed functions
            let dj = dn - 1;
            let (vertical, horizontal) = (di + dj + 2, di + dj + 1);
            let height = simulator.measurement_cycles * (noisy_measurements + 1) + 1;
            // each measurement takes 6 time steps
            let mut nodes = Vec::with_capacity(height);
            let is_present = |i: usize, j: usize| -> bool {
                let is_present_dj = |pi, pj| { pi + pj <= dj };
                let is_present_di = |pi, pj| { pi + pj <= di };
                let presented = if i <= dj && j <= dj {
                    is_present_dj(dj - i, dj - j)
                } else if i >= di && j >= di {
                    is_present_dj(i - di, j - di)
                } else if i >= dj && j <= di {
                    is_present_di(i - dj, di - j)
                } else if i <= di && j >= dj {
                    is_present_di(di - i, j - dj)
                } else {
                    unreachable!()
                };
                presented || i == j + dj + 1 || i + j == 2 * di + dj + 1
            };
            for t in 0..height {
                let mut row_i = Vec::with_capacity(vertical);
                for i in 0..vertical {
                    let mut row_j = Vec::with_capacity(horizontal);
                    for j in 0..horizontal {
                        if is_present(i, j) {
                            let qubit_type = if (i + j) % 2 == 0 { QubitType::Data } else { if i % 2 == 1 { QubitType::StabY } else { QubitType::StabX } };
                            let mut gate_type = GateType::None;
                            let mut gate_peer = None;
                            match t % simulator.measurement_cycles {
                                1 => {  // initialization
                                    match qubit_type {
                                        QubitType::StabY => { gate_type = GateType::InitializeX; }
                                        QubitType::StabX => { gate_type = GateType::InitializeX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                2 => {  // gate 1
                                    if qubit_type == QubitType::Data {
                                        let (pi, pj) = code_type.get_down(i, j, builtin_code_information);
                                        gate_type = if j % 2 == 1 { GateType::CXGateTarget } else { GateType::CYGateTarget };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    } else {
                                        let (pi, pj) = code_type.get_up(i, j, builtin_code_information);
                                        gate_type = if j % 2 == 1 { GateType::CXGateControl } else { GateType::CYGateControl };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    }
                                },
                                3 => {  // gate 2
                                    if j % 2 == 1 {  // operate with right
                                        let (pi, pj) = code_type.get_right(i, j, builtin_code_information);
                                        gate_type = if qubit_type == QubitType::Data { GateType::CYGateTarget } else { GateType::CXGateControl };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    } else {  // operate with left
                                        let (pi, pj) = code_type.get_left(i, j, builtin_code_information);
                                        gate_type = if qubit_type == QubitType::Data { GateType::CXGateTarget } else { GateType::CYGateControl };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    }
                                },
                                4 => {  // gate 3
                                    if j % 2 == 1 {  // operate with left
                                        let (pi, pj) = code_type.get_left(i, j, builtin_code_information);
                                        gate_type = if qubit_type == QubitType::Data { GateType::CYGateTarget } else { GateType::CXGateControl };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    } else {  // operate with right
                                        let (pi, pj) = code_type.get_right(i, j, builtin_code_information);
                                        gate_type = if qubit_type == QubitType::Data { GateType::CXGateTarget } else { GateType::CYGateControl };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    }
                                },
                                5 => {  // gate 4
                                    if qubit_type == QubitType::Data {
                                        let (pi, pj) = code_type.get_up(i, j, builtin_code_information);
                                        gate_type = if j % 2 == 1 { GateType::CXGateTarget } else { GateType::CYGateTarget };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    } else {
                                        let (pi, pj) = code_type.get_down(i, j, builtin_code_information);
                                        gate_type = if j % 2 == 1 { GateType::CXGateControl } else { GateType::CYGateControl };
                                        gate_peer = Some(pos!(t, pi, pj));
                                    }
                                },
                                0 => {  // measurement
                                    match qubit_type {
                                        QubitType::StabY => { gate_type = GateType::MeasureX; }
                                        QubitType::StabX => { gate_type = GateType::MeasureX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                _ => unreachable!()
                            }
                            row_j.push(Some(Box::new(SimulatorNode::new(qubit_type, gate_type, gate_peer.clone()))));
                        } else {
                            row_j.push(None);
                        }
                    }
                    row_i.push(row_j);
                }
                nodes.push(row_i)
            }
            simulator.vertical = vertical;
            simulator.horizontal = horizontal;
            simulator.height = height;
            simulator.nodes = nodes;
        },
        CodeType::Customized => {
            // skip user customized code
        },
        &CodeType::StandardXZZXCode | &CodeType::RotatedXZZXCode => {
            let di = builtin_code_information.di;
            let dj = builtin_code_information.dj;
            let noisy_measurements = builtin_code_information.noisy_measurements;
            simulator.measurement_cycles = 6;
            assert!(di > 0, "code distance must be positive integer");
            assert!(dj > 0, "code distance must be positive integer");
            let is_rotated = matches!(code_type, CodeType::RotatedPlanarCode { .. });
            if is_rotated {
                assert!(di % 2 == 1, "code distance must be odd integer, current: di = {}", di);
                assert!(dj % 2 == 1, "code distance must be odd integer, current: dj = {}", dj);
            }
            // println!("noisy_measurements: {}, di: {}, dj: {}, is_rotated: {}", noisy_measurements, di, dj, is_rotated);
            let (vertical, horizontal) = if is_rotated {
                (di + dj + 1, di + dj + 1)
            } else {
                (2 * di + 1, 2 * dj + 1)
            };
            let height = simulator.measurement_cycles * (noisy_measurements + 1) + 1;
            // each measurement takes 6 time steps
            let mut nodes = Vec::with_capacity(height);
            let is_real = |i: usize, j: usize| -> bool {
                if is_rotated {
                    let is_real_dj = |pi, pj| { pi + pj < dj || (pi + pj == dj && pi % 2 == 0 && pi > 0) };
                    let is_real_di = |pi, pj| { pi + pj < di || (pi + pj == di && pj % 2 == 0 && pj > 0) };
                    if i <= dj && j <= dj {
                        is_real_dj(dj - i, dj - j)
                    } else if i >= di && j >= di {
                        is_real_dj(i - di, j - di)
                    } else if i >= dj && j <= di {
                        is_real_di(i - dj, di - j)
                    } else if i <= di && j >= dj {
                        is_real_di(di - i, j - dj)
                    } else {
                        unreachable!()
                    }
                } else {
                    i > 0 && j > 0 && i < vertical - 1 && j < horizontal - 1
                }
            };
            let is_virtual = |i: usize, j: usize| -> bool {
                if is_rotated {
                    let is_virtual_dj = |pi, pj| { pi + pj == dj && (pi % 2 == 1 || pi == 0) };
                    let is_virtual_di = |pi, pj| { pi + pj == di && (pj % 2 == 1 || pj == 0) };
                    if i <= dj && j <= dj {
                        is_virtual_dj(dj - i, dj - j)
                    } else if i >= di && j >= di {
                        is_virtual_dj(i - di, j - di)
                    } else if i >= dj && j <= di {
                        is_virtual_di(i - dj, di - j)
                    } else if i <= di && j >= dj {
                        is_virtual_di(di - i, j - dj)
                    } else {
                        unreachable!()
                    }
                } else {
                    if i == 0 || i == vertical - 1 {
                        j % 2 == 1
                    } else if j == 0 || j == horizontal - 1 {
                        i % 2 == 1
                    } else {
                        false
                    }
                }
            };
            let is_present = |i: usize, j: usize| -> bool {
                let is_this_real = is_real(i, j);
                let is_this_virtual = is_virtual(i, j);
                assert!(!(is_this_real && is_this_virtual), "a position cannot be both real and virtual");
                is_this_real || is_this_virtual
            };
            for t in 0..height {
                let mut row_i = Vec::with_capacity(vertical);
                for i in 0..vertical {
                    let mut row_j = Vec::with_capacity(horizontal);
                    for j in 0..horizontal {
                        if is_present(i, j) {
                            let qubit_type = if (i + j) % 2 == 0 {
                                assert!(is_real(i, j), "data qubits should not be virtual");
                                QubitType::Data
                            } else { if i % 2 == 1 { QubitType::StabXZZXLogicalZ } else { QubitType::StabXZZXLogicalX } };
                            let mut gate_type = GateType::None;
                            let mut gate_peer = None;
                            match t % simulator.measurement_cycles {
                                1 => {  // initialization
                                    match qubit_type {
                                        QubitType::StabXZZXLogicalZ => { gate_type = GateType::InitializeX; }
                                        QubitType::StabXZZXLogicalX => { gate_type = GateType::InitializeX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                2 => {  // gate 1
                                    if qubit_type == QubitType::Data {
                                        if i+1 < vertical && is_present(i+1, j) {
                                            gate_type = GateType::CZGate;
                                            gate_peer = Some(pos!(t, i+1, j));
                                        }
                                    } else {
                                        if i >= 1 && is_present(i-1, j) {
                                            gate_type = GateType::CZGate;
                                            gate_peer = Some(pos!(t, i-1, j));
                                        }
                                    }
                                },
                                3 => {  // gate 2
                                    if qubit_type == QubitType::Data {
                                        if j+1 < horizontal && is_present(i, j+1) {
                                            gate_type = GateType::CXGateTarget;
                                            gate_peer = Some(pos!(t, i, j+1));
                                        }
                                    } else {
                                        if j >= 1 && is_present(i, j-1) {
                                            gate_type = GateType::CXGateControl;
                                            gate_peer = Some(pos!(t, i, j-1));
                                        }
                                    }
                                },
                                4 => {  // gate 3
                                    if qubit_type == QubitType::Data {
                                        if j >= 1 && is_present(i, j-1) {
                                            gate_type = GateType::CXGateTarget;
                                            gate_peer = Some(pos!(t, i, j-1));
                                        }
                                    } else {
                                        if j+1 < horizontal && is_present(i, j+1) {
                                            gate_type = GateType::CXGateControl;
                                            gate_peer = Some(pos!(t, i, j+1));
                                        }
                                    }
                                },
                                5 => {  // gate 4
                                    if qubit_type == QubitType::Data {
                                        if i >= 1 && is_present(i-1, j) {
                                            gate_type = GateType::CZGate;
                                            gate_peer = Some(pos!(t, i-1, j));
                                        }
                                    } else {
                                        if i+1 < vertical && is_present(i+1, j) {
                                            gate_type = GateType::CZGate;
                                            gate_peer = Some(pos!(t, i+1, j));
                                        }
                                    }
                                },
                                0 => {  // measurement
                                    match qubit_type {
                                        QubitType::StabXZZXLogicalZ => { gate_type = GateType::MeasureX; }
                                        QubitType::StabXZZXLogicalX => { gate_type = GateType::MeasureX; }
                                        QubitType::Data => { }
                                        _ => { unreachable!() }
                                    }
                                },
                                _ => unreachable!()
                            }
                            row_j.push(Some(Box::new(SimulatorNode::new(qubit_type, gate_type, gate_peer.clone()).set_virtual(
                                is_virtual(i, j), gate_peer.map_or(false, |peer| is_virtual(peer.i, peer.j))))));
                        } else {
                            row_j.push(None);
                        }
                    }
                    row_i.push(row_j);
                }
                nodes.push(row_i)
            }
            simulator.vertical = vertical;
            simulator.horizontal = horizontal;
            simulator.height = height;
            simulator.nodes = nodes;
        },
    }
}

/// detect common bugs of code building, e.g. peer gate invalid type, is_virtual not correct, etc...
pub fn code_builder_sanity_check(simulator: &Simulator) -> Result<(), String> {
    simulator_iter!(simulator, position, node, {
        // println!("{}", node);
        if node.qubit_type == QubitType::Data {
            if node.gate_type.is_initialization() {
                return Err(format!("data qubit at {} cannot be initialized: gate_type = {:?}", position, node.gate_type))
            }
            if node.gate_type.is_measurement() {
                return Err(format!("data qubit at {} cannot be initialized: gate_type = {:?}", position, node.gate_type))
            }
        }
        match node.gate_peer.as_ref() {
            Some(peer_position) => {
                if node.gate_type.is_single_qubit_gate() {
                    return Err(format!("{} has single qubit gate {:?} should not have peer", position, node.gate_type))
                }
                if !simulator.is_node_exist(&peer_position) {
                    return Err(format!("{}'s peer not exist: {}", position, peer_position))
                }
                let peer_node = simulator.get_node_unwrap(&peer_position);
                match &peer_node.gate_peer {
                    Some(peer_peer_position) => {
                        if peer_peer_position.as_ref() != position {
                            return Err(format!("{}, as the peer of {}, doesn't have correct peer but {}", peer_position, position, peer_peer_position))
                        }
                        if peer_node.gate_type.is_single_qubit_gate() {
                            return Err(format!("{}, as the peer of {}, doesn't have two-qubit gate", peer_position, position))
                        }
                        if node.gate_type.peer_gate() != peer_node.gate_type {
                            return Err(format!("{}, as the peer of {}, doesn't have correct peer gate {:?}, the correct one should be {:?}"
                                , peer_position, position, node.gate_type.peer_gate(), peer_node.gate_type))
                        }
                    },
                    None => {
                        return Err(format!("{}, as the peer of {}, doesn't have peer which is invalid", peer_position, position))
                    }
                }
            }, 
            None => {
                if !node.gate_type.is_single_qubit_gate() {
                    return Err(format!("two qubit gate {:?} should have peer", node.gate_type))
                }
            }
        }
    });
    simulator_iter!(simulator, base_position, _base_node, t => 0, {
        // check that initialization and measurement are always in the same basis
        let mut previous_initialization = GateType::None;
        for t in 1..simulator.height {
            let position = &mut base_position.clone();
            position.t = t;
            let node = simulator.get_node_unwrap(position);
            if node.gate_type.is_initialization() {
                previous_initialization = node.gate_type;
            }
            if node.gate_type.is_measurement() {
                if !node.gate_type.is_corresponding_initialization(&previous_initialization) {
                    return Err(format!("measurement and initialization not in the same basis: node {} has gate type {:?} but previous initialization is {:?}"
                        , position, node.gate_type, previous_initialization))
                }
            }
        }
    });
    Ok(())
}

pub fn code_builder_validate_correction(simulator: &mut Simulator, correction: &SparseCorrection) -> Option<(bool, bool)> {
    // apply the correction directly to the top layer
    let top_t = simulator.height - 1;
    for (position, error) in correction.iter() {
        assert_eq!(position.t, top_t, "correction pattern must only be at top layer");
        let node = simulator.get_node_mut_unwrap(position);
        node.propagated = node.propagated.multiply(error);
    }
    // validate the result
    let code_type = &simulator.code_type;
    let builtin_code_information = &simulator.builtin_code_information;
    let result = match code_type {
        &CodeType::StandardPlanarCode => {
            // check cardinality of top boundary for logical_i
            let mut top_cardinality = 0;
            for j in (1..simulator.horizontal).step_by(2) {
                let node = simulator.get_node_unwrap(&pos!(top_t, 1, j));
                if node.propagated == Z || node.propagated == Y {
                    top_cardinality += 1;
                }
            }
            let logical_i = top_cardinality % 2 != 0;  // odd cardinality means there is a logical Z error
            // check cardinality of left boundary for logical_j
            let mut left_cardinality = 0;
            for i in (1..simulator.vertical).step_by(2) {
                let node = simulator.get_node_unwrap(&pos!(top_t, i, 1));
                if node.propagated == X || node.propagated == Y {
                    left_cardinality += 1;
                }
            }
            let logical_j = left_cardinality % 2 != 0;  // odd cardinality means there is a logical X error
            Some((logical_i, logical_j))
        },
        &CodeType::RotatedPlanarCode => {
            // check cardinality of top boundary for logical_i
            let dp = builtin_code_information.di;
            let dn = builtin_code_information.dj;
            let mut top_cardinality = 0;
            for delta in 0..dn {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn-delta, 1+delta));
                if node.propagated == Z || node.propagated == Y {
                    top_cardinality += 1;
                }
            }
            let logical_p = top_cardinality % 2 != 0;  // odd cardinality means there is a logical Z error
            // check cardinality of left boundary for logical_j
            let mut left_cardinality = 0;
            for delta in 0..dp {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn+delta, 1+delta));
                if node.propagated == X || node.propagated == Y {
                    left_cardinality += 1;
                }
            }
            let logical_n = left_cardinality % 2 != 0;  // odd cardinality means there is a logical X error
            Some((logical_p, logical_n))
        },
        &CodeType::StandardTailoredCode => {
            // check cardinality of top boundary for logical_i
            let mut top_cardinality = 0;
            for j in (1..simulator.horizontal).step_by(2) {
                let node = simulator.get_node_unwrap(&pos!(top_t, 1, j));
                if node.propagated == Y || node.propagated == Z {
                    top_cardinality += 1;
                }
            }
            let logical_i = top_cardinality % 2 != 0;  // odd cardinality means there is a logical Z error
            // check cardinality of left boundary for logical_j
            let mut left_cardinality = 0;
            for i in (1..simulator.vertical).step_by(2) {
                let node = simulator.get_node_unwrap(&pos!(top_t, i, 1));
                if node.propagated == X || node.propagated == Z {
                    left_cardinality += 1;
                }
            }
            let logical_j = left_cardinality % 2 != 0;  // odd cardinality means there is a logical X error
            Some((logical_i, logical_j))
        },
        &CodeType::RotatedTailoredCode => {
            // check cardinality of top boundary for logical_i
            let dp = builtin_code_information.di;
            let dn = builtin_code_information.dj;
            let mut top_cardinality = 0;
            for delta in 0..dn {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn-delta, 1+delta));
                if node.propagated == Y || node.propagated == Z {
                    top_cardinality += 1;
                }
            }
            let logical_p = top_cardinality % 2 != 0;  // odd cardinality means there is a logical Z error
            // check cardinality of left boundary for logical_j
            let mut left_cardinality = 0;
            for delta in 0..dp {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn+delta, 1+delta));
                if node.propagated == X || node.propagated == Z {
                    left_cardinality += 1;
                }
            }
            let logical_n = left_cardinality % 2 != 0;  // odd cardinality means there is a logical X error
            Some((logical_p, logical_n))
        },
        &CodeType::PeriodicRotatedTailoredCode => {
            let dp = builtin_code_information.di;
            let dn = builtin_code_information.dj;
            // check cardinality of top boundary for logical_i
            let mut top_cardinality_y = 0;
            let mut top_cardinality_x = 0;
            for delta in 0..dn {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn-delta, delta));
                if node.propagated == Y || node.propagated == Z {
                    top_cardinality_y += 1;
                }
                if node.propagated == X || node.propagated == Z {
                    top_cardinality_x += 1;
                }
            }
            // check cardinality of left boundary for logical_j
            let mut left_cardinality_y = 0;
            let mut left_cardinality_x = 0;
            for delta in 0..dp {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn+delta, delta));
                if node.propagated == Y || node.propagated == Z {
                    left_cardinality_y += 1;
                }
                if node.propagated == X || node.propagated == Z {
                    left_cardinality_x += 1;
                }
            }
            // odd cardinality means there is a logical error; there are two logical qubits so either error
            let logical_p = top_cardinality_y % 2 != 0 || left_cardinality_y % 2 != 0;
            let logical_n = top_cardinality_x % 2 != 0 || left_cardinality_x % 2 != 0;
            Some((logical_p, logical_n))
        },
        &CodeType::StandardXZZXCode => {
            // check cardinality of top boundary for logical_i
            let mut top_cardinality = 0;
            for j in (1..simulator.horizontal).step_by(2) {
                let node = simulator.get_node_unwrap(&pos!(top_t, 1, j));
                if node.propagated == X || node.propagated == Y {
                    top_cardinality += 1;
                }
            }
            let logical_i = top_cardinality % 2 != 0;  // odd cardinality means there is a logical Z error
            // check cardinality of left boundary for logical_j
            let mut left_cardinality = 0;
            for i in (1..simulator.vertical).step_by(2) {
                let node = simulator.get_node_unwrap(&pos!(top_t, i, 1));
                if node.propagated == Z || node.propagated == Y {
                    left_cardinality += 1;
                }
            }
            let logical_j = left_cardinality % 2 != 0;  // odd cardinality means there is a logical X error
            Some((logical_i, logical_j))
        },
        &CodeType::RotatedXZZXCode => {
            let dp = builtin_code_information.di;
            let dn = builtin_code_information.dj;
            // check cardinality of top boundary for logical_i
            let mut top_cardinality = 0;
            for delta in 0..dn {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn-delta, 1+delta));
                if node.propagated == X || node.propagated == Y {
                    top_cardinality += 1;
                }
            }
            let logical_p = top_cardinality % 2 != 0;  // odd cardinality means there is a logical Z error
            // check cardinality of left boundary for logical_j
            let mut left_cardinality = 0;
            for delta in 0..dp {
                let node = simulator.get_node_unwrap(&pos!(top_t, dn+delta, 1+delta));
                if node.propagated == Z || node.propagated == Y {
                    left_cardinality += 1;
                }
            }
            let logical_n = left_cardinality % 2 != 0;  // odd cardinality means there is a logical X error
            Some((logical_p, logical_n))
        },
        _ => None
    };
    // recover the errors
    for (position, error) in correction.iter() {
        let node = simulator.get_node_mut_unwrap(position);
        node.propagated = node.propagated.multiply(error);
    }
    result
}

/// check if correction indeed recover all stabilizer measurements (this is expensive for runtime)
#[allow(dead_code)]
pub fn code_builder_sanity_check_correction(simulator: &mut Simulator, correction: &SparseCorrection) -> Result<(), Vec<Position>> {
    // apply the correction directly to the top layer
    let top_t = simulator.height - 1;
    for (position, error) in correction.iter() {
        assert_eq!(position.t, top_t, "correction pattern must only be at top layer");
        let mut position = position.clone();
        position.t -= simulator.measurement_cycles;  // apply it before the final perfect measurement
        let node = simulator.get_node_mut_unwrap(&position);
        node.error = node.error.multiply(error);
    }
    simulator.clear_propagate_errors();
    simulator.propagate_errors();
    // check if all stabilizers at the final measurement round don't detect errors
    let mut violating_positions = Vec::new();
    simulator_iter_real!(simulator, position, node, t => top_t, {
        if node.gate_type.is_measurement() {
            let minus_one = node.gate_type.stabilizer_measurement(&node.propagated);
            if minus_one {
                violating_positions.push(position.clone());
            }
        }
    });
    // recover the errors
    for (position, error) in correction.iter() {
        let mut position = position.clone();
        position.t -= simulator.measurement_cycles;  // apply it before the final perfect measurement
        let node = simulator.get_node_mut_unwrap(&position);
        node.error = node.error.multiply(error);
    }
    simulator.clear_propagate_errors();
    simulator.propagate_errors();
    if violating_positions.len() > 0 {
        Err(violating_positions)
    } else {
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! assert_measurement {
        ($simulator:ident, $errors:expr, $expected_measurements:expr) => {
            $simulator.clear_all_errors();
            for (position, error) in $errors.iter() {
                let node = $simulator.get_node_mut_unwrap(position);
                assert_eq!(node.error, ErrorType::I, "do not set the error at a same position twice: {} {}", position, error);
                node.error = *error;
            }
            $simulator.propagate_errors();
            assert_eq!($simulator.generate_sparse_measurement().to_vec(), $expected_measurements);
        };
    }

    #[test]
    fn code_builder_standard_planar_code() {  // cargo test code_builder_standard_planar_code -- --nocapture
        let di = 7;
        let dj = 5;
        let noisy_measurements = 3;
        let mut simulator = Simulator::new(CodeType::StandardPlanarCode, BuiltinCodeInformation::new(noisy_measurements, di, dj));
        code_builder_sanity_check(&simulator).unwrap();
        {  // count how many nodes
            let mut nodes_count = 0;
            let mut virtual_nodes_count = 0;
            simulator_iter!(simulator, position, node, {
                // println!("{}", node);
                nodes_count += 1;
                if node.is_virtual {
                    virtual_nodes_count += 1;
                }
            });
            let each_layer_real_node_count = (2 * di - 1) * (2 * dj - 1);
            let each_layer_virtual_node_count = 2 * (di + dj);
            let layer_count = 6 * (noisy_measurements + 1) + 1;
            assert_eq!(nodes_count, layer_count * (each_layer_real_node_count + each_layer_virtual_node_count));
            assert_eq!(virtual_nodes_count, layer_count * each_layer_virtual_node_count);
        }
        {  // check individual qubit type
            {
                let node = simulator.get_node_unwrap(&pos!(0, 0, 1));
                assert_eq!(node.qubit_type, QubitType::StabX);
                assert_eq!(node.gate_type, GateType::MeasureX);
                assert_eq!(node.is_virtual, true);
            }
            {
                let node = simulator.get_node_unwrap(&pos!(0, 0, 2 * dj - 1));
                assert_eq!(node.qubit_type, QubitType::StabX);
                assert_eq!(node.gate_type, GateType::MeasureX);
                assert_eq!(node.is_virtual, true);
            }
            {
                let node = simulator.get_node_unwrap(&pos!(0, 1, 0));
                assert_eq!(node.qubit_type, QubitType::StabZ);
                assert_eq!(node.gate_type, GateType::MeasureZ);
                assert_eq!(node.is_virtual, true);
            }
            {
                let node = simulator.get_node_unwrap(&pos!(0, 2 * di - 1, 0));
                assert_eq!(node.qubit_type, QubitType::StabZ);
                assert_eq!(node.gate_type, GateType::MeasureZ);
                assert_eq!(node.is_virtual, true);
            }
            {
                let node = simulator.get_node_unwrap(&pos!(0, 1, 1));
                assert_eq!(node.qubit_type, QubitType::Data);
                assert_eq!(node.gate_type, GateType::None);
                assert_eq!(node.is_virtual, false);
            }
            {
                let node = simulator.get_node_unwrap(&pos!(0, 1, 2));
                assert_eq!(node.qubit_type, QubitType::StabZ);
                assert_eq!(node.gate_type, GateType::MeasureZ);
                assert_eq!(node.is_virtual, false);
            }
            {
                let node = simulator.get_node_unwrap(&pos!(0, 2, 1));
                assert_eq!(node.qubit_type, QubitType::StabX);
                assert_eq!(node.gate_type, GateType::MeasureX);
                assert_eq!(node.is_virtual, false);
            }
        }
        {  // check gate sequence
            {  // data qubit
                let node = simulator.get_node_unwrap(&pos!(1, 1, 1));
                assert_eq!(node.is_peer_virtual, false);
                assert_eq!(node.gate_type, GateType::None);
                let node = simulator.get_node_unwrap(&pos!(2, 1, 1));
                assert_eq!(node.is_peer_virtual, false);
                assert_eq!(node.gate_type, GateType::CXGateTarget);
                assert_eq!(node.gate_peer.as_ref().map(|x| (**x).clone()), Some(pos!(2, 2, 1)));
                let node = simulator.get_node_unwrap(&pos!(3, 1, 1));
                assert_eq!(node.is_peer_virtual, false);
                assert_eq!(node.gate_type, GateType::CXGateControl);
                assert_eq!(node.gate_peer.as_ref().map(|x| (**x).clone()), Some(pos!(3, 1, 2)));
                let node = simulator.get_node_unwrap(&pos!(4, 1, 1));
                assert_eq!(node.is_peer_virtual, true);
                assert_eq!(node.gate_type, GateType::CXGateControl);
                assert_eq!(node.gate_peer.as_ref().map(|x| (**x).clone()), Some(pos!(4, 1, 0)));
                let node = simulator.get_node_unwrap(&pos!(5, 1, 1));
                assert_eq!(node.is_peer_virtual, true);
                assert_eq!(node.gate_type, GateType::CXGateTarget);
                assert_eq!(node.gate_peer.as_ref().map(|x| (**x).clone()), Some(pos!(5, 0, 1)));
            }
        }
        {  // check stabilizer measurements
            // data qubit at corner
            assert_measurement!(simulator, [(pos!(0, 1, 1), X)], [pos!(6, 1, 2)]);
            assert_measurement!(simulator, [(pos!(0, 1, 1), Z)], [pos!(6, 2, 1)]);
            assert_measurement!(simulator, [(pos!(0, 1, 1), Y)], [pos!(6, 1, 2), pos!(6, 2, 1)]);
            // data qubit at center
            assert_measurement!(simulator, [(pos!(0, 2, 2), X)], [pos!(6, 1, 2), pos!(6, 3, 2)]);
            assert_measurement!(simulator, [(pos!(0, 2, 2), Z)], [pos!(6, 2, 1), pos!(6, 2, 3)]);
            assert_measurement!(simulator, [(pos!(0, 2, 2), Y)], [pos!(6, 1, 2), pos!(6, 2, 1), pos!(6, 2, 3), pos!(6, 3, 2)]);
            // Z stabilizer measurement error
            assert_measurement!(simulator, [(pos!(5, 1, 2), X)], [pos!(6, 1, 2), pos!(12, 1, 2)]);
            assert_measurement!(simulator, [(pos!(5, 1, 2), Z)], []);  // not sensitive to Z error
            assert_measurement!(simulator, [(pos!(5, 1, 2), Y)], [pos!(6, 1, 2), pos!(12, 1, 2)]);
            // X stabilizer measurement error
            assert_measurement!(simulator, [(pos!(5, 2, 1), X)], []);  // not sensitive to X error
            assert_measurement!(simulator, [(pos!(5, 2, 1), Z)], [pos!(6, 2, 1), pos!(12, 2, 1)]);
            assert_measurement!(simulator, [(pos!(5, 2, 1), Y)], [pos!(6, 2, 1), pos!(12, 2, 1)]);
        }
    }

    #[test]
    fn code_builder_standard_tailored_code() {  // cargo test code_builder_standard_tailored_code -- --nocapture
        let di = 7;
        let dj = 5;
        let noisy_measurements = 3;
        let mut simulator = Simulator::new(CodeType::StandardTailoredCode, BuiltinCodeInformation::new(noisy_measurements, di, dj));
        code_builder_sanity_check(&simulator).unwrap();
        {  // check stabilizer measurements
            // data qubit at corner
            assert_measurement!(simulator, [(pos!(0, 1, 1), X)], [pos!(6, 1, 2)]);
            assert_measurement!(simulator, [(pos!(0, 1, 1), Z)], [pos!(6, 1, 2), pos!(6, 2, 1)]);
            assert_measurement!(simulator, [(pos!(0, 1, 1), Y)], [pos!(6, 2, 1)]);
            // data qubit at center
            assert_measurement!(simulator, [(pos!(0, 2, 2), X)], [pos!(6, 1, 2), pos!(6, 3, 2)]);
            assert_measurement!(simulator, [(pos!(0, 2, 2), Z)], [pos!(6, 1, 2), pos!(6, 2, 1), pos!(6, 2, 3), pos!(6, 3, 2)]);
            assert_measurement!(simulator, [(pos!(0, 2, 2), Y)], [pos!(6, 2, 1), pos!(6, 2, 3)]);
            // Y stabilizer measurement error
            assert_measurement!(simulator, [(pos!(5, 1, 2), X)], []);
            assert_measurement!(simulator, [(pos!(5, 1, 2), Z)], [pos!(6, 1, 2), pos!(12, 1, 2)]);  // not sensitive to Z error
            assert_measurement!(simulator, [(pos!(5, 1, 2), Y)], [pos!(6, 1, 2), pos!(12, 1, 2)]);
            // X stabilizer measurement error
            assert_measurement!(simulator, [(pos!(5, 2, 1), X)], []);  // not sensitive to X error
            assert_measurement!(simulator, [(pos!(5, 2, 1), Z)], [pos!(6, 2, 1), pos!(12, 2, 1)]);
            assert_measurement!(simulator, [(pos!(5, 2, 1), Y)], [pos!(6, 2, 1), pos!(12, 2, 1)]);
        }
    }

    #[test]
    fn code_builder_periodic_rotated_tailored_code() {  // cargo test code_builder_periodic_rotated_tailored_code -- --nocapture
        let di = 7;
        let dj = 5;
        let noisy_measurements = 0;
        let mut simulator = Simulator::new(CodeType::PeriodicRotatedTailoredCode, BuiltinCodeInformation::new(noisy_measurements, di+1, dj+1));
        code_builder_sanity_check(&simulator).unwrap();
        {  // check stabilizer measurements
            // data qubit at center
            assert_measurement!(simulator, [(pos!(0, 1, 5), X)], [pos!(6, 1, 4), pos!(6, 1, 6)]);
            assert_measurement!(simulator, [(pos!(0, 1, 5), Z)], [pos!(6, 0, 5), pos!(6, 1, 4), pos!(6, 1, 6), pos!(6, 2, 5)]);
            assert_measurement!(simulator, [(pos!(0, 1, 5), Y)], [pos!(6, 0, 5), pos!(6, 2, 5)]);
            // data qubit at periodic boundary
            assert_measurement!(simulator, [(pos!(0, 6, 0), X)], [pos!(6, 1, 6), pos!(6, 5, 0)]);
            assert_measurement!(simulator, [(pos!(0, 6, 0), Z)], [pos!(6, 0, 5), pos!(6, 1, 6), pos!(6, 5, 0), pos!(6, 6, 1)]);
            assert_measurement!(simulator, [(pos!(0, 6, 0), Y)], [pos!(6, 0, 5), pos!(6, 6, 1)]);
            // data qubit at periodic boundary
            assert_measurement!(simulator, [(pos!(0, 7, 1), X)], [pos!(6, 1, 6), pos!(6, 7, 2)]);
            assert_measurement!(simulator, [(pos!(0, 7, 1), Z)], [pos!(6, 1, 6), pos!(6, 2, 7), pos!(6, 6, 1), pos!(6, 7, 2)]);
            assert_measurement!(simulator, [(pos!(0, 7, 1), Y)], [pos!(6, 2, 7), pos!(6, 6, 1)]);
            // data qubit at periodic boundary
            assert_measurement!(simulator, [(pos!(0, 13, 7), X)], [pos!(6, 5, 0), pos!(6, 7, 12)]);
            assert_measurement!(simulator, [(pos!(0, 13, 7), Z)], [pos!(6, 0, 5), pos!(6, 5, 0), pos!(6, 7, 12), pos!(6, 12, 7)]);
            assert_measurement!(simulator, [(pos!(0, 13, 7), Y)], [pos!(6, 0, 5), pos!(6, 12, 7)]);
            // data qubit at periodic boundary
            assert_measurement!(simulator, [(pos!(0, 8, 12), X)], [pos!(6, 1, 4), pos!(6, 7, 12)]);
            assert_measurement!(simulator, [(pos!(0, 8, 12), Z)], [pos!(6, 0, 5), pos!(6, 1, 4), pos!(6, 7, 12), pos!(6, 8, 11)]);
            assert_measurement!(simulator, [(pos!(0, 8, 12), Y)], [pos!(6, 0, 5), pos!(6, 8, 11)]);
        }
    }

}

#[cfg(feature="python_binding")]
#[pyfunction]
pub(crate) fn register(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<CodeType>()?;
    m.add_class::<BuiltinCodeInformation>()?;
    Ok(())
}  
