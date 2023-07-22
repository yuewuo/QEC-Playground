use crate::clap::builder::{StringValueParser, TypedValueParser, ValueParser};
use crate::clap::error::{ContextKind, ContextValue, ErrorKind};
use crate::clap::{Parser, Subcommand};
use crate::code_builder;
use crate::noise_model_builder;
use crate::serde::{Deserialize, Serialize};
use crate::serde_json;
use crate::tool;

#[derive(Parser, Clone)]
#[clap(author = clap::crate_authors!(", "))]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "Quantum Error Correction Playground")]
#[clap(color = clap::ColorChoice::Auto)]
#[clap(propagate_version = true)]
#[clap(subcommand_required = true)]
#[clap(arg_required_else_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// testing features
    Test {
        #[clap(subcommand)]
        command: TestCommands,
    },
    /// built-in tests
    Tool {
        #[clap(subcommand)]
        command: ToolCommands,
    },
    /// HTTP server for decoding information
    Server(ServerParameters),
}

#[derive(Subcommand, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum TestCommands {
    /// test for debug
    DebugTests,
    /// archived debug tests
    ArchivedDebugTests,
    /// run all tests
    All,
}

#[derive(Subcommand, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ToolCommands {
    /// built-in tests
    Benchmark(BenchmarkParameters),
}

#[derive(Clone)]
struct VecUsizeParser;
impl TypedValueParser for VecUsizeParser {
    type Value = Vec<usize>;
    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner = StringValueParser::new();
        let val = inner.parse_ref(cmd, arg, value)?;
        match serde_json::from_str::<Vec<usize>>(&val) {
            Ok(vector) => Ok(vector),
            Err(error) => {
                let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                if let Some(arg) = arg {
                    err.insert(ContextKind::InvalidArg, ContextValue::String(arg.to_string()));
                }
                err.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String(format!("should be like [1,2,3], parse error: {}", error)),
                );
                Err(err)
            }
        }
    }
}

#[derive(Clone)]
struct VecF64Parser;
impl TypedValueParser for VecF64Parser {
    type Value = Vec<f64>;
    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner = StringValueParser::new();
        let val = inner.parse_ref(cmd, arg, value)?;
        match serde_json::from_str::<Vec<f64>>(&val) {
            Ok(vector) => Ok(vector),
            Err(error) => {
                let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                if let Some(arg) = arg {
                    err.insert(ContextKind::InvalidArg, ContextValue::String(arg.to_string()));
                }
                err.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String(format!("should be like [0.1,0.2,0.3], parse error: {error}")),
                );
                Err(err)
            }
        }
    }
}

#[derive(Clone)]
struct SerdeJsonParser;
impl TypedValueParser for SerdeJsonParser {
    type Value = serde_json::Value;
    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner = StringValueParser::new();
        let val = inner.parse_ref(cmd, arg, value)?;
        match serde_json::from_str::<serde_json::Value>(&val) {
            Ok(vector) => Ok(vector),
            Err(error) => {
                let mut err = clap::Error::new(ErrorKind::ValueValidation).with_cmd(cmd);
                if let Some(arg) = arg {
                    err.insert(ContextKind::InvalidArg, ContextValue::String(arg.to_string()));
                }
                err.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String(format!("should be like {{\"a\":1}}, parse error: {error}")),
                );
                Err(err)
            }
        }
    }
}

