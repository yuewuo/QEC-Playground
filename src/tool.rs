#![allow(non_snake_case)]

use crate::cli::*;
use crate::code_builder::*;
use crate::complete_model_graph::*;
#[cfg(feature = "fusion_blossom")]
use crate::decoder_fusion::*;
#[cfg(feature = "hyperion")]
use crate::decoder_hyper_union_find::*;
use crate::decoder_mwpm::*;
use crate::decoder_tailored_mwpm::*;
use crate::decoder_union_find::*;
use crate::erasure_graph::*;
use crate::model_graph::*;
use crate::model_hypergraph::*;
use crate::noise_model::*;
use crate::noise_model_builder::*;
use crate::reproducible_rand::Xoroshiro128StarStar;
use crate::simulator::*;
use crate::simulator_compact::*;
use crate::tailored_complete_model_graph::*;
use crate::tailored_model_graph::*;
use crate::util::local_get_temporary_store;
use crate::visualize::*;
use clap;
use clap::ValueEnum;
use num_cpus;
use pbr::ProgressBar;
#[cfg(feature = "python_binding")]
use pyo3::prelude::*;
use rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

impl ToolCommands {
    pub fn run(self) -> Result<String, String> {
        match self {
            Self::Benchmark(benchmark_parameters) => benchmark_parameters.run(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
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
    #[serde(alias = "pcmg")] // abbreviation
    #[serde(default = "mwpm_default_configs::precompute_complete_model_graph")]
    pub precompute_complete_model_graph: bool,
    /// see [`MWPMDecoderConfig`]
    #[serde(alias = "wf")] // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")] // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct BenchmarkControl {
    pub total_repeats: usize,
    pub qec_failed: usize,
    pub external_termination: bool,
}

impl BenchmarkControl {
    fn new() -> Self {
        Self {
            total_repeats: 0,
            qec_failed: 0,
            external_termination: false,
        }
    }
    fn update_data_should_terminate(
        &mut self,
        is_qec_failed: bool,
        max_repeats: usize,
        min_failed_cases: usize,
    ) -> bool {
        self.total_repeats += 1;
        if is_qec_failed {
            self.qec_failed += 1;
        }
        self.should_terminate(max_repeats, min_failed_cases)
    }
    fn should_terminate(&self, max_repeats: usize, min_failed_cases: usize) -> bool {
        self.external_termination
            || self.total_repeats >= max_repeats
            || self.qec_failed >= min_failed_cases
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
            simulator
                .load_sparse_error_pattern(&self.error_pattern.as_ref().unwrap(), noise_model)
                .expect("success");
        }
        if self.detected_erasures.is_some() {
            simulator
                .load_sparse_detected_erasures(
                    &self.detected_erasures.as_ref().unwrap(),
                    noise_model,
                )
                .expect("success");
        }
        // propagate the errors and erasures
        simulator.propagate_errors();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleSimulationConfig {
    di: usize,
    dj: usize,
    noisy_measurements: usize,
    p: f64,
    pe: f64,
    p_graph: f64,
    pe_graph: f64,
}

impl SingleSimulationConfig {
    pub fn new(
        di: usize,
        dj: usize,
        noisy_measurements: usize,
        p: f64,
        pe: f64,
        p_graph: f64,
        pe_graph: f64,
    ) -> Self {
        Self {
            di,
            dj,
            noisy_measurements,
            p,
            pe,
            p_graph,
            pe_graph,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfigs {
    dis: Vec<usize>,
    djs: Vec<usize>,
    nms: Vec<usize>,
    ps: Vec<f64>,
    pes: Vec<f64>,
    ps_graph: Vec<f64>,
    pes_graph: Vec<f64>,
    max_repeats: usize,
    min_failed_cases: usize,
    parallel: usize,
    parallel_init: usize,
    noise_model_modifier: Option<serde_json::Value>,
    deterministic_seed: Option<u64>,
}

impl SimulationConfigs {
    pub fn new(
        dis: Vec<usize>,
        djs: Vec<usize>,
        nms: Vec<usize>,
        ps: Vec<f64>,
        pes: Vec<f64>,
        ps_graph: Vec<f64>,
        pes_graph: Vec<f64>,
        max_repeats: usize,
        min_failed_cases: usize,
        parallel: usize,
        parallel_init: usize,
        noise_model_modifier: Option<serde_json::Value>,
        deterministic_seed: Option<u64>,
    ) -> Self {
        Self {
            dis,
            djs,
            nms,
            ps,
            pes,
            ps_graph,
            pes_graph,
            max_repeats,
            min_failed_cases,
            parallel,
            parallel_init,
            noise_model_modifier,
            deterministic_seed,
        }
    }
}
impl BenchmarkParameters {
    pub fn run(&self) -> Result<String, String> {
        let configs = self.fill_in_default_parameters()?;
        // create runtime statistics file object if given file path
        let log_runtime_statistics_file = self.log_runtime_statistics.clone().map(|filename| {
            Arc::new(Mutex::new(
                File::create(filename.as_str()).expect("cannot create file"),
            ))
        });
        let simulation_configuration = json!({
            "configs": configs,
            "parameters": self,
        });
        match &log_runtime_statistics_file {
            // append runtime statistics data
            Some(log_runtime_statistics_file) => {
                let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                log_runtime_statistics_file.write_all(b"#f ").unwrap();
                log_runtime_statistics_file
                    .write_all(simulation_configuration.to_string().as_bytes())
                    .unwrap();
                log_runtime_statistics_file.write_all(b"\n").unwrap();
                log_runtime_statistics_file.sync_data().unwrap();
            }
            _ => {}
        }
        // first list all configurations and validate them at the beginning
        let mut output = format!("");
        let titles = format!("format: <p> <di> <nm> <shots> <failed> <pL> <dj> <pL_dev> <pe>");
        eprintln!("{}", titles); // compatible with old scripts
        if self.debug_print.is_none() {
            // debug print only, outputs user specified debug info
            output = titles + "\n";
        }
        if self.enable_visualizer {
            self.assert_single_configuration(&configs)?;
        }
        // start running simulations
        let configurations = self.extract_simulation_configurations(&configs);
        for config in configurations.iter() {
            // append runtime statistics data
            match &log_runtime_statistics_file {
                Some(log_runtime_statistics_file) => {
                    let mut log_runtime_statistics_file =
                        log_runtime_statistics_file.lock().unwrap();
                    log_runtime_statistics_file.write_all(b"# ").unwrap();
                    log_runtime_statistics_file
                        .write_all(json!(config).to_string().as_bytes())
                        .unwrap();
                    log_runtime_statistics_file.write_all(b"\n").unwrap();
                    log_runtime_statistics_file.sync_data().unwrap();
                }
                _ => {}
            }
            output += &(self.run_single(&configs, &config, &log_runtime_statistics_file)? + "\n");
        }
        Ok(output)
    }

    pub fn fill_in_default_parameters(&self) -> Result<SimulationConfigs, String> {
        // prepare default variables
        let dis = self.dis.clone();
        let djs = self.djs.clone().unwrap_or(dis.clone());
        // let djs: Vec<usize> = serde_json::from_str(&self.djs.unwrap_or(self.dis.clone())).expect("djs should be [dj1,dj2,dj3,...,djn]");
        let nms = self.nms.clone();
        assert!(nms.len() == dis.len(), "nms and dis should be paired");
        assert!(dis.len() == djs.len(), "dis and djs should be paired");
        let ps = self.ps.clone();
        let ps_graph = self.ps_graph.clone().unwrap_or(ps.clone());
        let pes = self.pes.clone().unwrap_or(vec![0.; ps.len()]); // by default no erasure errors
        let pes_graph = self.pes_graph.clone().unwrap_or(pes.clone());
        assert_eq!(pes.len(), ps.len(), "pe and p should be matched");
        assert_eq!(ps_graph.len(), ps.len(), "ps_graph and p should be matched");
        assert_eq!(
            pes_graph.len(),
            ps.len(),
            "pes_graph and p should be matched"
        );
        let mut max_repeats: usize = self.max_repeats;
        if max_repeats == 0 {
            max_repeats = usize::MAX;
        }
        let mut min_failed_cases: usize = self.min_failed_cases;
        if min_failed_cases == 0 {
            min_failed_cases = usize::MAX;
        }
        // if parallel = 0, use all CPU resources
        let parallel = if self.parallel == 0 {
            std::cmp::max(num_cpus::get() - 1, 1)
        } else {
            self.parallel
        };
        let parallel_init: usize = self.parallel_init.clone().unwrap_or(self.parallel);
        // noise model modifier, might from `load_noise_model_from_temporary_store` or `load_noise_model_from_file`
        let mut noise_model_modifier_str: Option<String> = None;
        match self.load_noise_model_from_temporary_store {
            Some(noise_model_temporary_id) => {
                match local_get_temporary_store(noise_model_temporary_id) {
                    Some(value) => {
                        noise_model_modifier_str = Some(value);
                    }
                    None => {
                        return Err(format!(
                            "[error] temporary id not found (may expire): {}",
                            noise_model_temporary_id
                        ))
                    }
                }
            }
            None => {}
        }
        match &self.load_noise_model_from_file {
            Some(noise_model_filepath) => match fs::read_to_string(noise_model_filepath.clone()) {
                Ok(value) => {
                    noise_model_modifier_str = Some(value);
                }
                Err(_) => {
                    return Err(format!(
                        "[error] noise model file cannot open: {}",
                        noise_model_filepath
                    ))
                }
            },
            None => {}
        }
        let noise_model_modifier: Option<serde_json::Value> = match noise_model_modifier_str {
            Some(value) => match serde_json::from_str(&value) {
                Ok(noise_model_modifier) => Some(noise_model_modifier),
                Err(_) => {
                    return Err(format!(
                        "[error] noise model cannot recognize, please check file format"
                    ))
                }
            },
            None => None,
        };
        Ok(SimulationConfigs::new(
            dis,
            djs,
            nms,
            ps,
            pes,
            ps_graph,
            pes_graph,
            max_repeats,
            min_failed_cases,
            parallel,
            parallel_init,
            noise_model_modifier,
            self.deterministic_seed.clone(),
        ))
    }

    pub fn assert_single_configuration(&self, configs: &SimulationConfigs) -> Result<(), String> {
        if configs.dis.len() != 1 || configs.ps.len() != 1 {
            return Err("only single configuration is allowed".to_string());
        }
        Ok(())
    }

    pub fn extract_simulation_configurations(
        &self,
        configs: &SimulationConfigs,
    ) -> Vec<SingleSimulationConfig> {
        let mut configurations = Vec::new();
        for (di_idx, &di) in configs.dis.iter().enumerate() {
            let noisy_measurements = configs.nms[di_idx];
            let dj = configs.djs[di_idx];
            for (p_idx, p) in configs.ps.iter().enumerate() {
                let p = *p;
                let pe = configs.pes[p_idx];
                let p_graph = configs.ps_graph[p_idx];
                let pe_graph = configs.pes_graph[p_idx];
                assert!(p >= 0. && p <= 1.0, "invalid probability value");
                assert!(p_graph >= 0. && p_graph <= 1.0, "invalid probability value");
                assert!(pe >= 0. && pe <= 1.0, "invalid probability value");
                assert!(
                    pe_graph >= 0. && pe_graph <= 1.0,
                    "invalid probability value"
                );
                configurations.push(SingleSimulationConfig::new(
                    di,
                    dj,
                    noisy_measurements,
                    p,
                    pe,
                    p_graph,
                    pe_graph,
                ));
            }
        }
        configurations
    }

    pub fn construct_noise_model(
        &self,
        simulator: &mut Simulator,
        configs: &SimulationConfigs,
        config: &SingleSimulationConfig,
        use_p_graph: bool,
    ) -> Result<Arc<NoiseModel>, String> {
        let mut noise_model: NoiseModel = NoiseModel::new(&simulator);
        let p = if use_p_graph {
            config.p_graph
        } else {
            config.p
        };
        let pe = if use_p_graph {
            config.pe_graph
        } else {
            config.pe
        };
        let px = p / (1. + self.bias_eta) / 2.;
        let py = px;
        let pz = p - 2. * px;
        simulator.set_error_rates(&mut noise_model, px, py, pz, pe);
        // apply customized noise model
        if let Some(noise_model_builder) = &self.noise_model_builder {
            noise_model_builder.apply(
                simulator,
                &mut noise_model,
                &self.noise_model_configuration,
                p,
                self.bias_eta,
                pe,
            );
        }
        // apply noise model modifier
        match &configs.noise_model_modifier {
            Some(modifier) => {
                NoiseModelBuilder::apply_noise_model_modifier(
                    simulator,
                    &mut noise_model,
                    &modifier,
                )
                .map_err(|e| format!("apply noise model failed: {e}"))?;
            }
            None => {}
        }
        debug_assert!({
            // check correctness only in debug mode because it's expensive
            let sanity_check_result = code_builder_sanity_check(&simulator);
            if let Err(message) = &sanity_check_result {
                eprintln!("\n[error] code_builder_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        assert!({
            // this assertion is cheap, check it in release mode as well
            let sanity_check_result = noise_model_sanity_check(&simulator, &noise_model);
            if let Err(message) = &sanity_check_result {
                eprintln!("\n[error] noise_model_sanity_check: {}", message)
            }
            sanity_check_result.is_ok()
        });
        simulator.compress_error_rates(&mut noise_model); // by default compress all error rates
        Ok(Arc::new(noise_model))
    }

    /// return Some(info) will indicate termination of simulation: some debug prints are intended to only print something in the beginning
    pub fn execute_debug_print(
        &self,
        configs: &SimulationConfigs,
        simulator: &mut Simulator,
        noise_model: &Arc<NoiseModel>,
    ) -> Result<Option<String>, String> {
        match self.debug_print {
            Some(BenchmarkDebugPrint::NoiseModel) => {
                return Ok(Some(format!(
                    "{}\n",
                    serde_json::to_string(&simulator.to_json(&noise_model)).unwrap()
                )));
            }
            Some(BenchmarkDebugPrint::FullNoiseModel) => {
                let mut noise_model = (**noise_model).clone();
                simulator.expand_error_rates(&mut noise_model); // expand all optional error rates for display purpose
                return Ok(Some(format!(
                    "{}\n",
                    serde_json::to_string(&simulator.to_json(&noise_model)).unwrap()
                )));
            }
            Some(BenchmarkDebugPrint::ModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig =
                    serde_json::from_value(self.decoder_config.clone())
                        .map_err(|x| x.to_string())?;
                let mut model_graph = ModelGraph::new(&simulator);
                model_graph.build(
                    simulator,
                    noise_model.clone(),
                    &config.weight_function,
                    configs.parallel_init,
                    config.use_combined_probability,
                    self.use_brief_edge,
                );
                return Ok(Some(format!(
                    "{}\n",
                    serde_json::to_string(&model_graph.to_json(&simulator)).unwrap()
                )));
            }
            Some(BenchmarkDebugPrint::CompleteModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig =
                    serde_json::from_value(self.decoder_config.clone())
                        .map_err(|x| x.to_string())?;
                let mut model_graph = ModelGraph::new(&simulator);
                model_graph.build(
                    simulator,
                    noise_model.clone(),
                    &config.weight_function,
                    configs.parallel_init,
                    config.use_combined_probability,
                    self.use_brief_edge,
                );
                let model_graph = Arc::new(model_graph);
                let mut complete_model_graph =
                    CompleteModelGraph::new(&simulator, Arc::clone(&model_graph));
                complete_model_graph.precompute(
                    &simulator,
                    config.precompute_complete_model_graph,
                    configs.parallel_init,
                );
                return Ok(Some(format!(
                    "{}\n",
                    serde_json::to_string(&complete_model_graph.to_json(&simulator)).unwrap()
                )));
            }
            Some(BenchmarkDebugPrint::TailoredModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig =
                    serde_json::from_value(self.decoder_config.clone())
                        .map_err(|x| x.to_string())?;
                let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                tailored_model_graph.build(
                    simulator,
                    noise_model,
                    &config.weight_function,
                    config.use_combined_probability,
                );
                return Ok(Some(format!(
                    "{}\n",
                    serde_json::to_string(&tailored_model_graph.to_json(&simulator)).unwrap()
                )));
            }
            Some(BenchmarkDebugPrint::TailoredCompleteModelGraph) => {
                let config: BenchmarkDebugPrintDecoderConfig =
                    serde_json::from_value(self.decoder_config.clone())
                        .map_err(|x| x.to_string())?;
                let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                tailored_model_graph.build(
                    simulator,
                    noise_model,
                    &config.weight_function,
                    config.use_combined_probability,
                );
                let tailored_model_graph = Arc::new(tailored_model_graph);
                let mut complete_tailored_model_graph =
                    TailoredCompleteModelGraph::new(&simulator, Arc::clone(&tailored_model_graph));
                complete_tailored_model_graph.precompute(
                    &simulator,
                    config.precompute_complete_model_graph,
                    configs.parallel_init,
                );
                return Ok(Some(format!(
                    "{}\n",
                    serde_json::to_string(&complete_tailored_model_graph.to_json(&simulator))
                        .unwrap()
                )));
            }
            Some(BenchmarkDebugPrint::ErasureGraph) => {
                let mut erasure_graph = ErasureGraph::new(&simulator);
                erasure_graph.build(simulator, noise_model.clone(), configs.parallel_init);
                return Ok(Some(format!(
                    "{}\n",
                    serde_json::to_string(&erasure_graph.to_json(&simulator)).unwrap()
                )));
            }
            _ => {}
        }
        Ok(None)
    }

    pub fn prepare_visualizer(
        &self,
        simulator: &mut Simulator,
        noise_model: &Arc<NoiseModel>,
        noise_model_graph: &Arc<NoiseModel>,
        configs: &SimulationConfigs,
    ) -> Result<Option<Arc<Mutex<Visualizer>>>, String> {
        let mut visualizer = None;
        if self.enable_visualizer {
            print_visualize_link(self.visualizer_filename.clone());
            let mut new_visualizer: Visualizer = Visualizer::new(Some(
                visualize_data_folder() + self.visualizer_filename.as_str(),
            ))
            .map_err(|x| x.to_string())?;
            new_visualizer
                .add_component(simulator)
                .map_err(|x| x.to_string())?;
            new_visualizer
                .add_component(noise_model.as_ref())
                .map_err(|x| x.to_string())?;
            if self.visualizer_model_graph {
                let config: BenchmarkDebugPrintDecoderConfig =
                    serde_json::from_value(self.decoder_config.clone())
                        .map_err(|x| x.to_string())?;
                let mut model_graph = ModelGraph::new(&simulator);
                model_graph.build(
                    simulator,
                    noise_model_graph.clone(),
                    &config.weight_function,
                    configs.parallel_init,
                    config.use_combined_probability,
                    self.use_brief_edge,
                );
                new_visualizer
                    .add_component(&model_graph)
                    .map_err(|x| x.to_string())?;
            }
            if self.visualizer_model_hypergraph {
                let config: BenchmarkDebugPrintDecoderConfig =
                    serde_json::from_value(self.decoder_config.clone())
                        .map_err(|x| x.to_string())?;
                let mut model_hypergraph = ModelHypergraph::new(&simulator);
                model_hypergraph.build(
                    simulator,
                    noise_model_graph.clone(),
                    &config.weight_function,
                    configs.parallel_init,
                    config.use_combined_probability,
                    self.use_brief_edge,
                );
                new_visualizer
                    .add_component(&model_hypergraph)
                    .map_err(|x| x.to_string())?;
            }
            if self.visualizer_tailored_model_graph {
                let config: BenchmarkDebugPrintDecoderConfig =
                    serde_json::from_value(self.decoder_config.clone())
                        .map_err(|x| x.to_string())?;
                let mut tailored_model_graph = TailoredModelGraph::new(&simulator);
                tailored_model_graph.build(
                    simulator,
                    noise_model_graph.as_ref(),
                    &config.weight_function,
                    config.use_combined_probability,
                );
                new_visualizer
                    .add_component(&tailored_model_graph)
                    .map_err(|x| x.to_string())?;
            }
            new_visualizer.end_component().map_err(|x| x.to_string())?; // make sure the visualization file is valid even user exit the benchmark
            visualizer = Some(Arc::new(Mutex::new(new_visualizer)));
        }
        Ok(visualizer)
    }

    /// run a single simulation; self and configs are general for all simulations, config is specific to a single simulation
    pub fn run_single(
        &self,
        configs: &SimulationConfigs,
        config: &SingleSimulationConfig,
        log_runtime_statistics_file: &Option<Arc<Mutex<File>>>,
    ) -> Result<String, String> {
        // first use p_graph and pe_graph to build decoder graph, then go back to real noise model for simulation; a mismatch between decoding graph and real noise model is realistic
        let mut simulator = Simulator::new(
            self.code_type,
            CodeSize::new(config.noisy_measurements, config.di, config.dj),
        );
        let noise_model_graph =
            self.construct_noise_model(&mut simulator, configs, config, true)?;
        if let Some(terminate_message) =
            self.execute_debug_print(configs, &mut simulator, &noise_model_graph)?
        {
            return Ok(terminate_message); // debug print terminates
        }
        // build decoder instances
        let general_decoder =
            GeneralDecoder::from_parameters(self, configs, config, &simulator, &noise_model_graph)?;
        // prepare fusion blossom exporter
        cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {
            let mut fusion_blossom_syndrome_exporter = None;
            if matches!(self.debug_print, Some(BenchmarkDebugPrint::FusionBlossomSyndromeFile)) {
                if let GeneralDecoder::Fusion(fusion_decoder) = &general_decoder {
                    fusion_blossom_syndrome_exporter = Some(FusionBlossomSyndromeExporter::new(&fusion_decoder, self.fusion_blossom_syndrome_export_filename.clone()));
                } else {
                    return Err("need `fusion` decoder to export".to_string())
                }
            }
            let fusion_blossom_syndrome_exporter = Arc::new(fusion_blossom_syndrome_exporter);
        } }
        // then prepare the real noise model
        let noise_model = self.construct_noise_model(&mut simulator, configs, config, false)?;
        // prepare visualizer
        let visualizer =
            self.prepare_visualizer(&mut simulator, &noise_model, &noise_model_graph, configs)?;
        // prepare result variables for simulation
        let benchmark_control = Arc::new(Mutex::new(BenchmarkControl::new()));
        // setup progress bar
        let mut pb = ProgressBar::on(std::io::stderr(), configs.max_repeats as u64);
        pb.set(0);
        // spawn threads to do simulation
        let mut handlers = Vec::new();
        let mut threads_debugger: Vec<Arc<Mutex<BenchmarkThreadDebugger>>> = Vec::new();
        let mut threads_ended = Vec::new(); // keep updating progress bar until all threads ends
        let general_simulator: GeneralSimulator = if self.use_compact_simulator {
            let first = SimulatorCompact::from_simulator(
                simulator,
                noise_model.clone(),
                configs.parallel_init,
            );
            if let Some(simulator_compact_extender_noisy_measurements) =
                self.simulator_compact_extender_noisy_measurements
            {
                self.assert_single_configuration(&configs)?;
                if simulator_compact_extender_noisy_measurements < config.noisy_measurements {
                    return Err(format!("extender only works for larger noisy_measurement than nms[0], now {simulator_compact_extender_noisy_measurements} < {}", config.noisy_measurements));
                } else {
                    let mut second_simulator = Simulator::new(
                        self.code_type,
                        CodeSize::new(config.noisy_measurements + 1, config.di, config.dj),
                    );
                    let second_noise_model =
                        self.construct_noise_model(&mut second_simulator, configs, config, false)?;
                    let second = SimulatorCompact::from_simulator(
                        second_simulator,
                        second_noise_model,
                        configs.parallel_init,
                    );
                    let extender =
                        SimulatorCompactExtender::new(first, second, config.noisy_measurements);
                    if self.use_compact_simulator_compressed {
                        GeneralSimulator::SimulatorCompactCompressed(
                            SimulatorCompactCompressed::new(
                                extender,
                                simulator_compact_extender_noisy_measurements,
                            ),
                        )
                    } else {
                        let generated =
                            extender.generate(simulator_compact_extender_noisy_measurements);
                        GeneralSimulator::SimulatorCompact(generated)
                    }
                }
            } else {
                GeneralSimulator::SimulatorCompact(first)
            }
        } else {
            GeneralSimulator::Simulator(simulator)
        };
        for parallel_idx in 0..configs.parallel {
            let thread_debugger = Arc::new(Mutex::new(BenchmarkThreadDebugger::new()));
            threads_debugger.push(thread_debugger.clone());
            let thread_ended = Arc::new(AtomicBool::new(false));
            threads_ended.push(Arc::clone(&thread_ended));
            let mut thread_general_simulator = general_simulator.clone();
            if let Some(deterministic_seed) = configs.deterministic_seed {
                let seed: u64 = deterministic_seed + parallel_idx as u64;
                thread_general_simulator.set_rng(Xoroshiro128StarStar::seed_from_u64(seed));
            }
            let mut worker_state = SimulationWorker {
                benchmark_control: benchmark_control.clone(),
                general_simulator: thread_general_simulator,
                noise_model: noise_model.clone(),
                log_runtime_statistics_file: log_runtime_statistics_file.clone(),
                visualizer: visualizer.clone(),
                general_decoder: general_decoder.clone(),
                #[cfg(feature = "fusion_blossom")]
                fusion_blossom_syndrome_exporter: fusion_blossom_syndrome_exporter.clone(),
                thread_debugger,
                thread_ended,
                parameters: self.clone(),
            };
            handlers.push(std::thread::spawn(move || {
                worker_state.run();
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
            let confidence_interval_95_percent = 1.96
                * (error_rate * (1. - error_rate) / (total_repeats as f64)).sqrt()
                / error_rate;
            format!(
                "{} {} {} {} {} {} {} {:.1e} {} ",
                config.p,
                config.di,
                config.noisy_measurements,
                total_repeats,
                qec_failed,
                error_rate,
                config.dj,
                confidence_interval_95_percent,
                config.pe
            )
        };
        loop {
            let time_elapsed = repeat_begin.elapsed().as_secs_f64();
            match self.time_budget {
                Some(time_budget) => {
                    if time_elapsed > time_budget {
                        benchmark_control.lock().unwrap().set_external_terminate();
                    }
                }
                _ => {}
            }
            // compute simulation results
            pb.message(progress_information().as_str());
            {
                // estimate running time cleverer
                let benchmark_control = benchmark_control.lock().unwrap().clone();
                let total_repeats = benchmark_control.total_repeats;
                let qec_failed = benchmark_control.qec_failed;
                let ratio_total_rounds = (total_repeats as f64) / (configs.max_repeats as f64);
                let ratio_qec_failed = (qec_failed as f64) / (configs.min_failed_cases as f64);
                let (mut pb_total, mut set_progress) = if ratio_total_rounds >= ratio_qec_failed {
                    let progress = total_repeats as u64;
                    (
                        if configs.max_repeats as u64 > progress {
                            configs.max_repeats as u64
                        } else {
                            progress
                        },
                        progress,
                    )
                } else {
                    let progress = qec_failed as u64;
                    (
                        if configs.min_failed_cases as u64 > progress {
                            configs.min_failed_cases as u64
                        } else {
                            progress
                        },
                        progress,
                    )
                };
                match self.time_budget {
                    Some(time_budget) => {
                        let ratio_time = time_elapsed / time_budget;
                        if ratio_time >= ratio_total_rounds && ratio_time >= ratio_qec_failed {
                            let progress = total_repeats as u64;
                            pb_total = ((progress as f64) / ratio_time) as u64;
                            set_progress = progress;
                        }
                    }
                    _ => {}
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
            if benchmark_control
                .lock()
                .unwrap()
                .should_terminate(configs.max_repeats, configs.min_failed_cases)
            {
                break;
            }
            // refresh 4 times per second
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        // wait for all threads to terminate until timeout
        let begin = Instant::now();
        std::thread::sleep(std::time::Duration::from_millis(500));
        loop {
            let time_elapsed = begin.elapsed().as_secs_f64();
            if self.thread_timeout >= 0. && time_elapsed >= self.thread_timeout {
                // abnormal break because of timeout
                eprintln!("[error] some threads don't terminate properly within timeout, here are the details:");
                for parallel_idx in (0..configs.parallel).rev() {
                    let thread_ended = threads_ended.swap_remove(parallel_idx);
                    let handler = handlers.swap_remove(parallel_idx);
                    let thread_debugger = threads_debugger.swap_remove(parallel_idx);
                    if !thread_ended.load(Ordering::SeqCst) {
                        eprintln!(
                            "[error] thread {} doesn't terminate within timeout",
                            parallel_idx
                        );
                        eprintln!("{}", json!(thread_debugger.lock().unwrap().clone()));
                    } else {
                        // still join normal threads
                        eprintln!("[info] thread {} normally exit", parallel_idx);
                        handler.join().unwrap();
                    }
                }
                break;
            }
            // check if all threads ended before break the loop
            let mut all_threads_ended = true;
            for thread_ended in threads_ended.iter() {
                if !thread_ended.load(Ordering::SeqCst) {
                    all_threads_ended = false;
                }
            }
            if all_threads_ended {
                // only when all threads ended normally will it joina
                for handler in handlers.drain(..) {
                    handler.join().unwrap();
                }
                break;
            }
            eprintln!(
                "[info] waiting for all threads to end, time elapsed: {:.3}s",
                time_elapsed
            );
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        pb.finish();
        eprintln!("{}", progress_information());
        Ok(format!("{}", progress_information()))
    }
}

/// general class of all supported decoders in QECP
#[derive(Clone)]
pub enum GeneralDecoder {
    None,
    MWPM(MWPMDecoder),
    #[cfg(feature = "fusion_blossom")]
    Fusion(FusionDecoder),
    TailoredMWPM(TailoredMWPMDecoder),
    UnionFind(UnionFindDecoder),
    #[cfg(feature = "hyperion")]
    HyperUnionFind(HyperUnionFindDecoder),
}

impl GeneralDecoder {
    pub fn from_parameters(
        parameters: &BenchmarkParameters,
        configs: &SimulationConfigs,
        config: &SingleSimulationConfig,
        simulator: &Simulator,
        noise_model_graph: &Arc<NoiseModel>,
    ) -> Result<Self, String> {
        Ok(match parameters.decoder {
            BenchmarkDecoder::None => {
                // if parameters.decoder_config.is_object() && parameters.decoder_config.as_object().ok_or("decoder config is not json object")?.len() != 0 {
                //     return Err("`None` decoder doesn't support decoder configuration".to_string());
                // }
                GeneralDecoder::None
            }
            BenchmarkDecoder::MWPM => GeneralDecoder::MWPM(MWPMDecoder::new(
                &simulator,
                noise_model_graph.clone(),
                &parameters.decoder_config,
                configs.parallel_init,
                parameters.use_brief_edge,
            )),
            #[cfg(feature = "fusion_blossom")]
            BenchmarkDecoder::Fusion => {
                let first = FusionDecoder::new(
                    &simulator,
                    noise_model_graph.clone(),
                    &parameters.decoder_config,
                    configs.parallel_init,
                    parameters.use_brief_edge,
                );
                if let Some(simulator_compact_extender_noisy_measurements) =
                    parameters.simulator_compact_extender_noisy_measurements
                {
                    parameters.assert_single_configuration(&configs)?;
                    if simulator_compact_extender_noisy_measurements < config.noisy_measurements {
                        return Err(format!("extender only works for larger noisy_measurement than nms[0], now {simulator_compact_extender_noisy_measurements} < {}", config.noisy_measurements));
                    } else {
                        // use extender to build decoder
                        let mut second_simulator = Simulator::new(
                            parameters.code_type,
                            CodeSize::new(config.noisy_measurements + 1, config.di, config.dj),
                        );
                        let mut second_config = config.clone();
                        second_config.noisy_measurements += 1;
                        let second_noise_model_graph = parameters.construct_noise_model(
                            &mut second_simulator,
                            configs,
                            &second_config,
                            true,
                        )?;
                        let second = FusionDecoder::new(
                            &second_simulator,
                            second_noise_model_graph.clone(),
                            &parameters.decoder_config,
                            configs.parallel_init,
                            parameters.use_brief_edge,
                        );
                        let skip_decoding = first.config.skip_decoding;
                        let extender = FusionBlossomAdaptorExtender::new(
                            Arc::try_unwrap(first.adaptor).unwrap(),
                            Arc::try_unwrap(second.adaptor).unwrap(),
                            config.noisy_measurements,
                        );
                        let generated = extender
                            .generate(simulator_compact_extender_noisy_measurements, skip_decoding);
                        let fusion_solver = if first.config.skip_decoding {
                            fusion_blossom::mwpm_solver::SolverSerial::new(
                                &extender.base.initializer,
                            ) // no need to generate a large solver
                        } else {
                            fusion_blossom::mwpm_solver::SolverSerial::new(&generated.initializer)
                        };
                        GeneralDecoder::Fusion(FusionDecoder {
                            adaptor: Arc::new(generated),
                            fusion_solver,
                            config: first.config,
                        })
                    }
                } else {
                    GeneralDecoder::Fusion(first)
                }
            }
            #[cfg(not(feature = "fusion_blossom"))]
            BenchmarkDecoder::Fusion => {
                return Err(
                    "decoder is not available; try enable feature `fusion_blossom`".to_string(),
                )
            }
            BenchmarkDecoder::TailoredMWPM => {
                GeneralDecoder::TailoredMWPM(TailoredMWPMDecoder::new(
                    &simulator,
                    noise_model_graph.clone(),
                    &parameters.decoder_config,
                    configs.parallel_init,
                    parameters.use_brief_edge,
                ))
            }
            BenchmarkDecoder::UnionFind => GeneralDecoder::UnionFind(UnionFindDecoder::new(
                &simulator,
                noise_model_graph.clone(),
                &parameters.decoder_config,
                configs.parallel_init,
                parameters.use_brief_edge,
            )),
            #[cfg(feature = "hyperion")]
            BenchmarkDecoder::HyperUnionFind => {
                GeneralDecoder::HyperUnionFind(HyperUnionFindDecoder::new(
                    &simulator,
                    noise_model_graph.clone(),
                    &parameters.decoder_config,
                    configs.parallel_init,
                    parameters.use_brief_edge,
                ))
            }
            #[cfg(not(feature = "hyperion"))]
            BenchmarkDecoder::HyperUnionFind => {
                return Err("decoder is not available; try enable feature `hyperion`".to_string())
            }
        })
    }

    pub fn decode_with_erasure(
        &mut self,
        sparse_measurement: &SparseMeasurement,
        sparse_detected_erasures: &SparseErasures,
    ) -> (SparseCorrection, serde_json::Value) {
        match self {
            Self::None => (SparseCorrection::new(), json!({})),
            Self::MWPM(mwpm_decoder) => {
                mwpm_decoder.decode_with_erasure(sparse_measurement, sparse_detected_erasures)
            }
            #[cfg(feature = "fusion_blossom")]
            Self::Fusion(fusion_decoder) => {
                fusion_decoder.decode_with_erasure(sparse_measurement, sparse_detected_erasures)
            }
            Self::TailoredMWPM(tailored_mwpm_decoder) => {
                assert!(
                    sparse_detected_erasures.len() == 0,
                    "tailored MWPM decoder doesn't support erasures"
                );
                tailored_mwpm_decoder.decode(sparse_measurement)
            }
            Self::UnionFind(union_find_decoder) => {
                union_find_decoder.decode_with_erasure(sparse_measurement, sparse_detected_erasures)
            }
            #[cfg(feature = "hyperion")]
            Self::HyperUnionFind(hyper_union_find_decoder) => hyper_union_find_decoder
                .decode_with_erasure(sparse_measurement, sparse_detected_erasures),
        }
    }
}

pub struct SimulationWorker {
    pub benchmark_control: Arc<Mutex<BenchmarkControl>>,
    pub general_simulator: GeneralSimulator,
    pub noise_model: Arc<NoiseModel>,
    pub log_runtime_statistics_file: Option<Arc<Mutex<File>>>,
    pub visualizer: Option<Arc<Mutex<Visualizer>>>,
    pub general_decoder: GeneralDecoder,
    #[cfg(feature = "fusion_blossom")]
    pub fusion_blossom_syndrome_exporter: Arc<Option<FusionBlossomSyndromeExporter>>,
    pub thread_debugger: Arc<Mutex<BenchmarkThreadDebugger>>,
    pub thread_ended: Arc<AtomicBool>,
    pub parameters: BenchmarkParameters,
}

impl SimulationWorker {
    pub fn run(&mut self) {
        for thread_counter in 0..usize::MAX {
            let parameters = &self.parameters;
            if parameters.thread_timeout >= 0. {
                self.thread_debugger
                    .lock()
                    .unwrap()
                    .update_thread_counter(thread_counter);
            }
            // generate random errors and the corresponding measurement
            let begin = Instant::now();
            let (error_count, erasure_count) = self
                .general_simulator
                .generate_random_errors(&self.noise_model);
            let sparse_detected_erasures = if erasure_count != 0 {
                self.general_simulator.generate_sparse_detected_erasures()
            } else {
                SparseErasures::new()
            };
            if parameters.thread_timeout >= 0. {
                let mut thread_debugger = self.thread_debugger.lock().unwrap();
                thread_debugger.error_pattern =
                    Some(self.general_simulator.generate_sparse_error_pattern());
                thread_debugger.detected_erasures = Some(sparse_detected_erasures.clone());
            } // runtime debug: find deadlock cases
            if matches!(
                parameters.debug_print,
                Some(BenchmarkDebugPrint::AllErrorPattern)
            ) {
                let sparse_error_pattern = self.general_simulator.generate_sparse_error_pattern();
                eprint!(
                    "{}",
                    serde_json::to_string(&sparse_error_pattern).expect("serialize should success")
                );
                if sparse_detected_erasures.len() > 0 {
                    // has detected erasures, report as well
                    eprintln!(
                        ", {}",
                        serde_json::to_string(&sparse_detected_erasures)
                            .expect("serialize should success")
                    );
                } else {
                    eprintln!("");
                }
            }
            let sparse_measurement = if error_count != 0 {
                self.general_simulator.generate_sparse_measurement()
            } else {
                SparseMeasurement::new()
            };
            if parameters.thread_timeout >= 0. {
                self.thread_debugger.lock().unwrap().measurement = Some(sparse_measurement.clone());
            } // runtime debug: find deadlock cases
            let simulate_elapsed = begin.elapsed().as_secs_f64();
            cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {
                if let Some(fusion_blossom_syndrome_exporter) = self.fusion_blossom_syndrome_exporter.as_ref() {
                    fusion_blossom_syndrome_exporter.add_syndrome(&sparse_measurement, &sparse_detected_erasures);
                }
            } }
            // decode
            let begin = Instant::now();
            let (correction, mut runtime_statistics) = self
                .general_decoder
                .decode_with_erasure(&sparse_measurement, &sparse_detected_erasures);
            if parameters.thread_timeout >= 0. {
                self.thread_debugger.lock().unwrap().correction = Some(correction.clone());
            } // runtime debug: find deadlock cases
            let decode_elapsed = begin.elapsed().as_secs_f64();
            // validate correction
            let begin = Instant::now();
            let mut is_qec_failed = false;
            let (logical_i, logical_j) = self.general_simulator.validate_correction(&correction);
            if logical_i && !parameters.ignore_logical_i {
                is_qec_failed = true;
            }
            if logical_j && !parameters.ignore_logical_j {
                is_qec_failed = true;
            }
            let validate_elapsed = begin.elapsed().as_secs_f64();
            if is_qec_failed
                && matches!(
                    parameters.debug_print,
                    Some(BenchmarkDebugPrint::FailedErrorPattern)
                )
            {
                let sparse_error_pattern = self.general_simulator.generate_sparse_error_pattern();
                eprint!(
                    "{}",
                    serde_json::to_string(&sparse_error_pattern).expect("serialize should success")
                );
                if sparse_detected_erasures.len() > 0 {
                    // has detected erasures, report as well
                    eprintln!(
                        ", {}",
                        serde_json::to_string(&sparse_detected_erasures)
                            .expect("serialize should success")
                    );
                } else {
                    eprintln!("");
                }
            }
            // update statistic information
            if let Some(log_runtime_statistics_file) = &self.log_runtime_statistics_file {
                runtime_statistics["qec_failed"] = json!(is_qec_failed);
                if parameters.log_error_pattern_when_logical_error && is_qec_failed {
                    runtime_statistics["error_pattern"] =
                        json!(self.general_simulator.generate_sparse_error_pattern());
                }
                runtime_statistics["elapsed"] = json!({
                    "simulate": simulate_elapsed,
                    "decode": decode_elapsed,
                    "validate": validate_elapsed,
                });
                let to_be_written = format!("{}\n", runtime_statistics.to_string());
                let mut log_runtime_statistics_file = log_runtime_statistics_file.lock().unwrap();
                log_runtime_statistics_file
                    .write_all(to_be_written.as_bytes())
                    .unwrap();
            }
            // update visualizer
            if let Some(visualizer) = &self.visualizer {
                if !parameters.visualizer_skip_success_cases || is_qec_failed {
                    let case = json!({
                        "error_pattern": self.general_simulator.generate_sparse_error_pattern(),
                        "measurement": sparse_measurement,
                        "detected_erasures": sparse_detected_erasures,
                        "correction": correction,
                        "qec_failed": is_qec_failed,
                        "elapsed": {
                            "simulate": simulate_elapsed,
                            "decode": decode_elapsed,
                            "validate": validate_elapsed,
                        },
                        "runtime_statistics": runtime_statistics,
                    });
                    let mut visualizer = visualizer.lock().unwrap();
                    visualizer.add_case(case).unwrap();
                }
            }
            // update simulation counters, then break the loop if benchmark should terminate
            if self
                .benchmark_control
                .lock()
                .unwrap()
                .update_data_should_terminate(
                    is_qec_failed,
                    parameters.max_repeats,
                    parameters.min_failed_cases,
                )
            {
                break;
            }
        }
        self.thread_ended.store(true, Ordering::SeqCst);
    }
}
