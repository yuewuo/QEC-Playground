#![allow(non_snake_case)]

use super::clap;
use super::serde_json;
#[cfg(feature="python_binding")]
use super::pyo3::prelude::*;
use super::num_cpus;
use std::sync::{Arc, Mutex};
use super::pbr::ProgressBar;
use super::serde_json::{json};
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;
use super::util::local_get_temporary_store;
use std::fs;
use super::code_builder::*;
use super::simulator::*;
use super::clap::{ArgEnum, PossibleValue};
use std::sync::atomic::{AtomicBool, Ordering};
use super::error_model::*;
use serde::{Serialize, Deserialize};
use super::decoder_mwpm::*;
use super::decoder_fusion::*;
use super::model_graph::*;
use super::complete_model_graph::*;
use super::decoder_tailored_mwpm::*;
use super::tailored_model_graph::*;
use super::tailored_complete_model_graph::*;
use super::error_model_builder::*;
use super::decoder_union_find::*;
use super::erasure_graph::*;

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
            let ps_graph: Option<String> = matches.value_of_t("ps_graph").ok();
            let ps_graph: Vec<f64> = match ps_graph {
                Some(ps_graph) => serde_json::from_str(&ps_graph).expect("ps_graph should be [p1,p2,p3,...,pm]"),
                None => ps.clone(),
            };
            let pes: Option<String> = matches.value_of_t("pes").ok();
            let pes: Vec<f64> = match pes {
                Some(pes) => serde_json::from_str(&pes).expect("pes should be [pe1,pe2,pe3,...,pem]"),
                None => vec![0.; ps.len()],  // by default no erasure errors
            };
            let pes_graph: Option<String> = matches.value_of_t("pes_graph").ok();
            let pes_graph: Vec<f64> = match pes_graph {
                Some(pes_graph) => serde_json::from_str(&pes_graph).expect("pes_graph should be [pe1,pe2,pe3,...,pem]"),
                None => pes.clone(),
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
            let parallel_init: usize = matches.value_of_t("parallel_init").unwrap_or(parallel);
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
            let use_brief_edge = matches.is_present("use_brief_edge");
            let label: String = matches.value_of_t("label").unwrap_or(format!(""));
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
            return Some(benchmark(&dis, &djs, &nms, &ps, &pes, bias_eta, max_repeats, min_failed_cases, parallel, code_type, decoder, decoder_config
                , ignore_logical_i, ignore_logical_j, debug_print, time_budget, log_runtime_statistics, log_error_pattern_when_logical_error
                , error_model_builder, error_model_configuration, thread_timeout, &ps_graph, &pes_graph, parallel_init, use_brief_edge, label
                , error_model_modifier));
        }
        _ => unreachable!()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Serialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
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
    /// print all error patterns immediately after generating random errors, typically useful to pinpoint how program assertion fail and debug deadlock
    AllErrorPattern,
    /// print failed error patterns that causes logical errors, typically useful to pinpoint how decoder fails to decode a likely error
    FailedErrorPattern,
    /// erasure graph
    ErasureGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct BenchmarkDebugPrintDecoderConfig {
    /// see [`MWPMDecoderConfig`]
    #[serde(alias = "pcmg")]  // abbreviation
    #[serde(default = "mwpm_default_configs::precompute_complete_model_graph")]
    pub precompute_complete_model_graph: bool,
    /// see [`MWPMDecoderConfig`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. error model
    #[serde(alias = "ucp")]  // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Serialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub enum BenchmarkDecoder {
    /// no decoder applied, return empty correction
    None,
    /// minimum-weight perfect matching decoder
    MWPM,
    /// a fast MWPM decoder based on fusion blossom algorithm
    Fusion,
    /// tailored surface code MWPM decoder
    TailoredMWPM,
    /// union-find decoder
    UnionFind,
}

/// progress variable shared between threads to update information
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkThreadDebugger {
    thread_counter: usize,
    error_pattern: Option<SparseErrorPattern>,
    measurement: Option<SparseMeasurement>,
    detected_erasures: Option<SparseDetectedErasures>,
    correction: Option<SparseCorrection>,
}

