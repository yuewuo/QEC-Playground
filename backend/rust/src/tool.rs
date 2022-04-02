#![allow(non_snake_case)]

use super::clap;
use super::rand::prelude::*;
use super::serde_json;
use super::types::*;
#[cfg(feature="python_interfaces")]
use super::pyo3::prelude::*;
#[cfg(feature="python_interfaces")]
use super::pyo3::types::{IntoPyDict};
use super::num_cpus;
use std::sync::{Arc, Mutex};
use super::ftqec;
use super::pbr::ProgressBar;
use super::offer_decoder;
use super::offer_mwpm;
use super::union_find_decoder;
use super::distributed_uf_decoder;
use super::serde_json::{json};
use super::either::Either;
use super::fast_benchmark;
use super::rug;
use super::rug::ops::Pow;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;
use super::reproducible_rand::Xoroshiro128StarStar;
use super::util::local_get_temporary_store;
use std::fs;
use super::code_builder::*;
use super::simulator::*;
use super::clap::{ArgEnum, PossibleValue};
use std::sync::atomic::{AtomicBool, Ordering};
use super::error_model::*;
use serde::{Serialize, Deserialize};
use super::mwpm_decoder::*;
use super::model_graph::*;
use super::complete_model_graph::*;
use super::tailored_mwpm_decoder::*;
use super::tailored_model_graph::*;
use super::tailored_complete_model_graph::*;
use super::error_model_builder::*;

