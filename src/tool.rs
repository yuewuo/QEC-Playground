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
use super::clap::ValueEnum;
use std::sync::atomic::{AtomicBool, Ordering};
use super::noise_model::*;
use serde::{Serialize, Deserialize};
use super::decoder_mwpm::*;
#[cfg(feature="fusion_blossom")]
use super::decoder_fusion::*;
use super::model_graph::*;
use super::complete_model_graph::*;
use super::decoder_tailored_mwpm::*;
use super::tailored_model_graph::*;
use super::tailored_complete_model_graph::*;
use super::noise_model_builder::*;
use super::decoder_union_find::*;
use super::erasure_graph::*;
use super::visualize::*;
use super::model_hypergraph::*;
#[cfg(feature="hyperion")]
use super::decoder_hyper_union_find::*;
use crate::cli::*;
use crate::simulator_compact::*;


impl ToolCommands {
    pub fn run(self) -> Option<String> {
        match self {
            Self::Benchmark(benchmark_parameters) => {
                Some(benchmark_parameters.run())
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub enum BenchmarkDebugPrint {
    /// the original noise model
    NoiseModel,
    /// including every possible error rate (correlated ones), but initialize them as 0
    FullNoiseModel,
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
    /// syndrome file for fusion-blossom library to use, output to `output_filename`
    FusionBlossomSyndromeFile,
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
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")]  // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize)]
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
    /// hypergraph union-find decoder
    HyperUnionFind,
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
    detected_erasures: Option<SparseErasures>,
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
    pub fn load_errors(&self, simulator: &mut Simulator, noise_model: &NoiseModel) {
        if self.error_pattern.is_some() {
            simulator.load_sparse_error_pattern(&self.error_pattern.as_ref().unwrap(), noise_model).expect("success");
        }
        if self.detected_erasures.is_some() {
            simulator.load_sparse_detected_erasures(&self.detected_erasures.as_ref().unwrap(), noise_model).expect("success");
        }
        // propagate the errors and erasures
        simulator.propagate_errors();
    }
}

impl BenchmarkParameters {
    pub fn run(self) -> String {
        // prepare default variables
        let dis: Vec<usize> = serde_json::from_str(&self.dis).expect("dis should be [di1,di2,di3,...,din]");
        let djs: Vec<usize> = serde_json::from_str(&self.djs.unwrap_or(self.dis.clone())).expect("djs should be [dj1,dj2,dj3,...,djn]");
        let nms: Vec<usize> = serde_json::from_str(&self.nms).expect("nms should be [nm1,nm2,nm3,...,nmn]");
        assert!(nms.len() == dis.len(), "nms and dis should be paired");
        assert!(dis.len() == djs.len(), "dis and djs should be paired");
        let ps: Vec<f64> = serde_json::from_str(&self.ps).expect("ps should be [p1,p2,p3,...,pm]");
        let ps_graph: Vec<f64> = self.ps_graph.map(|ps_graph| serde_json::from_str(&ps_graph).expect("ps_graph should be [p1,p2,p3,...,pm]")).unwrap_or(ps.clone());
        let pes: Vec<f64> = self.pes.map(|pes| serde_json::from_str(&pes).expect("pes should be [pe1,pe2,pe3,...,pem]")).unwrap_or(vec![0.; ps.len()]);  // by default no erasure errors
        let pes_graph: Vec<f64> = self.pes_graph.map(|pes_graph| serde_json::from_str(&pes_graph).expect("pes_graph should be [pe1,pe2,pe3,...,pem]")).unwrap_or(pes.clone());
        let fusion_blossom_syndrome_export_config: serde_json::Value = serde_json::from_str(&self.fusion_blossom_syndrome_export_config).expect("json object");
        assert_eq!(pes.len(), ps.len(), "pe and p should be paired");
        let mut max_repeats: usize = self.max_repeats;
        if max_repeats == 0 {
            max_repeats = usize::MAX;
        }
        let mut min_failed_cases: usize = self.min_failed_cases;
        if min_failed_cases == 0 {
            min_failed_cases = usize::MAX;
        }
        let decoder_config: serde_json::Value = serde_json::from_str(&self.decoder_config).unwrap();
        let noise_model_builder = self.noise_model.clone();
        let noise_model_configuration: serde_json::Value = serde_json::from_str(&self.noise_model_configuration).unwrap();
        let mut noise_model_modifier_str: Option<String> = None;
        match self.load_noise_model_from_temporary_store {
            Some(noise_model_temporary_id) => {
                match local_get_temporary_store(noise_model_temporary_id) {
                    Some(value) => { noise_model_modifier_str = Some(value); },
                    None => { return format!("[error] temporary id not found (may expire): {}", noise_model_temporary_id) }
                }
            },
            None => { },
        }
        match self.load_noise_model_from_file {
            Some(noise_model_filepath) => {
                match fs::read_to_string(noise_model_filepath.clone()) {
                    Ok(value) => { noise_model_modifier_str = Some(value); },
                    Err(_) => { return format!("[error] noise model file cannot open: {}", noise_model_filepath) }
                }
            },
            None => { },
        }
        let noise_model_modifier: Option<serde_json::Value> = match noise_model_modifier_str {
            Some(value) => match serde_json::from_str(&value) {
                Ok(noise_model_modifier) => Some(noise_model_modifier),
                Err(_) => { return format!("[error] noise model cannot recognize, please check file format") }
            },
            None => None,
        };
        let visualizer_filename = self.visualizer_filename.clone().unwrap_or(static_visualize_data_filename());
        let use_brief_edge = self.use_brief_edge;
        let decoder = self.decoder;
        let bias_eta = self.bias_eta;
        let debug_print = self.debug_print;
        let thread_timeout = self.thread_timeout;
        // if parallel = 0, use all CPU resources
        let parallel = if self.parallel == 0 { std::cmp::max(num_cpus::get() - 1, 1) } else { self.parallel };
        let parallel_init: usize = self.parallel_init.clone().unwrap_or(self.parallel);
        // create runtime statistics file object if given file path
        let log_runtime_statistics_file = self.log_runtime_statistics.clone().map(|filename| 
            Arc::new(Mutex::new(File::create(filename.as_str()).expect("cannot create file"))));
        let fixed_configuration = json!({
            "dis": dis,
            "djs": djs,
            "nms": nms,
            "ps": ps,
            "pes": pes,
            "ps_graph": ps_graph,  // used to build decoding graph
            "pes_graph": pes_graph,  // used to build decoding graph
            "bias_eta": self.bias_eta,
            "max_repeats": max_repeats,
            "min_failed_cases": min_failed_cases,
            "parallel": parallel,
            "parallel_init": parallel_init,
            "code_type": self.code_type,
            "decoder": decoder,
            "decoder_config": decoder_config,
            "ignore_logical_i": self.ignore_logical_i,
            "ignore_logical_j": self.ignore_logical_j,
            "debug_print": self.debug_print,
            "log_runtime_statistics": self.log_runtime_statistics.clone(),
            "log_error_pattern_when_logical_error": self.log_error_pattern_when_logical_error,
            "use_brief_edge": use_brief_edge,
            "label": self.label.clone(),
            "noise_model_modifier": noise_model_modifier,
            "fusion_blossom_syndrome_export_config": fusion_blossom_syndrome_export_config,
        });
        match &log_runtime_statistics_file {  // append runtime statistics data
            Some(log_runtime_statistics_file) => {
                let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                log_runtime_statistics_file.write_all(b"#f ").unwrap();
                log_runtime_statistics_file.write_all(fixed_configuration.to_string().as_bytes()).unwrap();
                log_runtime_statistics_file.write_all(b"\n").unwrap();
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
        if self.debug_print.is_none() {  // debug print only will not run simulations
            output = format!("format: <p> <di> <nm> <total_repeats> <qec_failed> <error_rate> <dj> <confidence_interval_95_percent> <pe>");
            eprintln!("{}", output);  // compatible with old scripts
        }
        if self.enable_visualizer {
            assert_eq!(configurations.len(), 1, "visualizer can only record a single configuration");
        }
        // start running simulations
        for &(di, dj, noisy_measurements, p, pe, p_graph, pe_graph) in configurations.iter() {
            // append runtime statistics data
            match &log_runtime_statistics_file {
                Some(log_runtime_statistics_file) => {
                    let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                    log_runtime_statistics_file.write_all(b"# ").unwrap();
                    log_runtime_statistics_file.write_all(json!({
                        "di": di,
                        "dj": dj,
                        "noisy_measurements": noisy_measurements,
                        "p": p,
                        "pe": pe,
                        "p_graph": p_graph,
                        "pe_graph": pe_graph,
                    }).to_string().as_bytes()).unwrap();
                    log_runtime_statistics_file.write_all(b"\n").unwrap();
                    log_runtime_statistics_file.sync_data().unwrap();
                }, _ => { },
            }
            // prepare simulator
            let mut simulator = Simulator::new(self.code_type, CodeSize::new(noisy_measurements, di, dj));
            let mut noise_model_graph = NoiseModel::new(&simulator);
            // first use p_graph and pe_graph to build decoder graph, then revert back to real noise model
            let px_graph = p_graph / (1. + self.bias_eta) / 2.;
            let py_graph = px_graph;
            let pz_graph = p_graph - 2. * px_graph;
            simulator.set_error_rates(&mut noise_model_graph, px_graph, py_graph, pz_graph, pe_graph);
            // apply customized noise model
            if let Some(noise_model_builder) = &noise_model_builder {
                noise_model_builder.apply(&mut simulator, &mut noise_model_graph, &noise_model_configuration, p_graph, bias_eta, pe_graph);
            }
            // apply noise model modifier
            match &noise_model_modifier {
                Some(modifier) => {
                    match NoiseModelBuilder::apply_noise_model_modifier(&mut simulator, &mut noise_model_graph, &modifier) {
                        Ok(_) => { },
                        Err(reason) => {
                            panic!("[error] apply noise model failed: {}", reason);
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
                let sanity_check_result = noise_model_sanity_check(&simulator, &noise_model_graph);
                if let Err(message) = &sanity_check_result {
                    println!("[error] noise_model_sanity_check: {}", message)
                }
                sanity_check_result.is_ok()
            });
            simulator.compress_error_rates(&mut noise_model_graph);  // by default compress all error rates
            cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {
                let mut fusion_blossom_syndrome_exporter = None;
            } }
            match debug_print {
                Some(BenchmarkDebugPrint::NoiseModel) => {
                    return format!("{}\n", serde_json::to_string(&simulator.to_json(&noise_model_graph)).expect("serialize should success"));
                },
                Some(BenchmarkDebugPrint::FullNoiseModel) => {
                    simulator.expand_error_rates(&mut noise_model_graph);  // expand all optional error rates
                    return format!("{}\n", serde_json::to_string(&simulator.to_json(&noise_model_graph)).expect("serialize should success"));
                },
                Some(BenchmarkDebugPrint::ModelGraph) => {
                    let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                    let mut model_graph = ModelGraph::new(&simulator);
                    let noise_model_graph = Arc::new(noise_model_graph);
                    model_graph.build(&mut simulator, noise_model_graph, &config.weight_function, parallel_init, config.use_combined_probability, use_brief_edge);
                    return format!("{}\n", serde_json::to_string(&model_graph.to_json(&simulator)).expect("serialize should success"));
                },
                Some(BenchmarkDebugPrint::CompleteModelGraph) => {
                    let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                    let mut model_graph = ModelGraph::new(&simulator);
                    let noise_model_graph = Arc::new(noise_model_graph);
                    model_graph.build(&mut simulator, noise_model_graph, &config.weight_function, parallel_init, config.use_combined_probability, use_brief_edge);
                    let model_graph = Arc::new(model_graph);
                    let mut complete_model_graph = CompleteModelGraph::new(&simulator, Arc::clone(&model_graph));
                    complete_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel_init);
                    return format!("{}\n", serde_json::to_string(&complete_model_graph.to_json(&simulator)).expect("serialize should success"));
                },
                Some(BenchmarkDebugPrint::TailoredModelGraph) => {
                    let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                    let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                    tailored_model_graph.build(&mut simulator, &noise_model_graph, &config.weight_function);
                    return format!("{}\n", serde_json::to_string(&tailored_model_graph.to_json(&simulator)).expect("serialize should success"));
                },
                Some(BenchmarkDebugPrint::TailoredCompleteModelGraph) => {
                    let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                    let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                    tailored_model_graph.build(&mut simulator, &noise_model_graph, &config.weight_function);
                    let tailored_model_graph = Arc::new(tailored_model_graph);
                    let mut complete_tailored_model_graph = TailoredCompleteModelGraph::new(&simulator, Arc::clone(&tailored_model_graph));
                    complete_tailored_model_graph.precompute(&simulator, config.precompute_complete_model_graph, parallel_init);
                    return format!("{}\n", serde_json::to_string(&complete_tailored_model_graph.to_json(&simulator)).expect("serialize should success"));
                },
                Some(BenchmarkDebugPrint::ErasureGraph) => {
                    let mut erasure_graph = ErasureGraph::new(&simulator);
                    let noise_model_graph = Arc::new(noise_model_graph);
                    erasure_graph.build(&mut simulator, noise_model_graph, parallel_init);
                    return format!("{}\n", serde_json::to_string(&erasure_graph.to_json(&simulator)).expect("serialize should success"));
                },
                Some(BenchmarkDebugPrint::FusionBlossomSyndromeFile) => {
                    cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {
                        fusion_blossom_syndrome_exporter = Some(crate::util::FusionBlossomSyndromeExporter::new(&fusion_blossom_syndrome_export_config, &mut simulator, Arc::new(noise_model_graph.clone()), parallel_init, use_brief_edge));
                    } else { panic!("fusion_blossom feature required") } }
                },
                _ => { }
            }
            #[cfg(feature="fusion_blossom")]
            let fusion_blossom_syndrome_exporter = Arc::new(fusion_blossom_syndrome_exporter);
            let debug_print = Arc::new(debug_print);  // share it across threads
            let noise_model_graph = Arc::new(noise_model_graph);  // change mutability of noise model
            // build decoder precomputed data which is shared between threads
            if decoder == BenchmarkDecoder::None {
                assert!(decoder_config.is_object() && decoder_config.as_object().unwrap().len() == 0, "this decoder doesn't support decoder configuration");
            }
            let mwpm_decoder = if decoder == BenchmarkDecoder::MWPM {
                Some(MWPMDecoder::new(&simulator, Arc::clone(&noise_model_graph), &decoder_config, parallel_init, use_brief_edge))
            } else { None };
            cfg_if::cfg_if! {
                if #[cfg(feature="fusion_blossom")] {
                    let fusion_decoder = if decoder == BenchmarkDecoder::Fusion {
                        Some(FusionDecoder::new(&simulator, Arc::clone(&noise_model_graph), &decoder_config, parallel_init, use_brief_edge))
                    } else { None };
                } else {
                    if decoder == BenchmarkDecoder::Fusion {
                        panic!("fusion blossom is not available; try enable the feature `fusion_blossom`")
                    }
                }
            }
            let tailored_mwpm_decoder = if decoder == BenchmarkDecoder::TailoredMWPM {
                Some(TailoredMWPMDecoder::new(&simulator, Arc::clone(&noise_model_graph), &decoder_config, parallel_init, use_brief_edge))
            } else { None };
            let union_find_decoder = if decoder == BenchmarkDecoder::UnionFind {
                Some(UnionFindDecoder::new(&simulator, Arc::clone(&noise_model_graph), &decoder_config, parallel_init, use_brief_edge))
            } else { None };
            cfg_if::cfg_if! {
                if #[cfg(feature="hyperion")] {
                    let hyper_union_find_decoder = if decoder == BenchmarkDecoder::HyperUnionFind {
                        Some(HyperUnionFindDecoder::new(&simulator, Arc::clone(&noise_model_graph), &decoder_config, parallel_init, use_brief_edge))
                    } else { None };
                } else {
                    if decoder == BenchmarkDecoder::HyperUnionFind {
                        panic!("hypergraph union-find decoder is not available; try enable the feature `hyperion`")
                    }
                }
            }
            // then prepare the real noise model
            let mut noise_model = NoiseModel::new(&simulator);
            let px = p / (1. + bias_eta) / 2.;
            let py = px;
            let pz = p - 2. * px;
            simulator.set_error_rates(&mut noise_model, px, py, pz, pe);
            // apply customized noise model
            if let Some(noise_model_builder) = &noise_model_builder {
                noise_model_builder.apply(&mut simulator, &mut noise_model, &noise_model_configuration, p, bias_eta, pe);
            }
            // apply noise model modifier
            match &noise_model_modifier {
                Some(modifier) => {
                    match NoiseModelBuilder::apply_noise_model_modifier(&mut simulator, &mut noise_model, &modifier) {
                        Ok(_) => { },
                        Err(reason) => {
                            panic!("[error] apply noise model failed: {}", reason);
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
                let sanity_check_result = noise_model_sanity_check(&simulator, &noise_model);
                if let Err(message) = &sanity_check_result {
                    println!("[error] noise_model_sanity_check: {}", message)
                }
                sanity_check_result.is_ok()
            });
            simulator.compress_error_rates(&mut noise_model);  // by default compress all error rates
            let noise_model = Arc::new(noise_model);  // change mutability of noise model
            let mut visualizer = None;
            if self.enable_visualizer {
                print_visualize_link(visualizer_filename.clone());
                let mut new_visualizer = Visualizer::new(Some(visualize_data_folder() + visualizer_filename.as_str())).unwrap();
                new_visualizer.add_component(&simulator).unwrap();
                new_visualizer.add_component(noise_model.as_ref()).unwrap();
                if self.visualizer_model_graph {
                    let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                    let mut model_graph = ModelGraph::new(&simulator);
                    model_graph.build(&mut simulator, Arc::clone(&noise_model_graph), &config.weight_function, parallel_init
                        , config.use_combined_probability, use_brief_edge);
                    new_visualizer.add_component(&model_graph).unwrap();
                }
                if self.visualizer_model_hypergraph {
                    let config: BenchmarkDebugPrintDecoderConfig = serde_json::from_value(decoder_config.clone()).unwrap();
                    let mut model_hypergraph = ModelHypergraph::new(&simulator);
                    model_hypergraph.build(&mut simulator, Arc::clone(&noise_model_graph), &config.weight_function, parallel_init
                        , config.use_combined_probability, use_brief_edge);
                    new_visualizer.add_component(&model_hypergraph).unwrap();
                }
                new_visualizer.end_component().unwrap();  // make sure the visualization file is valid even user exit the benchmark
                visualizer = Some(Arc::new(Mutex::new(new_visualizer)));
            }
            // prepare result variables for simulation
            let benchmark_control = Arc::new(Mutex::new(BenchmarkControl::new()));
            // setup progress bar
            let mut pb = ProgressBar::on(std::io::stderr(), max_repeats as u64);
            pb.set(0);
            // spawn threads to do simulation
            let mut handlers = Vec::new();
            let mut threads_debugger: Vec<Arc<Mutex<BenchmarkThreadDebugger>>> = Vec::new();
            let mut threads_ended = Vec::new();  // keep updating progress bar until all threads ends
            let general_simulator: GeneralSimulator = if self.use_compact_simulator {
                GeneralSimulator::SimulatorCompact(SimulatorCompact::from_simulator(simulator, noise_model_graph.clone(), parallel_init))
            } else {
                GeneralSimulator::Simulator(simulator)
            };
            for _parallel_idx in 0..parallel {
                let benchmark_control = Arc::clone(&benchmark_control);
                let mut general_simulator = general_simulator.clone();
                let noise_model = Arc::clone(&noise_model);
                let debug_print = Arc::clone(&debug_print);
                let log_runtime_statistics_file = log_runtime_statistics_file.clone();
                let visualizer = visualizer.clone();
                let mut mwpm_decoder = mwpm_decoder.clone();
                cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {
                    let fusion_blossom_syndrome_exporter = fusion_blossom_syndrome_exporter.clone();
                    let mut fusion_decoder = fusion_decoder.clone();
                } }
                let mut tailored_mwpm_decoder = tailored_mwpm_decoder.clone();
                let mut union_find_decoder = union_find_decoder.clone();
                cfg_if::cfg_if! { if #[cfg(feature="hyperion")] {
                    let mut hyper_union_find_decoder = hyper_union_find_decoder.clone();
                } }
                let thread_ended = Arc::new(AtomicBool::new(false));
                threads_ended.push(Arc::clone(&thread_ended));
                let thread_debugger = Arc::new(Mutex::new(BenchmarkThreadDebugger::new()));
                threads_debugger.push(thread_debugger.clone());
                handlers.push(std::thread::spawn(move || {
                    for thread_counter in 0..usize::MAX {
                        if thread_timeout >= 0. { thread_debugger.lock().unwrap().update_thread_counter(thread_counter); }
                        // generate random errors and the corresponding measurement
                        let begin = Instant::now();
                        let (error_count, erasure_count) = general_simulator.generate_random_errors(&noise_model);
                        let sparse_detected_erasures = if erasure_count != 0 { general_simulator.generate_sparse_detected_erasures() } else { SparseErasures::new() };
                        if thread_timeout >= 0. {
                            let mut thread_debugger = thread_debugger.lock().unwrap();
                            thread_debugger.error_pattern = Some(general_simulator.generate_sparse_error_pattern());
                            thread_debugger.detected_erasures = Some(sparse_detected_erasures.clone());
                        }  // runtime debug: find deadlock cases
                        if matches!(*debug_print, Some(BenchmarkDebugPrint::AllErrorPattern)) {
                            let sparse_error_pattern = general_simulator.generate_sparse_error_pattern();
                            eprint!("{}", serde_json::to_string(&sparse_error_pattern).expect("serialize should success"));
                            if sparse_detected_erasures.len() > 0 {  // has detected erasures, report as well
                                eprintln!(", {}", serde_json::to_string(&sparse_detected_erasures).expect("serialize should success"));
                            } else {
                                eprintln!("");
                            }
                        }
                        let sparse_measurement = if error_count != 0 { general_simulator.generate_sparse_measurement() } else { SparseMeasurement::new() };
                        if thread_timeout >= 0. { thread_debugger.lock().unwrap().measurement = Some(sparse_measurement.clone()); }  // runtime debug: find deadlock cases
                        let simulate_elapsed = begin.elapsed().as_secs_f64();
                        cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {
                            if let Some(fusion_blossom_syndrome_exporter) = fusion_blossom_syndrome_exporter.as_ref() {
                                fusion_blossom_syndrome_exporter.add_syndrome(&sparse_measurement, &sparse_detected_erasures);
                            }
                        } }
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
                                cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {
                                    fusion_decoder.as_mut().unwrap().decode_with_erasure(&sparse_measurement, &sparse_detected_erasures)
                                } else { unreachable!() } }
                            },
                            BenchmarkDecoder::TailoredMWPM => {
                                assert!(sparse_detected_erasures.len() == 0, "tailored MWPM decoder doesn't support erasures");
                                tailored_mwpm_decoder.as_mut().unwrap().decode(&sparse_measurement)
                            },
                            BenchmarkDecoder::UnionFind => {
                                union_find_decoder.as_mut().unwrap().decode_with_erasure(&sparse_measurement, &sparse_detected_erasures)
                            }
                            BenchmarkDecoder::HyperUnionFind => {
                                cfg_if::cfg_if! { if #[cfg(feature="hyperion")] {
                                    hyper_union_find_decoder.as_mut().unwrap().decode_with_erasure(&sparse_measurement, &sparse_detected_erasures)
                                } else { unreachable!() } }
                            }
                        };
                        if thread_timeout >= 0. { thread_debugger.lock().unwrap().correction = Some(correction.clone()); }  // runtime debug: find deadlock cases
                        let decode_elapsed = begin.elapsed().as_secs_f64();
                        // validate correction
                        let begin = Instant::now();
                        let mut is_qec_failed = false;
                        let (logical_i, logical_j) = general_simulator.validate_correction(&correction);
                        if logical_i && !self.ignore_logical_i {
                            is_qec_failed = true;
                        }
                        if logical_j && !self.ignore_logical_j {
                            is_qec_failed = true;
                        }
                        let validate_elapsed = begin.elapsed().as_secs_f64();
                        if is_qec_failed && matches!(*debug_print, Some(BenchmarkDebugPrint::FailedErrorPattern)) {
                            let sparse_error_pattern = general_simulator.generate_sparse_error_pattern();
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
                            if self.log_error_pattern_when_logical_error && is_qec_failed {
                                runtime_statistics["error_pattern"] = json!(general_simulator.generate_sparse_error_pattern());
                            }
                            runtime_statistics["elapsed"] = json!({
                                "simulate": simulate_elapsed,
                                "decode": decode_elapsed,
                                "validate": validate_elapsed,
                            });
                            let to_be_written = format!("{}\n", runtime_statistics.to_string());
                            let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                            log_runtime_statistics_file.write_all(to_be_written.as_bytes()).unwrap();
                        }
                        // update visualizer
                        if let Some(visualizer) = &visualizer {
                            if !self.visualizer_skip_success_cases || is_qec_failed {
                                let case = json!({
                                    "error_pattern": general_simulator.generate_sparse_error_pattern(),
                                    "measurement": sparse_measurement,
                                    "detected_erasures": sparse_detected_erasures,
                                    "correction": correction,
                                    "qec_failed": is_qec_failed,
                                    "elapsed": {
                                        "simulate": simulate_elapsed,
                                        "decode": decode_elapsed,
                                        "validate": validate_elapsed,
                                    },
                                });
                                let mut visualizer = visualizer.lock().unwrap();
                                visualizer.add_case(case).unwrap();
                            }
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
                match self.time_budget {
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
                    match self.time_budget {
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

}