impl BenchmarkThreadDebugger {
    fn new() -> Self {
        Self {
            thread_counter: 0,
            error_pattern: None,
            measurement: None,
            detected_erasures: None,
            correction: None,
        }
    }
    fn update_thread_counter(&mut self, thread_counter: usize) -> &mut Self {
        self.thread_counter = thread_counter;
        self.error_pattern = None;
        self.measurement = None;
        self.detected_erasures = None;
        self.correction = None;
        self
    }
    /// load error to simulator, useful when debug specific case
    #[allow(dead_code)]
    pub fn load_errors(&self, simulator: &mut Simulator) {
        if self.error_pattern.is_some() {
            simulator.load_sparse_error_pattern(&self.error_pattern.as_ref().unwrap()).expect("success");
        }
        if self.detected_erasures.is_some() {
            simulator.load_sparse_detected_erasures(&self.detected_erasures.as_ref().unwrap()).expect("success");
        }
        // propagate the errors and erasures
        simulator.propagate_errors();
    }
}

fn benchmark(dis: &Vec<usize>, djs: &Vec<usize>, nms: &Vec<usize>, ps: &Vec<f64>, pes: &Vec<f64>, bias_eta: f64, max_repeats: usize, min_failed_cases: usize
        , parallel: usize, code_type: String, decoder: BenchmarkDecoder, decoder_config: serde_json::Value, ignore_logical_i: bool, ignore_logical_j: bool
        , debug_print: Option<BenchmarkDebugPrint>, time_budget: Option<f64>, log_runtime_statistics: Option<String>, log_error_pattern_when_logical_error: bool
        , error_model_builder: Option<ErrorModelBuilder>, error_model_configuration: serde_json::Value, thread_timeout: f64, ps_graph: &Vec<f64>
        , pes_graph: &Vec<f64>, parallel_init: usize, use_brief_edge: bool, label: String, error_model_modifier: Option<serde_json::Value>) -> String {
    // if parallel = 0, use all CPU resources
    let parallel = if parallel == 0 { std::cmp::max(num_cpus::get() - 1, 1) } else { parallel };
    let parallel_init = if parallel_init == 0 { std::cmp::max(num_cpus::get() - 1, 1) } else { parallel_init };
    // create runtime statistics file object if given file path
    let log_runtime_statistics_file = log_runtime_statistics.clone().map(|filename| 
        Arc::new(Mutex::new(File::create(filename.as_str()).expect("cannot create file"))));
    let fixed_configuration = json!({
        "dis": dis,
        "djs": djs,
        "nms": nms,
        "ps": ps,
        "pes": pes,
        "ps_graph": ps_graph,  // used to build decoding graph
        "pes_graph": pes_graph,  // used to build decoding graph
        "bias_eta": bias_eta,
        "max_repeats": max_repeats,
        "min_failed_cases": min_failed_cases,
        "parallel": parallel,
        "parallel_init": parallel_init,
        "code_type": code_type,
        "decoder": decoder,
        "decoder_config": decoder_config,
        "ignore_logical_i": ignore_logical_i,
        "ignore_logical_j": ignore_logical_j,
        "debug_print": debug_print,
        "log_runtime_statistics": log_runtime_statistics,
        "log_error_pattern_when_logical_error": log_error_pattern_when_logical_error,
        "use_brief_edge": use_brief_edge,
        "label": label,
        "error_model_modifier": error_model_modifier,
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
    assert_eq!(ps_graph.len(), ps.len(), "ps_graph and p should be matched");
    assert_eq!(pes_graph.len(), ps.len(), "pes_graph and p should be matched");
    let mut configurations = Vec::new();
    for (di_idx, &di) in dis.iter().enumerate() {
        let noisy_measurements = nms[di_idx];
        let dj = djs[di_idx];
        for (p_idx, p) in ps.iter().enumerate() {
            let p = *p;
            let pe = pes[p_idx];
            let p_graph = ps_graph[p_idx];
            let pe_graph = pes_graph[p_idx];
            assert!(p >= 0. && p <= 1.0, "invalid probability value");
            assert!(p_graph >= 0. && p_graph <= 1.0, "invalid probability value");
            assert!(pe >= 0. && pe <= 1.0, "invalid probability value");
            assert!(pe_graph >= 0. && pe_graph <= 1.0, "invalid probability value");
            configurations.push((di, dj, noisy_measurements, p, pe, p_graph, pe_graph));
        }
    }
    let mut output = format!("");
    if debug_print.is_none() {  // debug print only will not run simulations
        output = format!("format: <p> <di> <nm> <total_repeats> <qec_failed> <error_rate> <dj> <confidence_interval_95_percent> <pe>");
        eprintln!("{}", output);  // compatible with old scripts
    }
    // start running simulations
    for &(di, dj, noisy_measurements, p, pe, p_graph, pe_graph) in configurations.iter() {
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
                    "p_graph": p_graph,
                    "pe_graph": pe_graph,
                }).to_string().as_bytes()).unwrap();
                log_runtime_statistics_file.write(b"\n").unwrap();
                log_runtime_statistics_file.sync_data().unwrap();
            }, _ => { },
        }
        // prepare simulator
        let mut simulator = Simulator::new(CodeType::new(&code_type), BuiltinCodeInformation::new(noisy_measurements, di, dj));
        let mut error_model_graph = ErrorModel::new(&simulator);
        // first use p_graph and pe_graph to build decoder graph, then revert back to real error model
        let px_graph = p_graph / (1. + bias_eta) / 2.;
        let py_graph = px_graph;
        let pz_graph = p_graph - 2. * px_graph;
        simulator.set_error_rates(&mut error_model_graph, px_graph, py_graph, pz_graph, pe_graph);
        // apply customized error model
        if let Some(error_model_builder) = &error_model_builder {
            error_model_builder.apply(&mut simulator, &mut error_model_graph, &error_model_configuration, p_graph, bias_eta, pe_graph);
        }
        // apply error model modifier
        match &error_model_modifier {
            Some(modifier) => {
                match ErrorModelBuilder::apply_error_model_modifier(&mut simulator, &mut error_model_graph, &modifier) {
                    Ok(_) => { },
                    Err(reason) => {
                        panic!("[error] apply error model failed: {}", reason);
                    },
                }
            },
            None => { }
        }
        debug_assert!({  // check correctness only in debug mode because it's expensive
            let sanity_check_result = code_builder_sanity_check(&simulator);
            if let Err(message) = &sanity_check_result {
                println!("[error] code_builder_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        assert!({  // this assertion is cheap, check it in release mode as well
            let sanity_check_result = error_model_sanity_check(&simulator, &error_model_graph);
            if let Err(message) = &sanity_check_result {
                println!("[error] error_model_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        simulator.compress_error_rates(&mut error_model_graph);  // by default compress all error rates
        match debug_print {
            Some(BenchmarkDebugPrint::ErrorModel) => {
                return format!("{}\n", serde_json::to_string(&simulator.to_json(&error_model_graph)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::FullErrorModel) => {
                simulator.expand_error_rates(&mut error_model_graph);  // expand all optional error rates
                return format!("{}\n", serde_json::to_string(&simulator.to_json(&error_model_graph)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::ModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut model_graph = ModelGraph::new(&simulator);
                let error_model_graph = Arc::new(error_model_graph);
                model_graph.build(&mut simulator, error_model_graph, &config.weight_function, parallel_init, config.use_combined_probability, use_brief_edge);
                return format!("{}\n", serde_json::to_string(&model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::CompleteModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut model_graph = ModelGraph::new(&simulator);
                let error_model_graph = Arc::new(error_model_graph);
                model_graph.build(&mut simulator, error_model_graph, &config.weight_function, parallel_init, config.use_combined_probability, use_brief_edge);
                let model_graph = Arc::new(model_graph);
                let mut complete_model_graph = CompleteModelGraph::new(&simulator, Arc::clone(&model_graph));
                complete_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel_init);
                return format!("{}\n", serde_json::to_string(&complete_model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::TailoredModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                tailored_model_graph.build(&mut simulator, &error_model_graph, &config.weight_function);
                return format!("{}\n", serde_json::to_string(&tailored_model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::TailoredCompleteModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                tailored_model_graph.build(&mut simulator, &error_model_graph, &config.weight_function);
                let tailored_model_graph = Arc::new(tailored_model_graph);
                let mut complete_tailored_model_graph = TailoredCompleteModelGraph::new(&simulator, Arc::clone(&tailored_model_graph));
                complete_tailored_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel_init);
                return format!("{}\n", serde_json::to_string(&complete_tailored_model_graph.to_json(&simulator)).expect("serialize should success"));
            },
            Some(BenchmarkDebugPrint::ErasureGraph) => {
                let mut erasure_graph = ErasureGraph::new(&simulator);
                let error_model_graph = Arc::new(error_model_graph);
                erasure_graph.build(&mut simulator, error_model_graph, parallel_init);
                return format!("{}\n", serde_json::to_string(&erasure_graph.to_json(&simulator)).expect("serialize should success"));
            },
            _ => { }
        }
        let debug_print = Arc::new(debug_print);  // share it across threads
        let error_model_graph = Arc::new(error_model_graph);  // change mutability of error model
        // build decoder precomputed data which is shared between threads
        if decoder == BenchmarkDecoder::None {
            assert!(decoder_config.is_object() && decoder_config.as_object().unwrap().len() == 0, "this decoder doesn't support decoder configuration");
        }
        let mwpm_decoder = if decoder == BenchmarkDecoder::MWPM {
            Some(MWPMDecoder::new(&simulator, Arc::clone(&error_model_graph), &decoder_config, parallel_init, use_brief_edge))
        } else { None };
        let fusion_decoder = if decoder == BenchmarkDecoder::Fusion {
            Some(FusionDecoder::new(&simulator, Arc::clone(&error_model_graph), &decoder_config, parallel_init, use_brief_edge))
        } else { None };
        let tailored_mwpm_decoder = if decoder == BenchmarkDecoder::TailoredMWPM {
            Some(TailoredMWPMDecoder::new(&simulator, Arc::clone(&error_model_graph), &decoder_config, parallel_init, use_brief_edge))
        } else { None };
        let union_find_decoder = if decoder == BenchmarkDecoder::UnionFind {
            Some(UnionFindDecoder::new(&simulator, Arc::clone(&error_model_graph), &decoder_config, parallel_init, use_brief_edge))
        } else { None };
        // then prepare the real error model
        let mut error_model = ErrorModel::new(&simulator);
        let px = p / (1. + bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut error_model, px, py, pz, pe);
        // apply customized error model
        if let Some(error_model_builder) = &error_model_builder {
            error_model_builder.apply(&mut simulator, &mut error_model, &error_model_configuration, p, bias_eta, pe);
        }
        // apply error model modifier
        match &error_model_modifier {
            Some(modifier) => {
                match ErrorModelBuilder::apply_error_model_modifier(&mut simulator, &mut error_model, &modifier) {
                    Ok(_) => { },
                    Err(reason) => {
                        panic!("[error] apply error model failed: {}", reason);
                    },
                }
            },
            None => { }
        }
        debug_assert!({  // check correctness only in debug mode because it's expensive
            let sanity_check_result = code_builder_sanity_check(&simulator);
            if let Err(message) = &sanity_check_result {
                println!("[error] code_builder_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        assert!({  // this assertion is cheap, check it in release mode as well
            let sanity_check_result = error_model_sanity_check(&simulator, &error_model);
            if let Err(message) = &sanity_check_result {
                println!("[error] error_model_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        simulator.compress_error_rates(&mut error_model);  // by default compress all error rates
        let error_model = Arc::new(error_model);  // change mutability of error model
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
            let mut fusion_decoder = fusion_decoder.clone();
            let mut tailored_mwpm_decoder = tailored_mwpm_decoder.clone();
            let mut union_find_decoder = union_find_decoder.clone();
            let thread_ended = Arc::new(AtomicBool::new(false));
            threads_ended.push(Arc::clone(&thread_ended));
            let thread_debugger = Arc::new(Mutex::new(BenchmarkThreadDebugger::new()));
            threads_debugger.push(thread_debugger.clone());
            handlers.push(std::thread::spawn(move || {
                for thread_counter in 0..usize::MAX {
                    if thread_timeout >= 0. { thread_debugger.lock().unwrap().update_thread_counter(thread_counter); }
                    // generate random errors and the corresponding measurement
                    let begin = Instant::now();
                    let (error_count, erasure_count) = simulator.generate_random_errors(&error_model);
                    let sparse_detected_erasures = if erasure_count != 0 { simulator.generate_sparse_detected_erasures() } else { SparseDetectedErasures::new() };
                    if thread_timeout >= 0. {
                        let mut thread_debugger = thread_debugger.lock().unwrap();
                        thread_debugger.error_pattern = Some(simulator.generate_sparse_error_pattern());
                        thread_debugger.detected_erasures = Some(sparse_detected_erasures.clone());
                    }  // runtime debug: find deadlock cases
                    if matches!(*debug_print, Some(BenchmarkDebugPrint::AllErrorPattern)) {
                        let sparse_error_pattern = simulator.generate_sparse_error_pattern();
                        eprint!("{}", serde_json::to_string(&sparse_error_pattern).expect("serialize should success"));
                        if sparse_detected_erasures.len() > 0 {  // has detected erasures, report as well
                            eprintln!(", {}", serde_json::to_string(&sparse_detected_erasures).expect("serialize should success"));
                        } else {
                            eprintln!("");
                        }
                    }
                    let sparse_measurement = if error_count != 0 { simulator.generate_sparse_measurement() } else { SparseMeasurement::new() };
                    if thread_timeout >= 0. { thread_debugger.lock().unwrap().measurement = Some(sparse_measurement.clone()); }  // runtime debug: find deadlock cases
                    let prepare_elapsed = begin.elapsed().as_secs_f64();
                    // decode
                    let begin = Instant::now();
                    let (correction, mut runtime_statistics) = match decoder {
                        BenchmarkDecoder::None => {
                            (SparseCorrection::new(), json!({}))
                        },
                        BenchmarkDecoder::MWPM => {
                            mwpm_decoder.as_mut().unwrap().decode_with_erasure(&sparse_measurement, &sparse_detected_erasures)
                        },
                        BenchmarkDecoder::Fusion => {
                            fusion_decoder.as_mut().unwrap().decode_with_erasure(&sparse_measurement, &sparse_detected_erasures)
                        },
                        BenchmarkDecoder::TailoredMWPM => {
                            assert!(sparse_detected_erasures.len() == 0, "tailored MWPM decoder doesn't support erasures");
                            tailored_mwpm_decoder.as_mut().unwrap().decode(&sparse_measurement)
                        },
                        BenchmarkDecoder::UnionFind => {
                            union_find_decoder.as_mut().unwrap().decode_with_erasure(&sparse_measurement, &sparse_detected_erasures)
                        }
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
                    if is_qec_failed && matches!(*debug_print, Some(BenchmarkDebugPrint::FailedErrorPattern)) {
                        let sparse_error_pattern = simulator.generate_sparse_error_pattern();
                        eprint!("{}", serde_json::to_string(&sparse_error_pattern).expect("serialize should success"));
                        if sparse_detected_erasures.len() > 0 {  // has detected erasures, report as well
                            eprintln!(", {}", serde_json::to_string(&sparse_detected_erasures).expect("serialize should success"));
                        } else {
                            eprintln!("");
                        }
                    }
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
        std::thread::sleep(std::time::Duration::from_millis(500));
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
        output += &format!("\n{}", progress_information());
    }
    output
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
