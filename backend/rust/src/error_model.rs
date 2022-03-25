//! # Error Model
//!
//! customized error rate with high flexibility
//! 

use super::simulator::*;
use super::util_macros::*;
use super::code_builder::*;


/// check if error rates are not zero at perfect measurement ranges or at virtual nodes,
/// also check for error rate constrains on virtual nodes
pub fn error_model_sanity_check(simulator: &Simulator) -> Result<(), String> {
    match simulator.code_type.builtin_code_information() {
        Some(BuiltinCodeInformation{ measurement_cycles, noisy_measurements, .. }) => {
            // check that no errors present in the final perfect measurement rounds
            let expected_height = measurement_cycles * (noisy_measurements + 1) + 1;
            if simulator.height != expected_height {
                return Err(format!("height {} is not expected {}, don't know where is perfect measurement", simulator.height, expected_height))
            }
            for t in simulator.height - measurement_cycles .. simulator.height {
                simulator_iter!(simulator, position, node, t => t, {
                    if !node.is_noiseless() {
                        return Err(format!("detected noisy position {} within final perfect measurement", position))
                    }
                });
            }
            // check all no error rate at virtual nodes
            simulator_iter_virtual!(simulator, position, node, {  // only check for virtual nodes
                if !node.is_noiseless() {
                    return Err(format!("detected noisy position {} which is virtual node", position))
                }
            });
        }, _ => { println!("[warning] code doesn't provide enough information for sanity check") }
    }
    simulator_iter!(simulator, position, node, {
        // println!("{}", node);
        if node.is_virtual {  // no errors on virtual node is allowed, because they don't physically exist
            if node.pauli_error_rates.error_probability() > 0. {
                return Err(format!("virtual position at {} have non-zero pauli_error_rates: {:?}", position, node.pauli_error_rates))
            }
            if node.erasure_error_rate > 0. {
                return Err(format!("virtual position at {} have non-zero erasure_error_rate: {}", position, node.erasure_error_rate))
            }
            if let Some(correlated_pauli_error_rates) = &node.correlated_pauli_error_rates {
                if correlated_pauli_error_rates.error_probability() > 0. {
                    return Err(format!("virtual position at {} have non-zero correlated_pauli_error_rates: {:?}", position, correlated_pauli_error_rates))
                }
            }
            if let Some(correlated_erasure_error_rates) = &node.correlated_erasure_error_rates {
                if correlated_erasure_error_rates.error_probability() > 0. {
                    return Err(format!("virtual position at {} have non-zero correlated_erasure_error_rates: {:?}", position, correlated_erasure_error_rates))
                }
            }
        }
        // if node.is_peer_virtual {  // no correlated errors if peer position is virtual, because this two-qubit gate doesn't physically exist
        //     if node.pauli_error_rates.error_probability() > 0. {
        //         return Err(format!("virtual position at {} have non-zero pauli_error_rates: {:?}", position, node.pauli_error_rates))
        //     }
        // }
    });
    Ok(())
}