pub fn run_matched_tool(matches: &clap::ArgMatches) -> Option<String> {
    match matches.subcommand() {
        Some(("benchmark", matches)) => {
            let dis: String = matches.value_of_t("dis").expect("required");
            let djs: String = matches.value_of_t("djs").unwrap_or(dis.clone());
            let dis: Vec<usize> = serde_json::from_str(&dis).expect("dis should be [di1,di2,di3,...,din]");
            let djs: Vec<usize> = serde_json::from_str(&djs).expect("djs should be [dj1,dj2,dj3,...,djn]");
            let nms: String = matches.value_of_t("nms").expect("required");
            let nms: Vec<usize> = serde_json::from_str(&nms).expect("nms should be [nm1,nm2,nm3,...,nmn]");
            assert!(nms.len() == dis.len(), "nms and dis should be paired");
            assert!(dis.len() == djs.len(), "dis and djs should be paired");
            let ps: String = matches.value_of_t("ps").expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let pes: Option<String> = matches.value_of_t("pes").ok();
            let pes: Vec<f64> = match pes {
                Some(pes) => serde_json::from_str(&pes).expect("pes should be [pe1,pe2,pe3,...,pem]"),
                None => vec![0.; ps.len()],  // by default no erasure errors
            };
            let bias_eta: f64 = matches.value_of_t("bias_eta").unwrap();
            assert_eq!(pes.len(), ps.len(), "pe and p should be paired");
            let mut max_repeats: usize = matches.value_of_t("max_repeats").unwrap();
            if max_repeats == 0 {
                max_repeats = usize::MAX;
            }
            let mut min_failed_cases: usize = matches.value_of_t("min_failed_cases").unwrap();
            if min_failed_cases == 0 {
                min_failed_cases = usize::MAX;
            }
            let parallel: usize = matches.value_of_t("parallel").unwrap();
            let code_type: String = matches.value_of_t("code_type").unwrap_or("StandardPlanarCode".to_string());
            let decoder = matches.value_of_t::<BenchmarkDecoder>("decoder").unwrap();
            let decoder_config = matches.value_of_t::<serde_json::Value>("decoder_config").unwrap();
            let ignore_logical_i = matches.is_present("ignore_logical_i");
            let ignore_logical_j = matches.is_present("ignore_logical_j");
            let debug_print = matches.value_of_t::<BenchmarkDebugPrint>("debug_print").ok();
            let time_budget: Option<f64> = matches.value_of_t("time_budget").ok();
            let log_runtime_statistics: Option<String> = matches.value_of_t("log_runtime_statistics").ok();
            let log_error_pattern_when_logical_error = matches.is_present("log_error_pattern_when_logical_error");
            let error_model_builder = matches.value_of_t::<ErrorModelBuilder>("error_model").ok();
            let error_model_configuration = matches.value_of_t::<serde_json::Value>("error_model_configuration").unwrap();
            let thread_timeout: f64 = matches.value_of_t("thread_timeout").unwrap();
            return Some(benchmark(&dis, &djs, &nms, &ps, &pes, bias_eta, max_repeats, min_failed_cases, parallel, code_type, decoder, decoder_config
                , ignore_logical_i, ignore_logical_j, debug_print, time_budget, log_runtime_statistics, log_error_pattern_when_logical_error
                , error_model_builder, error_model_configuration, thread_timeout));
        }
        Some(("fault_tolerant_benchmark", matches)) => {
            let dis: String = matches.value_of_t("Ls").expect("required");
            let djs: String = matches.value_of_t("djs").unwrap_or(dis.clone());
            let dis: Vec<usize> = serde_json::from_str(&dis).expect("Ls should be [L1,L2,L3,...,Ln]");
            let djs: Vec<usize> = serde_json::from_str(&djs).expect("djs should be [dj1,dj2,dj3,...,djn]");
            let Ts: String = matches.value_of_t("Ts").expect("required");
            let Ts: Vec<usize> = serde_json::from_str(&Ts).expect("Ts should be [T1,T2,T3,...,Tn]");
            assert!(Ts.len() == dis.len(), "Ts and dis should be paired");
            assert!(dis.len() == djs.len(), "dis and djs should be paired");
            let ps: String = matches.value_of_t("ps").expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let pes: Option<String> = matches.value_of_t("pes").ok();
            let pes: Vec<f64> = match pes {
                Some(pes) => serde_json::from_str(&pes).expect("pes should be [pe1,pe2,pe3,...,pem]"),
                None => vec![0.; ps.len()],  // by default no erasure errors
            };
            let mut max_N: usize = matches.value_of_t("max_N").unwrap_or(100000000);  // default to 1e8
            if max_N == 0 {
                max_N = usize::MAX;
            }
            let mut min_error_cases: usize = matches.value_of_t("min_error_cases").unwrap_or(10000);  // default to 1e3
            if min_error_cases == 0 {
                min_error_cases = usize::MAX;
            }
            let parallel: usize = matches.value_of_t("parallel").unwrap_or(1);  // default to 1
            let validate_layer: String = matches.value_of_t("validate_layer").unwrap_or("boundary".to_string());
            let mini_sync_time: f64 = matches.value_of_t("mini_sync_time").unwrap_or(0.5);  // default to 0.5s
            let autotune = ! matches.is_present("no_autotune");  // default autotune is enabled
            let rotated_planar_code = matches.is_present("rotated_planar_code");  // default use standard planar code
            let ignore_6_neighbors = matches.is_present("ignore_6_neighbors");  // default use 12 neighbors version
            let extra_measurement_error: f64 = matches.value_of_t("extra_measurement_error").unwrap_or(1.);  // default to 1.
            let bypass_correction = matches.is_present("bypass_correction");
            let independent_px_pz = matches.is_present("independent_px_pz");
            let only_count_logical_x = matches.is_present("only_count_logical_x");
            let only_count_logical_z = matches.is_present("only_count_logical_z");
            let imperfect_initialization = matches.is_present("imperfect_initialization");
            let shallow_error_on_bottom = matches.is_present("shallow_error_on_bottom");
            let no_y_error = matches.is_present("no_y_error");
            let use_xzzx_code = matches.is_present("use_xzzx_code");
            let use_rotated_tailored_code = matches.is_present("use_rotated_tailored_code");
            let bias_eta: f64 = matches.value_of_t("bias_eta").unwrap_or(0.5);  // default to 0.5
            let decoder_type = DecoderType::from(matches.value_of_t::<String>("decoder").unwrap_or("MWPM".to_string()));
            let max_half_weight: usize = matches.value_of_t("max_half_weight").unwrap_or(1);  // default to 1
            let disable_combined_probability = matches.is_present("disable_combined_probability");
            let disable_autotune_minus_no_error = matches.is_present("disable_autotune_minus_no_error");
            let error_model: Option<ErrorModelName> = matches.value_of_t("error_model").ok().map(|x: String| ErrorModelName::from(x));
            let error_model_configuration: Option<serde_json::Value> = matches.value_of_t::<String>("error_model_configuration").ok().and_then(|config| {
                Some(serde_json::from_str(config.as_str()).expect("error_model_configuration must be a json object"))
            });
            let no_stop_if_next_model_is_not_prepared = matches.is_present("no_stop_if_next_model_is_not_prepared");
            let log_runtime_statistics: Option<String> = matches.value_of_t("log_runtime_statistics").ok();
            let detailed_runtime_statistics = matches.is_present("detailed_runtime_statistics");
            let log_error_pattern_into_statistics_when_has_logical_error = matches.is_present("log_error_pattern_into_statistics_when_has_logical_error");
            let time_budget: Option<f64> = matches.value_of_t("time_budget").ok();
            let use_fast_benchmark = matches.is_present("use_fast_benchmark");
            let fbench_disable_additional_error = matches.is_present("fbench_disable_additional_error");
            let fbench_use_fake_decoder = matches.is_present("fbench_use_fake_decoder");
            let fbench_use_simple_sum = matches.is_present("fbench_use_simple_sum");
            let fbench_assignment_sampling_amount: usize = matches.value_of_t("fbench_assignment_sampling_amount").unwrap_or(1);  // default to 1
            let fbench_weighted_path_sampling = matches.is_present("fbench_weighted_path_sampling");
            let fbench_weighted_assignment_sampling = matches.is_present("fbench_weighted_assignment_sampling");
            let fbench_target_dev: f64 = matches.value_of_t("fbench_target_dev").unwrap_or(0.);  // default to 0
            let rug_precision: u32 = matches.value_of_t("rug_precision").unwrap_or(128);  // default to 128
            let disable_optimize_correction_pattern = matches.is_present("disable_optimize_correction_pattern");
            let debug_print_only = matches.is_present("debug_print_only");
            let debug_print_direct_connections = matches.is_present("debug_print_direct_connections");
            let debug_print_exhausted_connections = matches.is_present("debug_print_exhausted_connections");
            let debug_print_error_model = matches.is_present("debug_print_error_model");
            let debug_print_with_all_possible_error_rates = matches.is_present("debug_print_with_all_possible_error_rates");
            let use_reduced_graph = !matches.is_present("disable_reduced_graph");
            let mut error_model_modifier_str: Option<String> = None;
            match matches.value_of_t::<usize>("load_error_model_from_temporary_store") {
                Ok(error_model_temporary_id) => {
                    match local_get_temporary_store(error_model_temporary_id) {
                        Some(value) => { error_model_modifier_str = Some(value); },
                        None => { return Some(format!("[error] temporary id not found (may expire): {}", error_model_temporary_id)) }
                    }
                },
                Err(_) => { },
            }
            match matches.value_of_t::<String>("load_error_model_from_file") {
                Ok(error_model_filepath) => {
                    match fs::read_to_string(error_model_filepath.clone()) {
                        Ok(value) => { error_model_modifier_str = Some(value); },
                        Err(_) => { return Some(format!("[error] error model file cannot open: {}", error_model_filepath)) }
                    }
                },
                Err(_) => { },
            }
            let error_model_modifier: Option<serde_json::Value> = match error_model_modifier_str {
                Some(value) => match serde_json::from_str(&value) {
                    Ok(error_model_modifier) => Some(error_model_modifier),
                    Err(_) => { return Some(format!("[error] error model cannot recognize, please check file format")) }
                },
                None => None,
            };
            return Some(fault_tolerant_benchmark(&dis, &djs, &Ts, &ps, &pes, max_N, min_error_cases, parallel, validate_layer, mini_sync_time, autotune, rotated_planar_code
                , ignore_6_neighbors, extra_measurement_error, bypass_correction, independent_px_pz, only_count_logical_x, only_count_logical_z
                , !imperfect_initialization, shallow_error_on_bottom, no_y_error, use_xzzx_code, use_rotated_tailored_code, bias_eta, decoder_type, max_half_weight
                , !disable_combined_probability, !disable_autotune_minus_no_error, error_model, error_model_configuration, no_stop_if_next_model_is_not_prepared, log_runtime_statistics
                , detailed_runtime_statistics, log_error_pattern_into_statistics_when_has_logical_error, time_budget, use_fast_benchmark
                , fbench_disable_additional_error, fbench_use_fake_decoder, fbench_use_simple_sum, fbench_assignment_sampling_amount
                , fbench_weighted_path_sampling, fbench_weighted_assignment_sampling, fbench_target_dev, rug_precision, disable_optimize_correction_pattern
                , debug_print_only, debug_print_direct_connections, debug_print_exhausted_connections, debug_print_error_model, debug_print_with_all_possible_error_rates
                , use_reduced_graph, error_model_modifier));
        }
        // TODO: these tools can be part of `fault_tolerant_benchmark`, remove after merged
        Some(("offer_decoder_standard_planar_benchmark", matches)) => {
            let Ls: String = matches.value_of_t("Ls").expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps: String = matches.value_of_t("ps").expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N: usize = matches.value_of_t("max_N").unwrap_or(100000000);  // default to 1e8
            let min_error_cases: usize = matches.value_of_t("min_error_cases").unwrap_or(10000);  // default to 1e3
            let parallel: usize = matches.value_of_t("parallel").unwrap_or(1);  // default to 1
            let mini_batch: usize = matches.value_of_t("mini_batch").unwrap_or(1);  // default to 1
            let only_count_logical_x = matches.is_present("only_count_logical_x");
            let max_resend: usize = matches.value_of_t("max_resend").unwrap_or(usize::MAX);
            let max_cycles: usize = matches.value_of_t("max_cycles").unwrap_or(usize::MAX);
            let disable_probabilistic_accept = matches.is_present("disable_probabilistic_accept");
            let repeat_experiment_each_error: usize = matches.value_of_t("repeat_experiment_each_error").unwrap_or(1);
            offer_decoder_standard_planar_benchmark(&Ls, &ps, max_N, min_error_cases, parallel, mini_batch, only_count_logical_x, max_resend, max_cycles
                , disable_probabilistic_accept, repeat_experiment_each_error);
        }
        Some(("offer_algorithm_standard_planar_benchmark", matches)) => {
            let Ls: String = matches.value_of_t("Ls").expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps: String = matches.value_of_t("ps").expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N: usize = matches.value_of_t("max_N").unwrap_or(100000000);  // default to 1e8
            let min_error_cases: usize = matches.value_of_t("min_error_cases").unwrap_or(10000);  // default to 1e3
            let parallel: usize = matches.value_of_t("parallel").unwrap_or(1);  // default to 1
            let mini_batch: usize = matches.value_of_t("mini_batch").unwrap_or(1);  // default to 1
            let only_count_logical_x = matches.is_present("only_count_logical_x");
            let max_resend: usize = matches.value_of_t("max_resend").unwrap_or(usize::MAX);
            let max_cycles: usize = matches.value_of_t("max_cycles").unwrap_or(usize::MAX);
            let disable_probabilistic_accept = matches.is_present("disable_probabilistic_accept");
            let repeat_experiment_each_error: usize = matches.value_of_t("repeat_experiment_each_error").unwrap_or(1);
            offer_algorithm_standard_planar_benchmark(&Ls, &ps, max_N, min_error_cases, parallel, mini_batch, only_count_logical_x, max_resend, max_cycles
                , disable_probabilistic_accept, repeat_experiment_each_error);
        }
        Some(("union_find_decoder_standard_planar_benchmark", matches)) => {
            let Ls: String = matches.value_of_t("Ls").expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps: String = matches.value_of_t("ps").expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N: usize = matches.value_of_t("max_N").unwrap_or(100000000);  // default to 1e8
            let min_error_cases: usize = matches.value_of_t("min_error_cases").unwrap_or(10000);  // default to 1e3
            let parallel: usize = matches.value_of_t("parallel").unwrap_or(1);  // default to 1
            let mini_batch: usize = matches.value_of_t("mini_batch").unwrap_or(1);  // default to 1
            let only_count_logical_x = matches.is_present("only_count_logical_x");
            let no_y_error = matches.is_present("no_y_error");
            let towards_mwpm = matches.is_present("towards_mwpm");
            let max_half_weight: usize = matches.value_of_t("max_half_weight").unwrap_or(1);  // default to 1
            let bias_eta: f64 = matches.value_of_t("bias_eta").unwrap_or(0.5);  // default to 0.5
            union_find_decoder_standard_planar_benchmark(&Ls, &ps, max_N, min_error_cases, parallel, mini_batch, only_count_logical_x, no_y_error, towards_mwpm
                , max_half_weight, bias_eta);
        }
        Some(("distributed_union_find_decoder_standard_planar_benchmark", matches)) => {
            let Ls: String = matches.value_of_t("Ls").expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps: String = matches.value_of_t("ps").expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N: usize = matches.value_of_t("max_N").unwrap_or(100000000);  // default to 1e8
            let min_error_cases: usize = matches.value_of_t("min_error_cases").unwrap_or(10000);  // default to 1e3
            let parallel: usize = matches.value_of_t("parallel").unwrap_or(1);  // default to 1
            let mini_batch: usize = matches.value_of_t("mini_batch").unwrap_or(1);  // default to 1
            let only_count_logical_x = matches.is_present("only_count_logical_x");
            let output_cycle_distribution = matches.is_present("output_cycle_distribution");
            let fast_channel_interval: usize = matches.value_of_t("fast_channel_interval").unwrap_or(0);  // default to 0
            let no_y_error = matches.is_present("no_y_error");
            distributed_union_find_decoder_standard_planar_benchmark(&Ls, &ps, max_N, min_error_cases, parallel, mini_batch, only_count_logical_x, output_cycle_distribution, fast_channel_interval, no_y_error);
        }
        _ => unreachable!()
    }
    None
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Serialize)]
pub enum BenchmarkDebugPrint {
    /// the original error model
    ErrorModel,
    /// including every possible error rate (correlated ones), but initialize them as 0
    FullErrorModel,
    /// model graph, supporting decoder config `weight_function` or `wf`
    ModelGraph,
    /// complete model graph, supporting decoder config `weight_function` or `wf`, `precompute_complete_model_graph` or `pcmg`
    CompleteModelGraph,
    /// tailored model graph, supporting decoder config `weight_function` or `wf`
    TailoredModelGraph,
    /// tailored complete model graph, supporting decoder config `weight_function` or `wf`, `precompute_complete_model_graph` or `pcmg`
    TailoredCompleteModelGraph,
    /// print all error patterns immediately after generating random errors, typically useful to pinpoint how program assertion fail
    AllErrorPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDebugPrintDecoderConfig {
    /// see [`mwpm_decoder::MWPMDecoderConfig`]
    #[serde(alias = "pcmg")]  // abbreviation
    #[serde(default = "mwpm_default_configs::precompute_complete_model_graph")]
    pub precompute_complete_model_graph: bool,
    /// see [`mwpm_decoder::MWPMDecoderConfig`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Serialize)]
pub enum BenchmarkDecoder {
    /// no decoder applied, return empty correction
    None,
    /// minimum-weight perfect matching decoder
    MWPM,
    /// tailored surface code MWPM decoder
    TailoredMWPM,
}

/// progress variable shared between threads to update information
#[derive(Clone, Debug, Serialize)]
struct BenchmarkControl {
    total_repeats: usize,
    qec_failed: usize,
    external_termination: bool,
}

impl BenchmarkControl {
    fn new() -> Self {
        Self {
            total_repeats: 0,
            qec_failed: 0,
            external_termination: false,
        }
    }
    fn update_data_should_terminate(&mut self, is_qec_failed: bool, max_repeats: usize, min_failed_cases: usize) -> bool {
        self.total_repeats += 1;
        if is_qec_failed {
            self.qec_failed += 1;
        }
        self.should_terminate(max_repeats, min_failed_cases)
    }
    fn should_terminate(&self, max_repeats: usize, min_failed_cases: usize) -> bool {
        self.external_termination || self.total_repeats >= max_repeats || self.qec_failed >= min_failed_cases
    }
    fn set_external_terminate(&mut self) {
        self.external_termination = true;
    }
}

/// decoder might suffer from rare deadlock, and this controller will record the necessary information for debugging with low runtime overhead
#[derive(Clone, Debug, Serialize)]
struct BenchmarkThreadDebugger {
    thread_counter: usize,
    error_pattern: Option<SparseErrorPattern>,
    measurement: Option<SparseMeasurement>,
    correction: Option<SparseCorrection>,
}

impl BenchmarkThreadDebugger {
    fn new() -> Self {
        Self {
            thread_counter: 0,
            error_pattern: None,
            measurement: None,
            correction: None,
        }
    }
    fn update_thread_counter(&mut self, thread_counter: usize) -> &mut Self {
        self.thread_counter = thread_counter;
        self.error_pattern = None;
        self.measurement = None;
        self.correction = None;
        self
    }
}

fn benchmark(dis: &Vec<usize>, djs: &Vec<usize>, nms: &Vec<usize>, ps: &Vec<f64>, pes: &Vec<f64>, bias_eta: f64, max_repeats: usize, min_failed_cases: usize
        , parallel: usize, code_type: String, decoder: BenchmarkDecoder, decoder_config: serde_json::Value, ignore_logical_i: bool, ignore_logical_j: bool
        , debug_print: Option<BenchmarkDebugPrint>, time_budget: Option<f64>, log_runtime_statistics: Option<String>, log_error_pattern_when_logical_error: bool
        , error_model_builder: Option<ErrorModelBuilder>, error_model_configuration: serde_json::Value, thread_timeout: f64) -> String {
    // if parallel = 0, use all CPU resources
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    // create runtime statistics file object if given file path
    let log_runtime_statistics_file = log_runtime_statistics.clone().map(|filename| 
        Arc::new(Mutex::new(File::create(filename.as_str()).expect("cannot create file"))));
    let fixed_configuration = json!({
        "dis": dis,
        "djs": djs,
        "nms": nms,
        "ps": ps,
        "pes": pes,
        "bias_eta": bias_eta,
        "max_repeats": max_repeats,
        "min_failed_cases": min_failed_cases,
        "parallel": parallel,
        "code_type": code_type,
        "decoder": decoder,
        "decoder_config": decoder_config,
        "ignore_logical_i": ignore_logical_i,
        "ignore_logical_j": ignore_logical_j,
        "debug_print": debug_print,
        "log_runtime_statistics": log_runtime_statistics,
        "log_error_pattern_when_logical_error": log_error_pattern_when_logical_error,
    });
    match &log_runtime_statistics_file {  // append runtime statistics data
        Some(log_runtime_statistics_file) => {
            let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
            log_runtime_statistics_file.write(b"#f ").unwrap();
            log_runtime_statistics_file.write(fixed_configuration.to_string().as_bytes()).unwrap();
            log_runtime_statistics_file.write(b"\n").unwrap();
            log_runtime_statistics_file.sync_data().unwrap();
        }, _ => { },
    }
    // first list all configurations and validate them at the beginning
    assert_eq!(pes.len(), ps.len(), "pe and p should be matched");
    let mut configurations = Vec::new();
    for (di_idx, &di) in dis.iter().enumerate() {
        let noisy_measurements = nms[di_idx];
        let dj = djs[di_idx];
        for (p_idx, p) in ps.iter().enumerate() {
            let p = *p;
            let pe = pes[p_idx];
            assert!(p >= 0. && p <= 1.0, "invalid probability value");
            assert!(pe >= 0. && pe <= 1.0, "invalid probability value");
            configurations.push((di, dj, noisy_measurements, p, pe));
        }
    }
    let mut output = format!("");
    if debug_print.is_none() {  // debug print only will not run simulations
        output = format!("format: <p> <di> <nm> <total_repeats> <qec_failed> <error_rate> <dj> <confidence_interval_95_percent> <pe>");
        eprintln!("{}", output);  // compatible with old scripts
    }
    // start running simulations
    for &(di, dj, noisy_measurements, p, pe) in configurations.iter() {
        // append runtime statistics data
        match &log_runtime_statistics_file {
            Some(log_runtime_statistics_file) => {
                let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                log_runtime_statistics_file.write(b"# ").unwrap();
                log_runtime_statistics_file.write(json!({
                    "di": di,
                    "dj": dj,
                    "noisy_measurements": noisy_measurements,
                    "p": p,
                    "pe": pe,
                }).to_string().as_bytes()).unwrap();
                log_runtime_statistics_file.write(b"\n").unwrap();
                log_runtime_statistics_file.sync_data().unwrap();
            }, _ => { },
        }
        // prepare simulator
        let mut simulator = Simulator::new(CodeType::new(&code_type, noisy_measurements, di, dj));
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, pe);
        debug_assert!({  // check correctness only in debug mode because it's expensive
            let sanity_check_result = code_builder_sanity_check(&simulator);
            if let Err(message) = &sanity_check_result {
                println!("[error] code_builder_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        // apply customized error model
        if let Some(error_model_builder) = &error_model_builder {
            error_model_builder.apply(&simulator, &mut error_model, &error_model_configuration, p, bias_eta, pe);
        }
        assert!({  // this assertion is cheap, check it in release mode as well
            let sanity_check_result = error_model_sanity_check(&simulator, &error_model);
            if let Err(message) = &sanity_check_result {
                println!("[error] error_model_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        simulator.compress_error_rates(&mut error_model);  // by default compress all error rates
        match debug_print {
            Some(BenchmarkDebugPrint::ErrorModel) => {
                return format!("{}\n", serde_json::to_string(&simulator.to_json(&error_model)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::FullErrorModel) => {
                simulator.expand_error_rates(&mut error_model);  // expand all optional error rates
                return format!("{}\n", serde_json::to_string(&simulator.to_json(&error_model)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::ModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut model_graph = ModelGraph::new(&simulator);
                model_graph.build(&mut simulator, &error_model, &config.weight_function);
                return format!("{}\n", serde_json::to_string(&model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::CompleteModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut model_graph = ModelGraph::new(&simulator);
                model_graph.build(&mut simulator, &error_model, &config.weight_function);
                let mut complete_model_graph = CompleteModelGraph::new(&simulator, &model_graph);
                complete_model_graph.precompute(&simulator, &model_graph, config.precompute_complete_model_graph);
                return format!("{}\n", serde_json::to_string(&complete_model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::TailoredModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                tailored_model_graph.build(&mut simulator, &error_model, &config.weight_function);
                return format!("{}\n", serde_json::to_string(&tailored_model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::TailoredCompleteModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                tailored_model_graph.build(&mut simulator, &error_model, &config.weight_function);
                let mut complete_tailored_model_graph = TailoredCompleteModelGraph::new(&simulator, &tailored_model_graph);
                complete_tailored_model_graph.precompute(&simulator, &tailored_model_graph, config.precompute_complete_model_graph);
                return format!("{}\n", serde_json::to_string(&complete_tailored_model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            _ => { }
        }
        let debug_print = Arc::new(debug_print);  // share it across threads
        let error_model = Arc::new(error_model);  // change mutability of error model
        // build decoder precomputed data which is shared between threads
        if decoder == BenchmarkDecoder::None {
            assert!(decoder_config.is_object() && decoder_config.as_object().unwrap().len() == 0, "this decoder doesn't support decoder configuration");
        }
        let mwpm_decoder = if decoder == BenchmarkDecoder::MWPM {
            Some(MWPMDecoder::new(&simulator, &error_model, &decoder_config))
        } else { None };
        let tailored_mwpm_decoder = if decoder == BenchmarkDecoder::TailoredMWPM {
            Some(TailoredMWPMDecoder::new(&simulator, &error_model, &decoder_config))
        } else { None };
        // prepare result variables for simulation
        let benchmark_control = Arc::new(Mutex::new(BenchmarkControl::new()));
        // setup progress bar
        let mut pb = ProgressBar::on(std::io::stderr(), max_repeats as u64);
        pb.set(0);
        // spawn threads to do simulation
        let mut handlers = Vec::new();
        let mut threads_debugger: Vec<Arc<Mutex<BenchmarkThreadDebugger>>> = Vec::new();
        let mut threads_ended = Vec::new();  // keep updating progress bar until all threads ends
        for _parallel_idx in 0..parallel {
            let benchmark_control = Arc::clone(&benchmark_control);
            let mut simulator: Simulator = simulator.clone();
            let error_model = Arc::clone(&error_model);
            let debug_print = Arc::clone(&debug_print);
            let log_runtime_statistics_file = log_runtime_statistics_file.clone();
            let mut mwpm_decoder = mwpm_decoder.clone();
            let mut tailored_mwpm_decoder = tailored_mwpm_decoder.clone();
            let thread_ended = Arc::new(AtomicBool::new(false));
            threads_ended.push(Arc::clone(&thread_ended));
            let thread_debugger = Arc::new(Mutex::new(BenchmarkThreadDebugger::new()));
            threads_debugger.push(thread_debugger.clone());
            handlers.push(std::thread::spawn(move || {
                for thread_counter in 0..usize::MAX {
                    if thread_timeout >= 0. { thread_debugger.lock().unwrap().update_thread_counter(thread_counter); }
                    // generate random errors and the corresponding measurement
                    let begin = Instant::now();
                    simulator.generate_random_errors(&error_model);
                    if thread_timeout >= 0. { thread_debugger.lock().unwrap().error_pattern = Some(simulator.generate_sparse_error_pattern()); }  // runtime debug: find deadlock cases
                    if matches!(*debug_print, Some(BenchmarkDebugPrint::AllErrorPattern)) {
                        let sparse_error_pattern = simulator.generate_sparse_error_pattern();
                        eprintln!("{}", serde_json::to_string(&sparse_error_pattern).expect("serialize should success"));
                    }
                    let sparse_measurement = simulator.generate_sparse_measurement();
                    if thread_timeout >= 0. { thread_debugger.lock().unwrap().measurement = Some(sparse_measurement.clone()); }  // runtime debug: find deadlock cases
                    let prepare_elapsed = begin.elapsed().as_secs_f64();
                    // decode
                    let begin = Instant::now();
                    let (correction, mut runtime_statistics) = match decoder {
                        BenchmarkDecoder::None => {
                            (SparseCorrection::new(), json!({}))
                        },
                        BenchmarkDecoder::MWPM => {
                            mwpm_decoder.as_mut().unwrap().decode(&sparse_measurement)
                        },
                        BenchmarkDecoder::TailoredMWPM => {
                            tailored_mwpm_decoder.as_mut().unwrap().decode(&sparse_measurement)
                        },
                    };
                    if thread_timeout >= 0. { thread_debugger.lock().unwrap().correction = Some(correction.clone()); }  // runtime debug: find deadlock cases
                    let decode_elapsed = begin.elapsed().as_secs_f64();
                    // validate correction
                    let begin = Instant::now();
                    let mut is_qec_failed = false;
                    let (logical_i, logical_j) = simulator.validate_correction(&correction);
                    if logical_i && !ignore_logical_i {
                        is_qec_failed = true;
                    }
                    if logical_j && !ignore_logical_j {
                        is_qec_failed = true;
                    }
                    let validate_elapsed = begin.elapsed().as_secs_f64();
                    // update statistic information
                    if let Some(log_runtime_statistics_file) = &log_runtime_statistics_file {
                        runtime_statistics["qec_failed"] = json!(is_qec_failed);
                        if log_error_pattern_when_logical_error && is_qec_failed {
                            runtime_statistics["error_pattern"] = json!(simulator.generate_sparse_error_pattern());
                        }
                        runtime_statistics["elapsed"] = json!({
                            "prepare": prepare_elapsed,
                            "decode": decode_elapsed,
                            "validate": validate_elapsed,
                        });
                        let to_be_written = format!("{}\n", runtime_statistics.to_string());
                        let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                        log_runtime_statistics_file.write(to_be_written.as_bytes()).unwrap();
                    }
                    // update simulation counters, then break the loop if benchmark should terminate
                    if benchmark_control.lock().unwrap().update_data_should_terminate(is_qec_failed, max_repeats, min_failed_cases) {
                        break
                    }
                }
                thread_ended.store(true, Ordering::SeqCst);
            }));
        }
        // monitor results and display them using progress bar
        let repeat_begin = Instant::now();
        let progress_information = || -> String {
            let benchmark_control = benchmark_control.lock().unwrap().clone();
            let total_repeats = benchmark_control.total_repeats;
            let qec_failed = benchmark_control.qec_failed;
            // compute simulation results
            let error_rate = qec_failed as f64 / total_repeats as f64;
            let confidence_interval_95_percent = 1.96 * (error_rate * (1. - error_rate) / (total_repeats as f64)).sqrt() / error_rate;
            format!("{} {} {} {} {} {} {} {:.1e} {} ", p, di, noisy_measurements, total_repeats, qec_failed, error_rate, dj
                , confidence_interval_95_percent, pe)
        };
        loop {
            let time_elapsed = repeat_begin.elapsed().as_secs_f64();
            match time_budget {
                Some(time_budget) => {
                    if time_elapsed > time_budget {
                        benchmark_control.lock().unwrap().set_external_terminate();
                    }
                }, _ => { }
            }
            // compute simulation results
            pb.message(progress_information().as_str());
            {  // estimate running time cleverer
                let benchmark_control = benchmark_control.lock().unwrap().clone();
                let total_repeats = benchmark_control.total_repeats;
                let qec_failed = benchmark_control.qec_failed;
                let ratio_total_rounds = (total_repeats as f64) / (max_repeats as f64);
                let ratio_qec_failed = (qec_failed as f64) / (min_failed_cases as f64);
                let (mut pb_total, mut set_progress) = 
                if ratio_total_rounds >= ratio_qec_failed {
                    let progress = total_repeats as u64;
                    (if max_repeats as u64 > progress { max_repeats as u64 } else { progress }, progress)
                } else {
                    let progress = qec_failed as u64;
                    (if min_failed_cases as u64 > progress { min_failed_cases as u64 } else { progress }, progress)
                };
                match time_budget {
                    Some(time_budget) => {
                        let ratio_time = time_elapsed / time_budget;
                        if ratio_time >= ratio_total_rounds && ratio_time >= ratio_qec_failed {
                            let progress = total_repeats as u64;
                            pb_total = ((progress as f64) / ratio_time) as u64;
                            set_progress = progress;
                        }
                    }, _ => { }
                }
                // update progress bar only once, to avoid misleading outputs in stderr (although not visible for human when running it, it will be included in stderr file)
                pb.total = pb_total;
                pb.set(set_progress);
            }
            // synchronize statistics log file to make sure data is not lost when interrupting
            if let Some(log_runtime_statistics_file) = &log_runtime_statistics_file {
                let log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                log_runtime_statistics_file.sync_data().unwrap();
            }
            if benchmark_control.lock().unwrap().should_terminate(max_repeats, min_failed_cases) {
                break
            }
            // refresh 4 times per second
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        // wait for all threads to terminate until timeout
        let begin = Instant::now();
        loop {
            let time_elapsed = begin.elapsed().as_secs_f64();
            if thread_timeout >= 0. && time_elapsed >= thread_timeout {  // abnormal break because of timeout
                eprintln!("[error] some threads don't terminate properly within timeout, here are the details:");
                for parallel_idx in (0..parallel).rev() {
                    let thread_ended = threads_ended.swap_remove(parallel_idx);
                    let handler = handlers.swap_remove(parallel_idx);
                    let thread_debugger = threads_debugger.swap_remove(parallel_idx);
                    if !thread_ended.load(Ordering::SeqCst) {
                        eprintln!("[error] thread {} doesn't terminate within timeout", parallel_idx);
                        eprintln!("{}", json!(thread_debugger.lock().unwrap().clone()));
                    } else {  // still join normal threads
                        eprintln!("[info] thread {} normally exit", parallel_idx);
                        handler.join().unwrap();
                    }
                }
                break
            }
            // check if all threads ended before break the loop
            let mut all_threads_ended = true;
            for thread_ended in threads_ended.iter() {
                if !thread_ended.load(Ordering::SeqCst) {
                    all_threads_ended = false;
                }
            }
            if all_threads_ended {  // only when all threads ended normally will it joina
                for handler in handlers.drain(..) {
                    handler.join().unwrap();
                }
                break
            }
            eprintln!("[info] waiting for all threads to end, time elapsed: {:.3}s", time_elapsed);
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        pb.finish();
        eprintln!("{}", progress_information());
        output += &format!("\n{}\n", progress_information());
    }
    output
}

/**
default example:
`cargo run --release -- tool fault_tolerant_benchmark [5] [5] [1e-3]`
it supports progress bar (in stderr), so you can run this in backend by redirect stdout to a file. This will not contain information of dynamic progress
**/
fn fault_tolerant_benchmark(dis: &Vec<usize>, djs: &Vec<usize>, Ts: &Vec<usize>, ps: &Vec<f64>, pes: &Vec<f64>, max_N: usize, min_error_cases: usize
        , parallel: usize, validate_layer: String, mini_sync_time: f64, autotune: bool, rotated_planar_code: bool, ignore_6_neighbors: bool
        , extra_measurement_error: f64, bypass_correction: bool, independent_px_pz: bool, only_count_logical_x: bool, only_count_logical_z: bool
        , perfect_initialization: bool, shallow_error_on_bottom: bool, no_y_error: bool, use_xzzx_code: bool, use_rotated_tailored_code: bool, bias_eta: f64, decoder_type: DecoderType
        , max_half_weight: usize, use_combined_probability: bool, autotune_minus_no_error: bool, error_model: Option<ErrorModelName>
        , error_model_configuration: Option<serde_json::Value>, no_stop_if_next_model_is_not_prepared: bool, log_runtime_statistics: Option<String>
        , detailed_runtime_statistics: bool, log_error_pattern_into_statistics_when_has_logical_error: bool, time_budget: Option<f64>, use_fast_benchmark: bool
        , fbench_disable_additional_error: bool, fbench_use_fake_decoder: bool, fbench_use_simple_sum: bool, fbench_assignment_sampling_amount: usize
        , fbench_weighted_path_sampling: bool, fbench_weighted_assignment_sampling: bool, fbench_target_dev: f64, rug_precision: u32
        , disable_optimize_correction_pattern: bool, debug_print_only: bool, debug_print_direct_connections: bool, debug_print_exhausted_connections: bool
        , debug_print_error_model: bool, debug_print_with_all_possible_error_rates: bool, use_reduced_graph: bool, error_model_modifier: Option<serde_json::Value>) -> String {
    let mut output = format!("");  // empty output string
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    // check fast benchmark parameters
    if fbench_disable_additional_error || fbench_use_fake_decoder {
        assert!(use_fast_benchmark, "fast benchmark must be enabled to use additional parameters");
    }
    if fbench_use_fake_decoder {
        assert!(fbench_disable_additional_error, "fake decoder only works when the additional error is disabled");
    }
    // create runtime statistics file of specified
    let log_runtime_statistics_file = log_runtime_statistics.map(|filename| 
        Arc::new(Mutex::new(File::create(filename.as_str()).expect("cannot create file"))));
    let fixed_configuration = json!({
        "max_N": max_N,
        "min_error_cases": min_error_cases,
        "parallel": parallel,
        "validate_layer": validate_layer,
        "autotune": autotune,
        "rotated_planar_code": rotated_planar_code,
        "ignore_6_neighbors": ignore_6_neighbors,
        "extra_measurement_error": extra_measurement_error,
        "bypass_correction": bypass_correction,
        "independent_px_pz": independent_px_pz,
        "only_count_logical_x": only_count_logical_x,
        "only_count_logical_z": only_count_logical_z,
        "perfect_initialization": perfect_initialization,
        "shallow_error_on_bottom": shallow_error_on_bottom,
        "no_y_error": no_y_error,
        "use_xzzx_code": use_xzzx_code,
        "use_rotated_tailored_code": use_rotated_tailored_code,
        "bias_eta": bias_eta,
        "decoder_type": format!("{:?}", decoder_type),
        "max_half_weight": max_half_weight,
        "use_combined_probability": use_combined_probability,
        "autotune_minus_no_error": autotune_minus_no_error,
        "error_model": format!("{:?}", error_model),
        "no_stop_if_next_model_is_not_prepared": no_stop_if_next_model_is_not_prepared,
        "detailed_runtime_statistics": detailed_runtime_statistics,
        "log_error_pattern_into_statistics_when_has_logical_error": log_error_pattern_into_statistics_when_has_logical_error,
        "time_budget": format!("{:?}", time_budget),
        "use_fast_benchmark": use_fast_benchmark,
        "fbench_disable_additional_error": fbench_disable_additional_error,
        "fbench_use_fake_decoder": fbench_use_fake_decoder,
        "fbench_use_simple_sum": fbench_use_simple_sum,
        "fbench_assignment_sampling_amount": fbench_assignment_sampling_amount,
        "fbench_weighted_path_sampling": fbench_weighted_path_sampling,
        "fbench_weighted_assignment_sampling": fbench_weighted_assignment_sampling,
        "fbench_target_dev": fbench_target_dev,
        "rug_precision": rug_precision,
        "disable_optimize_correction_pattern": disable_optimize_correction_pattern,
        "use_reduced_graph": use_reduced_graph,
    });
    match &log_runtime_statistics_file {  // append runtime statistics data
        Some(log_runtime_statistics_file) => {
            let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
            log_runtime_statistics_file.write(b"#f ").unwrap();
            log_runtime_statistics_file.write(fixed_configuration.to_string().as_bytes()).unwrap();
            log_runtime_statistics_file.write(b"\n").unwrap();
            log_runtime_statistics_file.sync_data().unwrap();
        }, _ => { },
    }
    if !debug_print_only {  // debug print only will not run simulations
        println!("format: <p> <di> <T> <total_rounds> <qec_failed> <error_rate> <dj> <confidence_interval_95_percent> <pe>");
    }
    // first list all configurations
    assert_eq!(pes.len(), ps.len(), "pe and p should be matched");
    let mut configurations = Vec::new();
    for (di_idx, di) in dis.iter().enumerate() {
        let MeasurementRounds = Ts[di_idx];
        let dj = djs[di_idx];
        for (p_idx, p) in ps.iter().enumerate() {
            let p = *p;
            let pe = pes[p_idx];
            assert!(p >= 0. && p <= 1.0, "invalid probability value");
            assert!(pe >= 0. && pe <= 1.0, "invalid probability value");
            configurations.push((*di, dj, MeasurementRounds, p, pe));
        }
    }
    let weight_function = if autotune {
        if autotune_minus_no_error {
            ftqec::weight_autotune_minus_no_error
        } else {
            ftqec::weight_autotune
        }
    } else {
        ftqec::weight_equal
    };
    let configurations_len = configurations.len();
    let compute_model = Arc::new(move |di: usize, dj: usize, MeasurementRounds: usize, p: f64, pe: f64| {
        // build general models
        let mut model = if rotated_planar_code {
            if use_xzzx_code && !use_rotated_tailored_code {
                assert_eq!(di, dj, "rotated XZZX code doesn't support rectangle lattice yet");
                ftqec::PlanarCodeModel::new_rotated_XZZX_code(MeasurementRounds, di)
            } else if !use_xzzx_code && !use_rotated_tailored_code {
                assert_eq!(di, dj, "rotated planar code doesn't support rectangle lattice yet");
                ftqec::PlanarCodeModel::new_rotated_planar_code(MeasurementRounds, di)
            } else {
                panic!("conflict parameters: --rotated_planar_code, --use_xzzx_code and --use_rotated_tailored_code")
            }
        } else {
            if use_xzzx_code && !use_rotated_tailored_code {
                ftqec::PlanarCodeModel::new_standard_XZZX_code_rectangle(MeasurementRounds, di, dj)
            } else if !use_xzzx_code && use_rotated_tailored_code {
                assert_eq!(di, dj, "rotated tailored code doesn't support rectangle lattice yet");
                ftqec::PlanarCodeModel::new_rotated_tailored_code(MeasurementRounds, di)
            } else if !use_xzzx_code && !use_rotated_tailored_code {
                assert_eq!(di, dj, "standard planar code doesn't support rectangle lattice yet");
                ftqec::PlanarCodeModel::new_standard_planar_code(MeasurementRounds, di)
            } else {
                panic!("conflict parameters: --rotated_planar_code, --use_xzzx_code and --use_rotated_tailored_code")
            }
        };
        model.use_combined_probability = use_combined_probability;
        model.use_reduced_graph = use_reduced_graph;
        // compute pz, px, py individually given bias_eta
        // bias_eta = pz / (px + py) and px = py, px + py + pz = p
        // (px + py) * (1 + bias_eta) = p
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        // println!("px = {}, py = {}, pz = {}", px, py, pz);
        // initialize error rate
        if !perfect_initialization {
            model.set_individual_error_with_erasure(px, py, pz, pe);
        } else {
            // if we use the `set_depolarizing_error` model, then old judgement doesn't work
            // in order to verify that the modification is good, here we mimic the behavior of old model
            // that is, we do not generate error on the added bottom layer, so that there is no bottom boundary
            model.set_individual_error_with_perfect_initialization_with_erasure(px, py, pz, pe);
        }
        if shallow_error_on_bottom {
            model.iterate_snapshot_mut(|t, _i, _j, node| {
                if t == 6 && node.qubit_type == QubitType::Data {
                    node.error_rate_x = px;
                    node.error_rate_z = pz;
                    node.error_rate_y = py;
                    node.erasure_error_rate = pe;
                }
            })
        }
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t % 6 == 5 && node.qubit_type != QubitType::Data {  // just add error before the measurement stage
                node.error_rate_x *= extra_measurement_error;
                node.error_rate_z *= extra_measurement_error;
                node.error_rate_y *= extra_measurement_error;
            }
            if independent_px_pz {
                node.error_rate_y = node.error_rate_x * node.error_rate_z;
            }
            if no_y_error {
                node.error_rate_y = 0.;
            }
        });
        match &error_model {
            Some(error_model) => {
                model.apply_error_model(error_model, error_model_configuration.as_ref(), p, bias_eta, pe);
            },
            None => { }
        }
        match &error_model_modifier {
            Some(modifier) => {
                match model.apply_error_model_modifier(&modifier) {
                    Ok(_) => { },
                    Err(reason) => {
                        panic!("[error] apply error model failed: {}", reason);
                    },
                }
            },
            None => { }
        }
        if debug_print_with_all_possible_error_rates {
            model.iterate_snapshot_mut(|_t, _i, _j, node| {
                if node.connection.is_some() {
                    if node.correlated_error_model.is_none() {
                        node.correlated_error_model = Some(CorrelatedPauliErrorRates::default_with_probability(0.));
                    }
                    if node.correlated_erasure_error_model.is_none() {
                        node.correlated_erasure_error_model = Some(CorrelatedErasureErrorRates::default_with_probability(0.));
                    }
                }
            });
        }
        let fast_benchmark = if !bypass_correction {
            let mut fast_benchmark = model.build_graph_fast_benchmark(weight_function, use_fast_benchmark);
            if use_fast_benchmark {
                fast_benchmark.as_mut().unwrap().assignment_sampling_amount = fbench_assignment_sampling_amount;
                fast_benchmark.as_mut().unwrap().use_weighted_path_sampling = fbench_weighted_path_sampling;
                fast_benchmark.as_mut().unwrap().use_weighted_assignment_sampling = fbench_weighted_assignment_sampling;
                fast_benchmark.as_mut().unwrap().use_simple_sum = fbench_use_simple_sum;
                fast_benchmark.as_mut().unwrap().prepare();
            }
            if ignore_6_neighbors {
                model.iterate_snapshot_mut(|t, i, j, node| {
                    if node.edges.len() == 12 {
                        let mut modified_edges = Vec::new();
                        for edge in node.edges.drain(..) {
                            let tc = t != edge.t;
                            let ic = i != edge.i;
                            let jc = j != edge.j;
                            if (tc && !ic && !jc) || (!tc && ic && !jc) || (!tc && !ic && jc) {
                                modified_edges.push(edge);
                            }
                        }
                        assert!(modified_edges.len() <= 6, "we keep only 6 neighbors");
                        node.edges = modified_edges;
                    }
                });
            }
            fast_benchmark
        } else {
            None
        };
        let model_error = model.clone();  // avoid copying decoding structure a lot of times
        if !bypass_correction && !disable_optimize_correction_pattern {
            model.optimize_correction_pattern();
        }
        if !bypass_correction && !debug_print_error_model {
            model.build_exhausted_path();
        }
        (model, model_error, fast_benchmark)
    });
    let precomputed_model = Arc::new(Mutex::new(None));
    for i in 0..configurations_len {
        let (di, dj, MeasurementRounds, p, pe) = configurations[i];
        match &log_runtime_statistics_file {  // append runtime statistics data
            Some(log_runtime_statistics_file) => {
                let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                log_runtime_statistics_file.write(b"# ").unwrap();
                log_runtime_statistics_file.write(json!({
                    "di": di,
                    "dj": dj,
                    "MeasurementRounds": MeasurementRounds,
                    "p": p,
                    "pe": pe,
                }).to_string().as_bytes()).unwrap();
                log_runtime_statistics_file.write(b"\n").unwrap();
                log_runtime_statistics_file.sync_data().unwrap();
            }, _ => { },
        }
        if i == 0 {  // only i == 0 need to compute model immediately
            let mut precomputed_model = precomputed_model.lock().unwrap();
            *precomputed_model = Some((*compute_model)(di, dj, MeasurementRounds, p, pe));
        }
        let (model, model_error, fast_benchmark) = {  // must already prepared the model, and will take the value out of `precomputed_model`
            precomputed_model.lock().unwrap().take().expect("already prepared the model")
        };
        if debug_print_error_model {
            output += &format!("{}\n", serde_json::to_string(&model).expect("serialize should success"));
        }
        if debug_print_direct_connections {
            model.print_direct_connections();
        }
        if debug_print_exhausted_connections {
            model.print_exhausted_connections();
        }
        // create threads to run experiment
        if !debug_print_only {
            let total_rounds = Arc::new(Mutex::new(0));
            let qec_failed = Arc::new(Mutex::new(0));
            let external_termination = Arc::new(Mutex::new(false));
            let mut precomputing_model_thread = None;
            if i + 1 < configurations_len {
                let (di_next, dj_next, measurement_rounds_next, p_next, pe_next) = configurations[i + 1];
                let precomputed_model = Arc::clone(&precomputed_model);
                let compute_model = Arc::clone(&compute_model);
                // create a single thread to prepare next model
                precomputing_model_thread = Some(std::thread::spawn(move || {
                    let (model, model_error, fast_benchmark) = (*compute_model)(di_next, dj_next, measurement_rounds_next, p_next, pe_next);
                    // lock only after model is built, otherwise it will block experimenting threads
                    let mut precomputed_model = precomputed_model.lock().unwrap();
                    *precomputed_model = Some((model, model_error, fast_benchmark));
                }));
            }
            let mut pb = ProgressBar::on(std::io::stderr(), max_N as u64);
            pb.set(0);
            let mut handlers = Vec::new();
            let model_decoder = Arc::new(model);  // only for decode, so that you're confident I'm not cheating by using information of original errors
            let fast_benchmark_results = Arc::new(Mutex::new(Vec::new()));  // (result, updated)
            {
                let mut fast_benchmark_results = fast_benchmark_results.lock().unwrap();
                for _ in 0..parallel {
                    fast_benchmark_results.push((rug::Float::with_val(rug_precision, 0.), false));
                }
            }
            for parallel_idx in 0..parallel {
                let total_rounds = Arc::clone(&total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let external_termination = Arc::clone(&external_termination);
                let precomputed_model = Arc::clone(&precomputed_model);
                let mut model_error = model_error.clone();  // only for generating error and validating correction
                let model_decoder = Arc::clone(&model_decoder);  // only for decode, so that you're confident I'm not cheating by using information of original errors
                let mut fast_benchmark = fast_benchmark.clone();
                let fast_benchmark_results = Arc::clone(&fast_benchmark_results);
                let log_runtime_statistics_file = log_runtime_statistics_file.clone();
                let validate_layer: isize = match validate_layer.as_str() {
                    "boundary" => -2,
                    "all" => -1,
                    "bottom" => 0,
                    "top" => MeasurementRounds as isize,
                    _ => validate_layer.parse::<isize>().expect("integer"),
                };
                let decoder_type = decoder_type.clone();
                handlers.push(std::thread::spawn(move || {
                    // println!("thread {}", _i);
                    let mut slow_rng = thread_rng();
                    let mut rng = Xoroshiro128StarStar::seed_from_u64(slow_rng.gen::<u64>());
                    let mut rng_fast_benchmark = Xoroshiro128StarStar::seed_from_u64(slow_rng.gen::<u64>());
                    let mut current_external_termination = {
                        *external_termination.lock().unwrap()
                    };
                    let mut current_total_rounds = {
                        *total_rounds.lock().unwrap()
                    };
                    let mut current_qec_failed = {
                        *qec_failed.lock().unwrap()
                    };
                    let mut keep_running_next_model_not_prepared = if no_stop_if_next_model_is_not_prepared {
                        i + 1 < configurations_len && precomputed_model.lock().unwrap().is_none()
                    } else {
                        false
                    };
                    while keep_running_next_model_not_prepared || (current_total_rounds < max_N && current_qec_failed < min_error_cases
                            && !current_external_termination) {
                        let mut mini_qec_failed = 0;
                        let mut log_runtime_statistics_buffer = String::new();
                        let mut mini_batch = 0;
                        let mini_batch_begin = Instant::now();
                        // run for at least `mini_sync_time` before sync with outside, to avoid frequent lock
                        while mini_batch_begin.elapsed().as_secs_f64() < mini_sync_time {
                            let mut decode_and_update = |errors: Vec<(usize, usize, usize, Either<Either<ErrorType, CorrelatedPauliErrorType>, ()>)>
                                    , clearance_region: &BTreeSet<(usize, usize, usize)>, _: usize| -> bool {
                                mini_batch += 1;
                                let error_count = if use_fast_benchmark && !fbench_disable_additional_error {
                                    // set clearance region
                                    for &(t, i, j) in clearance_region.iter() {
                                        model_error.snapshot[t][i][j].as_mut().expect("exist").disable_in_random_error_generator = true;
                                    }
                                    // generate errors
                                    model_error.generate_random_errors(|| rng.next_f64());
                                    for (t, i, j, error_type) in errors.iter() {
                                        let (t, i, j) = (*t, *i, *j);
                                        match error_type {
                                            Either::Left(pauli_error) => {
                                                match pauli_error {
                                                    Either::Left(error) => {
                                                        model_error.add_error_at(t, i, j, error).unwrap();
                                                    },
                                                    Either::Right(correlated_error) => {
                                                        model_error.add_correlated_error_at(t, i, j, correlated_error).unwrap();
                                                    },
                                                }
                                            },
                                            Either::Right(_) => {  // erasure error
                                                model_error.add_random_erasure_error_at(t, i, j, || rng.next_f64()).unwrap();
                                            },
                                        }
                                    }
                                    // recover clearance region
                                    for &(t, i, j) in clearance_region.iter() {
                                        model_error.snapshot[t][i][j].as_mut().expect("exist").disable_in_random_error_generator = false;
                                    }
                                    model_error.count_error()
                                } else {
                                    model_error.generate_random_errors(|| rng.next_f64())
                                };
                                if error_count == 0 {
                                    return false;
                                }
                                model_error.propagate_error();
                                let measurement = model_error.generate_measurement();
                                let detected_erasures = model_error.generate_detected_erasures();
                                // use `model_decoder` for decoding, so that it is blind to the real error information
                                let (correction, mut runtime_statistics) = if !bypass_correction {
                                    match decoder_type {
                                        DecoderType::MinimumWeightPerfectMatching => model_decoder.decode_MWPM(&measurement),
                                        DecoderType::UnionFind => model_decoder.decode_UnionFind(&measurement, &detected_erasures
                                            , max_half_weight, false, detailed_runtime_statistics),
                                        DecoderType::DistributedUnionFind => model_decoder.decode_UnionFind(&measurement, &detected_erasures
                                            , max_half_weight, true, detailed_runtime_statistics),
                                        // _ => panic!("unsupported decoder type"),
                                    }
                                } else {
                                    (model_decoder.generate_default_correction(), json!({}))
                                };
                                let mut count_as_error = false;
                                if validate_layer == -2 {
                                    let validation_ret = model_error.validate_correction_on_boundary(&correction);
                                    match validation_ret {
                                        Err(ftqec::ValidationFailedReason::XLogicalError(_, _, _)) => { if !only_count_logical_z {
                                            count_as_error = true;
                                        } },
                                        Err(ftqec::ValidationFailedReason::ZLogicalError(_, _, _)) => { if !only_count_logical_x {
                                            count_as_error = true;
                                        } },
                                        Err(ftqec::ValidationFailedReason::BothXandZLogicalError(_, _, _, _, _)) => {
                                            count_as_error = true;
                                        },
                                        _ => {},
                                    }
                                } else if validate_layer == -1 {
                                    if model_error.validate_correction_on_all_layers(&correction).is_err() {
                                        count_as_error = true;
                                    }
                                } else {
                                    let validation_ret = model_error.validate_correction_on_t_layer(&correction, validate_layer as usize);
                                    match validation_ret {
                                        Err(ftqec::ValidationFailedReason::XLogicalError(_, _, _)) => { if !only_count_logical_z {
                                            count_as_error = true;
                                        } },
                                        Err(ftqec::ValidationFailedReason::ZLogicalError(_, _, _)) => { if !only_count_logical_x {
                                            count_as_error = true;
                                        } },
                                        Err(ftqec::ValidationFailedReason::BothXandZLogicalError(_, _, _, _, _)) => {
                                            count_as_error = true;
                                        },
                                        _ => {},
                                    }
                                }
                                if count_as_error {
                                    mini_qec_failed += 1;
                                }
                                if log_runtime_statistics_file.is_some() {
                                    runtime_statistics["error"] = json!(count_as_error);  // add result into runtime statistics information
                                    if log_error_pattern_into_statistics_when_has_logical_error && count_as_error {
                                        runtime_statistics["error_pattern"] = json!(model_error.get_all_qubit_errors_vec());
                                    }
                                    log_runtime_statistics_buffer.push_str(&runtime_statistics.to_string());
                                    log_runtime_statistics_buffer.push_str(&"\n".to_string())
                                }
                                count_as_error
                            };
                            if use_fast_benchmark {
                                if fbench_use_fake_decoder {
                                    // fast_benchmark.benchmark_once(&mut rng_fast_benchmark, ...);
                                    fast_benchmark.as_mut().unwrap().benchmark_random_starting_node(&mut rng_fast_benchmark, |errors, clearance_region, string_d| {
                                        mini_batch += 1;
                                        let count_as_error = fast_benchmark::fake_decoding(errors, clearance_region, string_d);
                                        if count_as_error {
                                            mini_qec_failed += 1;
                                        }
                                        count_as_error
                                    });  // is same as benchmark_once statistically but gives output quickly
                                } else {
                                    // fast_benchmark.benchmark_once(&mut rng_fast_benchmark, decode_and_update);
                                    fast_benchmark.as_mut().unwrap().benchmark_random_starting_node(&mut rng_fast_benchmark, decode_and_update);  // is same as benchmark_once statistically but gives output quickly
                                }
                            } else {
                                decode_and_update(Vec::new(), &BTreeSet::new(), 0);
                            }
                        }
                        // sync data with outside
                        current_external_termination = {
                            *external_termination.lock().unwrap()
                        };
                        current_total_rounds = {
                            let mut total_rounds = total_rounds.lock().unwrap();
                            *total_rounds += mini_batch;
                            *total_rounds
                        };
                        current_qec_failed = {
                            let mut qec_failed = qec_failed.lock().unwrap();
                            *qec_failed += mini_qec_failed;
                            *qec_failed
                        };
                        if use_fast_benchmark {
                            let mut fast_benchmark_results = fast_benchmark_results.lock().unwrap();
                            fast_benchmark_results[parallel_idx] = (fast_benchmark.as_mut().unwrap().logical_error_rate(), true);
                        }
                        keep_running_next_model_not_prepared = if no_stop_if_next_model_is_not_prepared {
                            i + 1 < configurations_len && precomputed_model.lock().unwrap().is_none()
                        } else {
                            false
                        };
                        match &log_runtime_statistics_file {  // append runtime statistics data
                            Some(log_runtime_statistics_file) => {
                                let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                                log_runtime_statistics_file.write(log_runtime_statistics_buffer.as_bytes()).unwrap();
                                // serde_json::to_writer(&f, &json!(cycle_distribution)).unwrap();
                                log_runtime_statistics_file.sync_data().unwrap();
                            }, _ => { },
                        }
                    }
                }));
            }
            let round_begin = Instant::now();
            let mut fast_benchmark_dev_satisfied = false;
            let mut fast_benchmark_first_total_rounds = 0;
            let fast_benchmark_exit = Arc::new(Mutex::new(false));
            let fast_benchmark_exit_updater = fast_benchmark_exit.clone();
            let mut generate_fast_benchmark_print = |total_rounds: usize| {
                if !use_fast_benchmark {
                    return format!("");
                }
                let fast_benchmark_results = {
                    let fast_benchmark_results = fast_benchmark_results.lock().unwrap();
                    fast_benchmark_results.clone()
                };
                let mut logical_error_rates = Vec::new();
                for (logical_error_rate, updated) in fast_benchmark_results.iter() {
                    if *updated {
                        logical_error_rates.push(logical_error_rate);
                    }
                }
                // calculate mean and stddev of these logical_error_rates
                let mut average_logical_error_rate = rug::Float::with_val(rug_precision, 0.);
                for logical_error_rate in logical_error_rates.iter() {
                    average_logical_error_rate += logical_error_rate.clone();
                }
                let average_logical_error_rate = average_logical_error_rate / (logical_error_rates.len() as f64);
                let mut variance = rug::Float::with_val(rug_precision, 0.);
                for logical_error_rate in logical_error_rates.iter() {
                    variance += (average_logical_error_rate.clone() - logical_error_rate.clone()).pow(2);
                }
                let variance = variance / (logical_error_rates.len() as f64);
                let confidence_interval_95_percent = 1.96 * variance.sqrt() / average_logical_error_rate.clone();
                if confidence_interval_95_percent < fbench_target_dev {
                    if fast_benchmark_dev_satisfied == false {
                        fast_benchmark_first_total_rounds = total_rounds;
                    }
                    fast_benchmark_dev_satisfied = true;
                    if total_rounds > fast_benchmark_first_total_rounds + 100 {
                        *fast_benchmark_exit_updater.lock().unwrap() = true;
                    }
                } else {
                    fast_benchmark_dev_satisfied = false;
                }
                format!("FB {} {} {} {} {} {} {:.8e} {:.2e}", p, pe, di, dj, MeasurementRounds, total_rounds, average_logical_error_rate
                    , confidence_interval_95_percent)
            };
            loop {
                let external_termination = {
                    let mut external_termination = external_termination.lock().unwrap();
                    match time_budget {
                        Some(time_budget) => {
                            if round_begin.elapsed().as_secs_f64() > time_budget {
                                *external_termination = true;
                            }
                        }, _ => { },
                    }
                    if *fast_benchmark_exit.lock().unwrap() {
                        *external_termination = true;
                    }
                    *external_termination
                };
                let total_rounds = *total_rounds.lock().unwrap();
                let qec_failed = *qec_failed.lock().unwrap();
                let fast_benchmark_print = generate_fast_benchmark_print(total_rounds);
                let keep_running_next_model_not_prepared = if no_stop_if_next_model_is_not_prepared {
                    i + 1 < configurations_len && precomputed_model.lock().unwrap().is_none()
                } else {
                    false
                };
                if !(keep_running_next_model_not_prepared || (total_rounds < max_N && qec_failed < min_error_cases && !external_termination)) {
                    break
                }
                let error_rate = qec_failed as f64 / total_rounds as f64;
                let confidence_interval_95_percent = 1.96 * (error_rate * (1. - error_rate) / (total_rounds as f64)).sqrt() / error_rate;
                if use_fast_benchmark {
                    pb.message(format!("{} ", fast_benchmark_print).as_str());
                } else {
                    pb.message(format!("{} {} {} {} {} {} {} {:.1e} {} ", p, di, MeasurementRounds, total_rounds, qec_failed, error_rate, dj
                        , confidence_interval_95_percent, pe).as_str());
                }
                // estimate running time cleverer
                let ratio_total_rounds = (total_rounds as f64) / (max_N as f64);
                let ratio_qec_failed = (qec_failed as f64) / (min_error_cases as f64);
                if ratio_total_rounds > ratio_qec_failed {
                    let progress = total_rounds as u64;
                    pb.total = if max_N as u64 > progress { max_N as u64 } else { progress };
                    pb.set(progress);
                } else {
                    let progress = qec_failed as u64;
                    pb.total = if min_error_cases as u64 > progress { min_error_cases as u64 } else { progress };
                    pb.set(progress);
                }
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
            pb.total = *total_rounds.lock().unwrap() as u64;
            pb.finish();
            for handler in handlers {
                handler.join().unwrap();
            }
            let total_rounds = *total_rounds.lock().unwrap();
            let qec_failed = *qec_failed.lock().unwrap();
            let fast_benchmark_print = generate_fast_benchmark_print(total_rounds);
            let error_rate = qec_failed as f64 / total_rounds as f64;
            let confidence_interval_95_percent = 1.96 * (error_rate * (1. - error_rate) / (total_rounds as f64)).sqrt() / error_rate;
            if use_fast_benchmark {
                println!("{}", fast_benchmark_print);
            } else {
                println!("{} {} {} {} {} {} {} {:.1e} {}", p, di, MeasurementRounds, total_rounds, qec_failed, error_rate, dj, confidence_interval_95_percent, pe);
            }
            match precomputing_model_thread {
                Some(precomputing_model_thread) => precomputing_model_thread.join().unwrap(),
                None => { }
            }
        }
    }
    output
}

/**
default example:
`cargo run --release -- tool offer_decoder_standard_planar_benchmark [5] [1e-3]`
it supports progress bar (in stderr), so you can run this in backend by redirect stdout to a file. This will not contain information of dynamic progress
**/
fn offer_decoder_standard_planar_benchmark(Ls: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, parallel: usize, mini_batch: usize
        , only_count_logical_x: bool, max_resend: usize, max_cycles: usize, disable_probabilistic_accept: bool, repeat_experiment_each_error: usize) {
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    println!("format: <p> <T> <total_rounds> <qec_failed> <error_rate> <average_cycles> <max_cycles>");
    for L in Ls.iter() {
        for p in ps {
            let p = *p;
            assert!(3. * p < 0.5, "why should errors (X, Z, Y) happening more than half of a time?");
            let L = *L;
            let total_rounds = Arc::new(Mutex::new(0));
            let qec_failed = Arc::new(Mutex::new(0));
            let total_cycles = Arc::new(Mutex::new(0));
            let max_cycles_used = Arc::new(Mutex::new(0));
            let mut handlers = Vec::new();
            let mini_batch_count = 1 + max_N / mini_batch;
            let mut pb = ProgressBar::on(std::io::stderr(), mini_batch_count as u64);
            pb.set(0);
            for _i in 0..parallel {
                let total_rounds = Arc::clone(&total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let total_cycles = Arc::clone(&total_cycles);
                let max_cycles_used = Arc::clone(&max_cycles_used);
                let mini_batch = mini_batch;
                let disable_probabilistic_accept = disable_probabilistic_accept;
                let L = L;
                let p = p;
                handlers.push(std::thread::spawn(move || {
                    let mut decoder = offer_decoder::create_standard_planar_code_offer_decoder(L);
                    decoder.disable_probabilistic_accept = disable_probabilistic_accept;
                    let mut rng = thread_rng();
                    let mut current_total_rounds = {
                        *total_rounds.lock().unwrap()
                    };
                    let mut current_qec_failed = {
                        *qec_failed.lock().unwrap()
                    };
                    let mut current_max_cycles_used = 0;
                    while current_total_rounds < max_N && current_qec_failed < min_error_cases {
                        let mut mini_qec_failed = 0;
                        let mut mini_total_cycles = 0;
                        for _j in 0..mini_batch {  // run at least `mini_batch` times before sync with outside
                            decoder.reinitialize();
                            let error_count = decoder.generate_depolarizing_random_errors(p, || rng.gen::<f64>());
                            if error_count == 0 {
                                continue
                            }
                            // repeat experiment multiple times for each error pattern
                            let error_pattern = decoder.error_pattern();
                            let mut succeed_count = 0;
                            let mut valid_count = 0;
                            let mut min_cycles_repeated = usize::MAX;
                            for k in 0..repeat_experiment_each_error {
                                decoder.load_error_pattern(&error_pattern);
                                decoder.error_changed();
                                let mut within_cycles = false;
                                let cycles = match decoder.pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(max_resend, max_cycles) {
                                    Ok(cycles) => {
                                        within_cycles = true;
                                        cycles
                                    },
                                    Err(cycles) => cycles,
                                };
                                if k == 0 || within_cycles {
                                    valid_count += 1;
                                    if cycles < min_cycles_repeated {
                                        min_cycles_repeated = cycles;
                                    }
                                    if only_count_logical_x {
                                        if !decoder.has_logical_error(ErrorType::X) {
                                            succeed_count += 1;
                                        }
                                    } else {  // check for both logical X and logical Z error
                                        if !decoder.has_logical_error(ErrorType::Y) {
                                            succeed_count += 1;
                                        }
                                    }
                                }
                            }
                            mini_total_cycles += min_cycles_repeated;
                            if min_cycles_repeated > current_max_cycles_used {
                                current_max_cycles_used = min_cycles_repeated;
                            }
                            if succeed_count * 2 <= valid_count {  // max vote
                                mini_qec_failed += 1;
                            }
                        }
                        // sync data from outside
                        current_total_rounds = {
                            let mut total_rounds = total_rounds.lock().unwrap();
                            *total_rounds += mini_batch;
                            *total_rounds
                        };
                        current_qec_failed = {
                            let mut qec_failed = qec_failed.lock().unwrap();
                            *qec_failed += mini_qec_failed;
                            *qec_failed
                        };
                        {
                            let mut total_cycles = total_cycles.lock().unwrap();
                            *total_cycles += mini_total_cycles;
                        };
                        {
                            let mut max_cycles_used = max_cycles_used.lock().unwrap();
                            if current_max_cycles_used > *max_cycles_used {
                                *max_cycles_used = current_max_cycles_used;
                            }
                        }
                    }
                }));
            }
            loop {
                let total_rounds = *total_rounds.lock().unwrap();
                if total_rounds >= max_N { break }
                let qec_failed = *qec_failed.lock().unwrap();
                if qec_failed >= min_error_cases { break }
                let error_rate = qec_failed as f64 / total_rounds as f64;
                let total_cycles = *total_cycles.lock().unwrap();
                let average_cycles = total_cycles as f64 / total_rounds as f64;
                let max_cycles_used = *max_cycles_used.lock().unwrap();
                pb.message(format!("{} {} {} {} {} {} {} ", p, L, total_rounds, qec_failed, error_rate, average_cycles, max_cycles_used).as_str());
                let progress = total_rounds / mini_batch;
                pb.set(progress as u64);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            pb.total = (*total_rounds.lock().unwrap() / mini_batch) as u64;
            pb.finish();
            for handler in handlers {
                handler.join().unwrap();
            }
            let total_rounds = *total_rounds.lock().unwrap();
            let qec_failed = *qec_failed.lock().unwrap();
            let error_rate = qec_failed as f64 / total_rounds as f64;
            let total_cycles = *total_cycles.lock().unwrap();
            let average_cycles = total_cycles as f64 / total_rounds as f64;
            let max_cycles_used = *max_cycles_used.lock().unwrap();
            println!("{} {} {} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate, average_cycles, max_cycles_used);
        }
    }
}

/**
default example:
`cargo run --release -- tool offer_algorithm_standard_planar_benchmark [5] [1e-3]`
it supports progress bar (in stderr), so you can run this in backend by redirect stdout to a file. This will not contain information of dynamic progress
**/
fn offer_algorithm_standard_planar_benchmark(Ls: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, parallel: usize, mini_batch: usize
    , only_count_logical_x: bool, max_resend: usize, max_cycles: usize, disable_probabilistic_accept: bool, repeat_experiment_each_error: usize) {
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    println!("format: <p> <T> <total_rounds> <qec_failed> <error_rate> <average_cycles> <max_cycles>");
    for L in Ls.iter() {
        for p in ps {
            let p = *p;
            assert!(3. * p < 0.5, "why should errors (X, Z, Y) happening more than half of a time?");
            let L = *L;
            let total_rounds = Arc::new(Mutex::new(0));
            let qec_failed = Arc::new(Mutex::new(0));
            let total_cycles = Arc::new(Mutex::new(0));
            let max_cycles_used = Arc::new(Mutex::new(0));
            let mut handlers = Vec::new();
            let mini_batch_count = 1 + max_N / mini_batch;
            let mut pb = ProgressBar::on(std::io::stderr(), mini_batch_count as u64);
            pb.set(0);
            for _i in 0..parallel {
                let total_rounds = Arc::clone(&total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let total_cycles = Arc::clone(&total_cycles);
                let max_cycles_used = Arc::clone(&max_cycles_used);
                let mini_batch = mini_batch;
                let disable_probabilistic_accept = disable_probabilistic_accept;
                let L = L;
                let p = p;
                handlers.push(std::thread::spawn(move || {
                    let mut decoder = offer_decoder::create_standard_planar_code_offer_decoder(L);
                    decoder.disable_probabilistic_accept = disable_probabilistic_accept;
                    let mut rng = thread_rng();
                    let mut current_total_rounds = {
                        *total_rounds.lock().unwrap()
                    };
                    let mut current_qec_failed = {
                        *qec_failed.lock().unwrap()
                    };
                    let mut current_max_cycles_used = 0;
                    while current_total_rounds < max_N && current_qec_failed < min_error_cases {
                        let mut mini_qec_failed = 0;
                        let mut mini_total_cycles = 0;
                        for _j in 0..mini_batch {  // run at least `mini_batch` times before sync with outside
                            decoder.reinitialize();
                            let error_count = decoder.generate_depolarizing_random_errors(p, || rng.gen::<f64>());
                            if error_count == 0 {
                                continue
                            }
                            // repeat experiment multiple times for each error pattern
                            let error_pattern = decoder.error_pattern();
                            let mut succeed_count = 0;
                            let mut valid_count = 0;
                            let mut min_cycles_repeated = usize::MAX;
                            for k in 0..repeat_experiment_each_error {
                                decoder.load_error_pattern(&error_pattern);
                                decoder.error_changed();
                                let mut within_cycles = false;
                                let ((_cost_x, cycles_x), (_cost_z, cycles_z)) = offer_mwpm::run_given_offer_decoder_instance(&mut decoder, max_resend, max_cycles);
                                let cycles = match (cycles_x, cycles_z) {
                                    (Ok(cycles_x), Ok(cycles_z)) => {
                                        within_cycles = true;
                                        std::cmp::max(cycles_x, cycles_z)
                                    }
                                    (Ok(cycles_x), Err(cycles_z)) => std::cmp::max(cycles_x, cycles_z),
                                    (Err(cycles_x), Ok(cycles_z)) => std::cmp::max(cycles_x, cycles_z),
                                    (Err(cycles_x), Err(cycles_z)) => std::cmp::max(cycles_x, cycles_z),
                                };
                                if k == 0 || within_cycles {
                                    valid_count += 1;
                                    if cycles < min_cycles_repeated {
                                        min_cycles_repeated = cycles;
                                    }
                                    if only_count_logical_x {
                                        if !decoder.has_logical_error(ErrorType::X) {
                                            succeed_count += 1;
                                        }
                                    } else {  // check for both logical X and logical Z error
                                        if !decoder.has_logical_error(ErrorType::Y) {
                                            succeed_count += 1;
                                        }
                                    }
                                }
                            }
                            mini_total_cycles += min_cycles_repeated;
                            if min_cycles_repeated > current_max_cycles_used {
                                current_max_cycles_used = min_cycles_repeated;
                            }
                            if succeed_count * 2 <= valid_count {  // max vote
                                mini_qec_failed += 1;
                            }
                        }
                        // sync data from outside
                        current_total_rounds = {
                            let mut total_rounds = total_rounds.lock().unwrap();
                            *total_rounds += mini_batch;
                            *total_rounds
                        };
                        current_qec_failed = {
                            let mut qec_failed = qec_failed.lock().unwrap();
                            *qec_failed += mini_qec_failed;
                            *qec_failed
                        };
                        {
                            let mut total_cycles = total_cycles.lock().unwrap();
                            *total_cycles += mini_total_cycles;
                        };
                        {
                            let mut max_cycles_used = max_cycles_used.lock().unwrap();
                            if current_max_cycles_used > *max_cycles_used {
                                *max_cycles_used = current_max_cycles_used;
                            }
                        }
                    }
                }));
            }
            loop {
                let total_rounds = *total_rounds.lock().unwrap();
                if total_rounds >= max_N { break }
                let qec_failed = *qec_failed.lock().unwrap();
                if qec_failed >= min_error_cases { break }
                let error_rate = qec_failed as f64 / total_rounds as f64;
                let total_cycles = *total_cycles.lock().unwrap();
                let average_cycles = total_cycles as f64 / total_rounds as f64;
                let max_cycles_used = *max_cycles_used.lock().unwrap();
                pb.message(format!("{} {} {} {} {} {} {} ", p, L, total_rounds, qec_failed, error_rate, average_cycles, max_cycles_used).as_str());
                let progress = total_rounds / mini_batch;
                pb.set(progress as u64);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            pb.total = (*total_rounds.lock().unwrap() / mini_batch) as u64;
            pb.finish();
            for handler in handlers {
                handler.join().unwrap();
            }
            let total_rounds = *total_rounds.lock().unwrap();
            let qec_failed = *qec_failed.lock().unwrap();
            let error_rate = qec_failed as f64 / total_rounds as f64;
            let total_cycles = *total_cycles.lock().unwrap();
            let average_cycles = total_cycles as f64 / total_rounds as f64;
            let max_cycles_used = *max_cycles_used.lock().unwrap();
            println!("{} {} {} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate, average_cycles, max_cycles_used);
        }
    }
}

/**
default example:
`cargo run --release -- tool union_find_decoder_standard_planar_benchmark [5] [1e-3]`
it supports progress bar (in stderr), so you can run this in backend by redirect stdout to a file. This will not contain information of dynamic progress
**/
fn union_find_decoder_standard_planar_benchmark(Ls: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, parallel: usize, mini_batch: usize
        , only_count_logical_x: bool, no_y_error: bool, towards_mwpm: bool, max_half_weight: usize, bias_eta: f64) {
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    println!("format: <p> <T> <total_rounds> <qec_failed> <error_rate>");
    for L in Ls.iter() {
        for p in ps {
            let p = *p;
            assert!(3. * p < 0.5, "why should errors (X, Z, Y) happening more than half of a time?");
            let L = *L;
            let total_rounds = Arc::new(Mutex::new(0));
            let qec_failed = Arc::new(Mutex::new(0));
            let mut handlers = Vec::new();
            let mini_batch_count = 1 + max_N / mini_batch;
            let mut pb = ProgressBar::on(std::io::stderr(), mini_batch_count as u64);
            pb.set(0);
            for _i in 0..parallel {
                let total_rounds = Arc::clone(&total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let mini_batch = mini_batch;
                let L = L;
                let p = p;
                handlers.push(std::thread::spawn(move || {
                    let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(1, L);
                    let px = p / (1. + bias_eta) / 2.;
                    let py = px;
                    let pz = p - 2. * px;
                    model.set_individual_error_with_perfect_initialization(0., 0., 0.);
                    // shallow_error_on_bottom
                    model.iterate_snapshot_mut(|t, _i, _j, node| {
                        if t == 12 && node.qubit_type == QubitType::Data {
                            node.error_rate_x = px;
                            node.error_rate_z = pz;
                            if no_y_error {
                                node.error_rate_y = 0.;
                            } else {
                                node.error_rate_y = py;
                            }
                        }
                    });
                    model.build_graph(ftqec::weight_autotune);
                    let mut rng = thread_rng();
                    let mut current_total_rounds = {
                        *total_rounds.lock().unwrap()
                    };
                    let mut current_qec_failed = {
                        *qec_failed.lock().unwrap()
                    };
                    while current_total_rounds < max_N && current_qec_failed < min_error_cases {
                        let mut mini_qec_failed = 0;
                        for _j in 0..mini_batch {  // run at least `mini_batch` times before sync with outside
                            let error_count = model.generate_random_errors(|| rng.gen::<f64>());
                            if error_count == 0 {
                                continue
                            }
                            model.propagate_error();
                            let (has_x_logical_error, has_z_logical_error) = union_find_decoder::run_given_mwpm_decoder_instance_weighted(&mut model
                                , towards_mwpm, max_half_weight, false);
                            if only_count_logical_x {
                                if has_x_logical_error {
                                    mini_qec_failed += 1;
                                }
                            } else {
                                if has_x_logical_error || has_z_logical_error {
                                    mini_qec_failed += 1;
                                }
                            }
                        }
                        // sync data from outside
                        current_total_rounds = {
                            let mut total_rounds = total_rounds.lock().unwrap();
                            *total_rounds += mini_batch;
                            *total_rounds
                        };
                        current_qec_failed = {
                            let mut qec_failed = qec_failed.lock().unwrap();
                            *qec_failed += mini_qec_failed;
                            *qec_failed
                        };
                    }
                }));
            }
            loop {
                let total_rounds = *total_rounds.lock().unwrap();
                if total_rounds >= max_N { break }
                let qec_failed = *qec_failed.lock().unwrap();
                if qec_failed >= min_error_cases { break }
                let error_rate = qec_failed as f64 / total_rounds as f64;
                pb.message(format!("{} {} {} {} {} ", p, L, total_rounds, qec_failed, error_rate).as_str());
                let progress = total_rounds / mini_batch;
                pb.set(progress as u64);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            pb.total = (*total_rounds.lock().unwrap() / mini_batch) as u64;
            pb.finish();
            for handler in handlers {
                handler.join().unwrap();
            }
            let total_rounds = *total_rounds.lock().unwrap();
            let qec_failed = *qec_failed.lock().unwrap();
            let error_rate = qec_failed as f64 / total_rounds as f64;
            println!("{} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate);
        }
    }
}

/**
default example:
`cargo run --release -- tool distributed_union_find_decoder_standard_planar_benchmark [5] [1e-3]`
it supports progress bar (in stderr), so you can run this in backend by redirect stdout to a file. This will not contain information of dynamic progress
**/
fn distributed_union_find_decoder_standard_planar_benchmark(Ls: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, parallel: usize, mini_batch: usize
    , only_count_logical_x: bool, output_cycle_distribution: bool, fast_channel_interval: usize, no_y_error: bool) {
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    println!("format: <p> <T> <total_rounds> <qec_failed> <error_rate> <average_cycles> <max_cycles>");
    for L in Ls.iter() {
        for p in ps {
            let p = *p;
            assert!(3. * p < 0.5, "why should errors (X, Z, Y) happening more than half of a time?");
            let L = *L;
            let total_rounds = Arc::new(Mutex::new(0));
            let qec_failed = Arc::new(Mutex::new(0));
            let total_cycles = Arc::new(Mutex::new(0));
            let max_cycles_used = Arc::new(Mutex::new(0));
            let cycle_distribution = Arc::new(Mutex::new(Vec::<(usize, usize)>::new()));
            let mut handlers = Vec::new();
            let mini_batch_count = 1 + max_N / mini_batch;
            let mut pb = ProgressBar::on(std::io::stderr(), mini_batch_count as u64);
            pb.set(0);
            for _i in 0..parallel {
                let total_rounds = Arc::clone(&total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let total_cycles = Arc::clone(&total_cycles);
                let max_cycles_used = Arc::clone(&max_cycles_used);
                let cycle_distribution = Arc::clone(&cycle_distribution);
                let mini_batch = mini_batch;
                let L = L;
                let p = p;
                handlers.push(std::thread::spawn(move || {
                    let mut decoder = offer_decoder::create_standard_planar_code_offer_decoder(L);
                    let mut rng = thread_rng();
                    let mut current_total_rounds = {
                        *total_rounds.lock().unwrap()
                    };
                    let mut current_qec_failed = {
                        *qec_failed.lock().unwrap()
                    };
                    let mut current_max_cycles_used = 0;
                    while current_total_rounds < max_N && current_qec_failed < min_error_cases {
                        let mut mini_qec_failed = 0;
                        let mut mini_total_cycles = 0;
                        let mut mini_cycle_distribution = Vec::<(usize, usize)>::new();
                        for _j in 0..mini_batch {  // run at least `mini_batch` times before sync with outside
                            decoder.reinitialize();
                            let error_count = if no_y_error {
                                assert!(only_count_logical_x, "not implemented if z errors needed");
                                decoder.generate_only_x_random_errors(p, || rng.gen::<f64>())
                            } else {
                                decoder.generate_depolarizing_random_errors(p, || rng.gen::<f64>())
                            };
                            if error_count == 0 {
                                continue
                            }
                            let (has_x_logical_error, has_z_logical_error, cycle) = 
                                distributed_uf_decoder::run_given_offer_decoder_instance_with_cycle(&mut decoder, fast_channel_interval);
                            if only_count_logical_x {
                                if has_x_logical_error {
                                    mini_qec_failed += 1;
                                }
                                if output_cycle_distribution {
                                    mini_cycle_distribution.resize(std::cmp::max(mini_cycle_distribution.len(), cycle + 1), (0, 0));
                                    if has_x_logical_error { mini_cycle_distribution[cycle].1 += 1; } else { mini_cycle_distribution[cycle].0 += 1; }
                                }
                            } else {
                                if has_x_logical_error || has_z_logical_error {
                                    mini_qec_failed += 1;
                                }
                                if output_cycle_distribution {
                                    mini_cycle_distribution.resize(std::cmp::max(mini_cycle_distribution.len(), cycle + 1), (0, 0));
                                    if has_x_logical_error || has_z_logical_error { mini_cycle_distribution[cycle].1 += 1; } else { mini_cycle_distribution[cycle].0 += 1; }
                                }
                            }
                            mini_total_cycles += cycle;
                            if cycle > current_max_cycles_used {
                                current_max_cycles_used = cycle;
                            }
                        }
                        // sync data from outside
                        current_total_rounds = {
                            let mut total_rounds = total_rounds.lock().unwrap();
                            *total_rounds += mini_batch;
                            *total_rounds
                        };
                        current_qec_failed = {
                            let mut qec_failed = qec_failed.lock().unwrap();
                            *qec_failed += mini_qec_failed;
                            *qec_failed
                        };
                        {
                            let mut total_cycles = total_cycles.lock().unwrap();
                            *total_cycles += mini_total_cycles;
                        };
                        {
                            let mut max_cycles_used = max_cycles_used.lock().unwrap();
                            if current_max_cycles_used > *max_cycles_used {
                                *max_cycles_used = current_max_cycles_used;
                            }
                        }
                        if output_cycle_distribution {
                            let mut cycle_distribution = cycle_distribution.lock().unwrap();
                            let extended_length = std::cmp::max(mini_cycle_distribution.len(), cycle_distribution.len());
                            cycle_distribution.resize(extended_length, (0, 0));
                            for di in 0..mini_cycle_distribution.len() {
                                cycle_distribution[di].0 += mini_cycle_distribution[di].0;
                                cycle_distribution[di].1 += mini_cycle_distribution[di].1;
                            }
                        }
                    }
                }));
            }
            loop {
                let total_rounds = *total_rounds.lock().unwrap();
                if total_rounds >= max_N { break }
                let qec_failed = *qec_failed.lock().unwrap();
                if qec_failed >= min_error_cases { break }
                let error_rate = qec_failed as f64 / total_rounds as f64;
                let total_cycles = *total_cycles.lock().unwrap();
                let average_cycles = total_cycles as f64 / total_rounds as f64;
                let max_cycles_used = *max_cycles_used.lock().unwrap();
                if output_cycle_distribution {
                    // save cycle distribution to file
                    let cycle_distribution = cycle_distribution.lock().unwrap().clone();
                    let f = File::create(format!("duf_{}_{}.json", L, p)).unwrap();
                    serde_json::to_writer(&f, &json!(cycle_distribution)).unwrap();
                    f.sync_all().unwrap();
                }
                // update progress bar
                pb.message(format!("{} {} {} {} {} {} {} ", p, L, total_rounds, qec_failed, error_rate, average_cycles, max_cycles_used).as_str());
                let progress = total_rounds / mini_batch;
                pb.set(progress as u64);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            pb.total = (*total_rounds.lock().unwrap() / mini_batch) as u64;
            pb.finish();
            for handler in handlers {
                handler.join().unwrap();
            }
            let total_rounds = *total_rounds.lock().unwrap();
            let qec_failed = *qec_failed.lock().unwrap();
            let error_rate = qec_failed as f64 / total_rounds as f64;
            let total_cycles = *total_cycles.lock().unwrap();
            let average_cycles = total_cycles as f64 / total_rounds as f64;
            let max_cycles_used = *max_cycles_used.lock().unwrap();
            println!("{} {} {} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate, average_cycles, max_cycles_used);
        }
    }
}

impl BenchmarkDebugPrint {
    pub fn possible_values<'a>() -> impl Iterator<Item = PossibleValue<'a>> {
        Self::value_variants().iter().filter_map(ArgEnum::to_possible_value)
    }
}

impl std::str::FromStr for BenchmarkDebugPrint {
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

impl BenchmarkDecoder {
    pub fn possible_values<'a>() -> impl Iterator<Item = PossibleValue<'a>> {
        Self::value_variants().iter().filter_map(ArgEnum::to_possible_value)
    }
}

impl std::str::FromStr for BenchmarkDecoder {
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