#[derive(Parser, Clone, Serialize, Deserialize)]
pub struct BenchmarkParameters {
    /// [di1,di2,di3,...,din] code distance of vertical axis
    #[clap(value_parser = ValueParser::new(VecUsizeParser))]
    pub dis: std::vec::Vec<usize>,
    /// [dj1,dj2,dj3,...,djn] code distance of horizontal axis, will use `dis` if not provided, otherwise must have exactly the same length as `dis`
    #[clap(long, value_parser = ValueParser::new(VecUsizeParser))]
    pub djs: Option<std::vec::Vec<usize>>,
    /// [nm1,nm2,nm3,...,nmn] number of noisy measurement rounds, must have exactly the same length as `dis`; note that a perfect measurement is always capped at the end, so to simulate a single round of perfect measurement you should set this to 0
    #[clap(value_parser = ValueParser::new(VecUsizeParser))]
    pub nms: std::vec::Vec<usize>,
    /// [p1,p2,p3,...,pm] p = px + py + pz unless noise model has special interpretation of this value
    #[clap(value_parser = ValueParser::new(VecF64Parser))]
    pub ps: std::vec::Vec<f64>,
    /// [p1,p2,p3,...,pm] defaults to ps, used to build the decoding graph
    #[clap(long, value_parser = ValueParser::new(VecF64Parser))]
    pub ps_graph: Option<std::vec::Vec<f64>>,
    /// [pe1,pe2,pe3,...,pem] erasure error rate, default to all 0
    #[clap(long, value_parser = ValueParser::new(VecF64Parser))]
    pub pes: Option<std::vec::Vec<f64>>,
    /// [pe1,pe2,pe3,...,pem] defaults to pes, used to build the decoding graph
    #[clap(long, value_parser = ValueParser::new(VecF64Parser))]
    pub pes_graph: Option<std::vec::Vec<f64>>,
    /// bias_eta = pz / (px + py) and px = py, px + py + pz = p. default to 1/2, which means px = pz = py
    #[clap(long, default_value_t = 0.5)]
    pub bias_eta: f64,
    /// maximum total repeats (previously known as `max_N`); 0 for infinity
    #[clap(short = 'm', long, default_value_t = 100000000)]
    pub max_repeats: usize,
    /// minimum failed cases; 0 for infinity
    #[clap(short = 'e', long, default_value_t = 10000)]
    pub min_failed_cases: usize,
    /// how many parallel threads to use. 0 means using number of CPUs - 1, by default single thread
    #[clap(short = 'p', long, default_value_t = 1)]
    pub parallel: usize,
    /// how many parallel threads to use when initializing decoders, default to be the same with `parallel`
    #[clap(long)]
    pub parallel_init: Option<usize>,
    /// code type, see code_builder.rs for more information
    #[clap(short = 'c', long, value_enum, default_value_t = code_builder::CodeType::StandardPlanarCode)]
    pub code_type: code_builder::CodeType,
    /// select the benchmarked decoder
    #[clap(long, value_enum, default_value_t = tool::BenchmarkDecoder::MWPM)]
    pub decoder: tool::BenchmarkDecoder,
    /// decoder configuration json, panic if any field is not recognized
    #[clap(long, default_value_t = json!({}), value_parser = ValueParser::new(SerdeJsonParser))]
    pub decoder_config: serde_json::Value,
    /// ignore the logical error of i axis, e.g. logical Z error in standard CSS surface code
    #[clap(long, action)]
    pub ignore_logical_i: bool,
    /// ignore the logical error of j axis, e.g. logical X error in standard CSS surface code
    #[clap(long, action)]
    pub ignore_logical_j: bool,
    /// only print requested information without running the benchmark
    #[clap(long)]
    pub debug_print: Option<tool::BenchmarkDebugPrint>,
    /// for each configuration, give a maximum time to run (in second)
    #[clap(long)]
    pub time_budget: Option<f64>,
    /// log the runtime statistical information, given the path of the statistics log file
    #[clap(long)]
    pub log_runtime_statistics: Option<String>,
    /// log the error pattern in the statistics log file, which is useful when debugging rare cases but it can make the log file much larger
    #[clap(long, action)]
    pub log_error_pattern_when_logical_error: bool,
    /// possible noise models see noise_model_builder.rs
    #[clap(long, alias = "noise-model")]
    pub noise_model_builder: Option<noise_model_builder::NoiseModelBuilder>,
    /// a json object describing the noise model details
    #[clap(long, default_value_t = json!({}), value_parser = ValueParser::new(SerdeJsonParser))]
    pub noise_model_configuration: serde_json::Value,
    /// wait for some time for threads to end, otherwise print out the unstopped threads and detach them; useful when debugging rare deadlock cases; if set to negative value, no timeout and no thread debug information recording for maximum performance
    #[clap(long, default_value_t = 60.)]
    pub thread_timeout: f64,
    /// use brief edges in model graph to save memories; it will drop the error pattern and correction as long as another one is more probable
    #[clap(long, action)]
    pub use_brief_edge: bool,
    /// arbitrary label information
    #[clap(long, default_value_t = ("").to_string())]
    pub label: String,
    /// if provided, will fetch a Json from temporary store in web module to update noise model
    #[clap(long)]
    pub load_noise_model_from_temporary_store: Option<usize>,
    /// if provided, will fetch a Json from file to update noise model
    #[clap(long)]
    pub load_noise_model_from_file: Option<String>,
    /// logging to the default visualizer file at visualize/data/visualizer.json
    #[clap(long, action)]
    pub enable_visualizer: bool,
    /// visualizer file at visualize/data/<visualizer_filename.json>
    #[clap(long, default_value_t = crate::visualize::static_visualize_data_filename())]
    pub visualizer_filename: String,
    /// when visualizer is enabled, only record failed cases; useful when trying to debug rare failed cases, e.g. finding the lowest number of physical errors that causes a logical error
    #[clap(long, action)]
    pub visualizer_skip_success_cases: bool,
    /// include model graph in the visualizer file
    #[clap(long, action)]
    pub visualizer_model_graph: bool,
    /// include model hypergraph in the visualizer file
    #[clap(long, action)]
    pub visualizer_model_hypergraph: bool,
    /// include the three tailored mwpm model graph in the visualizer file
    #[clap(long, action)]
    pub visualizer_tailored_model_graph: bool,
    /// fusion blossom syndrome export configuration
    #[clap(long, default_value_t = ("./tmp/fusion.syndromes").to_string())]
    pub fusion_blossom_syndrome_export_filename: String,
    /// when provided, it will override the default nms[0] value and generate a compact simulator using `SimulatorCompactExtender`;
    /// note that not all decoders can adapt to this, because they still use the original simulator to construct their decoding structure.
    /// the only supported decoder is `fusion`.
    #[clap(long, requires = "use_compact_simulator")]
    pub simulator_compact_extender_noisy_measurements: Option<usize>,
    /// use compact simulator to generate syndromes instead
    #[clap(long, action)]
    pub use_compact_simulator: bool,
    /// use compressed compact simulator, further reducing the memory requirement;
    /// note that this optimizes memory but sacrifices speed, since all the error sources are generated dynamically on the fly
    #[clap(long, requires = "use_compact_simulator")]
    pub use_compact_simulator_compressed: bool,
    /// use deterministic seed for debugging purpose
    #[clap(long)]
    pub deterministic_seed: Option<u64>,
}

#[derive(Parser, Clone)]
pub struct ServerParameters {
    /// listening on <addr>:<port>, default to 8066
    #[clap(short = 'p', long, default_value_t = 8066)]
    pub port: i32,
    /// listening on <addr>:<port>, default to "127.0.0.1"
    #[clap(short = 'a', long, default_value_t = ("127.0.0.1").to_string())]
    pub addr: String,
    /// root url
    #[clap(short = 'r', long, default_value_t = ("/").to_string())]
    pub root_url: String,
}
