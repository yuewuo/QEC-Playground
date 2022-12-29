use crate::clap;
use crate::code_builder;
use crate::noise_model_builder;
use crate::tool;


pub fn create_clap_parser<'a>(color_choice: clap::ColorChoice) -> clap::Command<'a> {
    clap::Command::new("QECPlayground")
        .version(env!("CARGO_PKG_VERSION"))
        .author(clap::crate_authors!(", "))
        .about("Quantum Error Correction Playground")
        .color(color_choice)
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("test")
            .about("testing features")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(clap::Command::new("debug_tests").about("test for debug"))
            .subcommand(clap::Command::new("archived_debug_tests").about("archived debug tests"))
            .subcommand(clap::Command::new("all").about("run all tests"))
        )
        .subcommand(clap::Command::new("tool")
            .about("tools")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(clap::Command::new("benchmark").about("benchmark surface code decoders")
                .arg(clap::Arg::new("dis").help("[di1,di2,di3,...,din] code distance of vertical axis").takes_value(true).required(true))
                .arg(clap::Arg::new("djs").long("djs").help("[dj1,dj2,dj3,...,djn] code distance of horizontal axis, will use `dis` if not provided, otherwise must have exactly the same length as `dis`").takes_value(true))
                .arg(clap::Arg::new("nms").help("[nm1,nm2,nm3,...,nmn] number of noisy measurement rounds, must have exactly the same length as `dis`; note that a perfect measurement is always capped at the end, so to simulate a single round of perfect measurement you should set this to 0").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm] p = px + py + pz unless noise model has special interpretation of this value").takes_value(true).required(true))
                .arg(clap::Arg::new("ps_graph").long("ps_graph").help("[p1,p2,p3,...,pm] defaults to ps, used to build the decoding graph").takes_value(true))
                .arg(clap::Arg::new("pes").long("pes").help("[pe1,pe2,pe3,...,pem] erasure error rate, default to all 0").takes_value(true))
                .arg(clap::Arg::new("pes_graph").long("pes_graph").help("[pe1,pe2,pe3,...,pem] defaults to pes, used to build the decoding graph").takes_value(true))
                .arg(clap::Arg::new("bias_eta").long("bias_eta").help("bias_eta = pz / (px + py) and px = py, px + py + pz = p. default to 1/2, which means px = pz = py").takes_value(true).default_value("0.5"))
                .arg(clap::Arg::new("max_repeats").short('m').long("max_repeats").help("maximum total repeats (previously known as `max_N`); 0 for infinity").takes_value(true).default_value("100000000"))
                .arg(clap::Arg::new("min_failed_cases").short('e').long("min_failed_cases").help("minimum failed cases; 0 for infinity").takes_value(true).default_value("10000"))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 means using number of CPUs - 1, by default single thread").takes_value(true).default_value("1"))
                .arg(clap::Arg::new("parallel_init").long("parallel_init").help("how many parallel threads to use when initializing decoders, default to be the same with `parallel`").takes_value(true))
                .arg(clap::Arg::new("code_type").short('c').long("code_type").help("code type, see code_builder.rs for more information").possible_values(code_builder::CodeType::possible_values()).default_value("StandardPlanarCode").takes_value(true))
                .arg(clap::Arg::new("decoder").long("decoder").help("select the benchmarked decoder").takes_value(true).possible_values(tool::BenchmarkDecoder::possible_values()).default_value("mwpm"))
                .arg(clap::Arg::new("decoder_config").long("decoder_config").help("decoder configuration json, panic if any field is not recognized").takes_value(true).default_value("{}"))
                .arg(clap::Arg::new("ignore_logical_i").long("ignore_logical_i").help("ignore the logical error of i axis, e.g. logical Z error in standard CSS surface code"))
                .arg(clap::Arg::new("ignore_logical_j").long("ignore_logical_j").help("ignore the logical error of j axis, e.g. logical X error in standard CSS surface code"))
                .arg(clap::Arg::new("debug_print").long("debug_print").help("only print requested information without running the benchmark").takes_value(true).possible_values(tool::BenchmarkDebugPrint::possible_values()))
                .arg(clap::Arg::new("time_budget").long("time_budget").help("for each configuration, give a maximum time to run (in second)").takes_value(true))
                .arg(clap::Arg::new("log_runtime_statistics").long("log_runtime_statistics").help("log the runtime statistical information, given the path of the statistics log file").takes_value(true))
                .arg(clap::Arg::new("log_error_pattern_when_logical_error").long("log_error_pattern_when_logical_error").help("log the error pattern in the statistics log file, which is useful when debugging rare cases but it can make the log file much larger"))
                .arg(clap::Arg::new("noise_model").long("noise_model").help("possible noise models see noise_model_builder.rs").possible_values(noise_model_builder::NoiseModelBuilder::possible_values()).takes_value(true))
                .arg(clap::Arg::new("noise_model_configuration").long("noise_model_configuration").help("a json object describing the noise model details").takes_value(true).default_value("{}"))
                .arg(clap::Arg::new("thread_timeout").long("thread_timeout").help("wait for some time for threads to end, otherwise print out the unstopped threads and detach them; useful when debugging rare deadlock cases; if set to negative value, no timeout and no thread debug information recording for maximum performance").takes_value(true).default_value("60"))
                .arg(clap::Arg::new("use_brief_edge").long("use_brief_edge").help("use brief edges in model graph to save memories; it will drop the error pattern and correction as long as another one is more probable"))
                .arg(clap::Arg::new("label").long("label").help("arbitrary label information").takes_value(true))
                .arg(clap::Arg::new("load_noise_model_from_temporary_store").long("load_noise_model_from_temporary_store").help("if provided, will fetch a Json from temporary store in web module to update noise model").takes_value(true))
                .arg(clap::Arg::new("load_noise_model_from_file").long("load_noise_model_from_file").help("if provided, will fetch a Json from file to update noise model").takes_value(true))
                .arg(clap::Arg::new("enable_visualizer").long("enable_visualizer").help("logging to the default visualizer file at visualize/data/visualizer.json"))
                .arg(clap::Arg::new("visualizer_filename").long("visualizer_filename").help("visualizer file at visualize/data/<visualizer_filename>.json").takes_value(true))
                .arg(clap::Arg::new("visualizer_skip_success_cases").long("visualizer_skip_success_cases").help("when visualizer is enabled, only record failed cases; useful when trying to debug rare failed cases, e.g. finding the lowest number of physical errors that causes a logical error"))
                .arg(clap::Arg::new("visualizer_model_graph").long("visualizer_model_graph").help("include model graph"))
                .arg(clap::Arg::new("visualizer_model_hypergraph").long("visualizer_model_hypergraph").help("include model hypergraph"))
            )
        )
        .subcommand(clap::Command::new("server").about("HTTP server for decoding information")
            .arg(clap::Arg::new("port").short('p').long("port").help("listening on <addr>:<port>, default to 8066").takes_value(true))
            .arg(clap::Arg::new("addr").short('a').long("addr").help("listening on <addr>:<port>, default to \"127.0.0.1\"").takes_value(true))
            .arg(clap::Arg::new("root_url").short('r').long("root_url").help("root url").takes_value(true))
        )
}
