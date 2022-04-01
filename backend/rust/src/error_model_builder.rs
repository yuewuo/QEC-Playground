//! build customized error model by giving a name
//! 

use super::simulator::*;
use serde::{Serialize};
use super::types::*;
use super::util_macros::*;
use super::error_model::*;
use super::clap::{ArgEnum, PossibleValue};
use std::sync::{Arc};

/// commonly used error models
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Serialize, Debug)]
pub enum ErrorModelBuilder {
    /// add data qubit errors and measurement errors individually
    Phenomenological,
}

impl ErrorModelBuilder {
    pub fn possible_values<'a>() -> impl Iterator<Item = PossibleValue<'a>> {
        Self::value_variants().iter().filter_map(ArgEnum::to_possible_value)
    }

    /// apply error model
    pub fn apply(&self, simulator: &Simulator, error_model: &mut ErrorModel, error_model_configuration: &serde_json::Value, p: f64, bias_eta: f64, pe: f64) {
        // commonly used biased qubit error node
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        let mut biased_node = ErrorModelNode::new();
        biased_node.pauli_error_rates.error_rate_X = px;
        biased_node.pauli_error_rates.error_rate_Y = py;
        biased_node.pauli_error_rates.error_rate_Z = pz;
        biased_node.erasure_error_rate = pe;
        let biased_node = Arc::new(biased_node);
        // commonly used pure measurement error node
        let mut pm = p;
        if let Some(value) = error_model_configuration.get("measurement_error_rate") {
            pm = value.as_f64().expect("measurement_error_rate must be `f64`");
        }
        let mut pure_measurement_node = ErrorModelNode::new();
        pure_measurement_node.pauli_error_rates.error_rate_Y = pm;  // Y error will cause pure measurement error for StabX (X basis), StabZ (Z basis), StabY (X basis)
        let pure_measurement_node = Arc::new(pure_measurement_node);
        // commonly used noiseless error node
        let noiseless_node = Arc::new(ErrorModelNode::new());
        // error model builder
        match self {
            ErrorModelBuilder::Phenomenological => {
                assert!(px + py + pz <= 1. && px >= 0. && py >= 0. && pz >= 0.);
                assert!(pe == 0.);  // phenomenological error model doesn't support erasure errors
                if simulator.measurement_cycles == 1 {
                    eprintln!("[warning] setting error rates of unknown code, no perfect measurement protection is enabled");
                }
                simulator_iter_real!(simulator, position, node, {
                    error_model.set_node(position, Some(noiseless_node.clone()));  // clear existing noise model
                    if position.t < simulator.height - simulator.measurement_cycles {  // no error at the final perfect measurement round
                        if position.t % simulator.measurement_cycles == 0 && node.qubit_type == QubitType::Data {
                            error_model.set_node(position, Some(biased_node.clone()));
                        }
                        if (position.t + 1) % simulator.measurement_cycles == 0 && node.qubit_type != QubitType::Data {  // measurement error must happen before measurement round
                            error_model.set_node(position, Some(pure_measurement_node.clone()));
                        }
                    }
                });
            },
        }
    }
}

impl std::str::FromStr for ErrorModelBuilder {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("Invalid variant: {}", s))
    }
}
