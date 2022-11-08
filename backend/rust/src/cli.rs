use crate::clap;
use crate::code_builder;
use crate::error_model_builder;
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
            .subcommand(clap::Command::new("try_blossom_correction").about("try to use blossom library to decoder"))
            .subcommand(clap::Command::new("maximum_max_weight_matching_correction").about("try to use networkx python library to decoder"))
            .subcommand(clap::Command::new("debug_tests").about("test for debug"))
            .subcommand(clap::Command::new("archived_debug_tests").about("archived debug tests"))
            .subcommand(clap::Command::new("all").about("run all tests"))
            .subcommand(clap::Command::new("offer_decoder_study").about("find some error cases in which offer decoder fails but MWPM decoder succeeds")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
                .arg(clap::Arg::new("p").help("error rate").takes_value(true).required(true))
                .arg(clap::Arg::new("count").short('c').long("count").help("how many cases to find").takes_value(true))
                .arg(clap::Arg::new("max_resend").short('r').long("max_resend").help("maximum rounds to resend offer, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("max_cycles").short('m').long("max_cycles").help("maximum cycles to run, corresponding to clock cycle in real hardware, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("print_error_pattern_to_find_infinite_loop").long("print_error_pattern_to_find_infinite_loop").help("print all error patterns"))
            )
            .subcommand(clap::Command::new("offer_algorithm_study").about("find some error cases in which offer algorithm fails but MWPM algorithm succeeds")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
                .arg(clap::Arg::new("p").help("error rate").takes_value(true).required(true))
                .arg(clap::Arg::new("count").short('c').long("count").help("how many cases to find").takes_value(true))
                .arg(clap::Arg::new("max_resend").short('r').long("max_resend").help("maximum rounds to resend offer, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("max_cycles").short('m').long("max_cycles").help("maximum cycles to run, corresponding to clock cycle in real hardware, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("print_error_pattern_to_find_infinite_loop").long("print_error_pattern_to_find_infinite_loop").help("print all error patterns"))
            )
            .subcommand(clap::Command::new("union_find_decoder_study").about("find some error cases in which union find decoder fails but MWPM algorithm succeeds")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
                .arg(clap::Arg::new("p").help("error rate").takes_value(true).required(true))
                .arg(clap::Arg::new("count").short('c').long("count").help("how many cases to find").takes_value(true))
                .arg(clap::Arg::new("max_cost").short('m').long("max_cost").help("maximum cost").takes_value(true))
            )
            .subcommand(clap::Command::new("union_find_decoder_xzzx_code_study").about("find some error cases in which union find decoder fails but MWPM algorithm succeeds")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
                .arg(clap::Arg::new("p").help("error rate").takes_value(true).required(true))
                .arg(clap::Arg::new("count").short('c').long("count").help("how many cases to find").takes_value(true))
                .arg(clap::Arg::new("max_half_weight").long("max_half_weight").help("maximum weight will be 2 * max_half_weight").takes_value(true))
                .arg(clap::Arg::new("bias_eta").long("bias_eta").help("bias_eta = pz / (px + py) and px = py, px + py + pz = p. default to 1/2, which is px = pz = py").takes_value(true))
            )
            .subcommand(clap::Command::new("distributed_union_find_decoder_study").about("find some error cases in which distributed union find decoder fails but MWPM algorithm succeeds")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
                .arg(clap::Arg::new("p").help("error rate").takes_value(true).required(true))
                .arg(clap::Arg::new("count").short('c').long("count").help("how many cases to find").takes_value(true))
            )
            .subcommand(clap::Command::new("code_capacity_tailored_decoder_study").about("find some error cases in which tailored decoder fails to decode but it should in fact")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
                .arg(clap::Arg::new("p").help("error rate").takes_value(true).required(true))
                .arg(clap::Arg::new("a").help("noise bias eta").takes_value(true).required(true))
                .arg(clap::Arg::new("count").short('c').long("count").help("how many cases to find").takes_value(true))
                .arg(clap::Arg::new("print_error_pattern_to_find_infinite_loop").long("print_error_pattern_to_find_infinite_loop").help("print all error patterns"))
            )
        )
        .subcommand(clap::Command::new("tool")
            .about("tools")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(clap::Command::new("benchmark").about("benchmark surface code decoders")
                .arg(clap::Arg::new("dis").help("[di1,di2,di3,...,din] code distance of vertical axis").takes_value(true).required(true))
                .arg(clap::Arg::new("djs").long("djs").help("[dj1,dj2,dj3,...,djn] code distance of horizontal axis, will use `dis` if not provided, otherwise must have exactly the same length as `dis`").takes_value(true))
                .arg(clap::Arg::new("nms").help("[nm1,nm2,nm3,...,nmn] number of noisy measurement rounds, must have exactly the same length as `dis`; note that a perfect measurement is always capped at the end, so to simulate a single round of perfect measurement you should set this to 0").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm] p = px + py + pz unless error model has special interpretation of this value").takes_value(true).required(true))
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
                .arg(clap::Arg::new("error_model").long("error_model").help("possible error models see error_model_builder.rs").possible_values(error_model_builder::ErrorModelBuilder::possible_values()).takes_value(true))
                .arg(clap::Arg::new("error_model_configuration").long("error_model_configuration").help("a json object describing the error model details").takes_value(true).default_value("{}"))
                .arg(clap::Arg::new("thread_timeout").long("thread_timeout").help("wait for some time for threads to end, otherwise print out the unstopped threads and detach them; useful when debugging rare deadlock cases; if set to negative value, no timeout and no thread debug information recording for maximum performance").takes_value(true).default_value("60"))
                .arg(clap::Arg::new("use_brief_edge").long("use_brief_edge").help("use brief edges in model graph to save memories; it will drop the error pattern and correction as long as another one is more probable"))
                .arg(clap::Arg::new("label").long("label").help("arbitrary label information").takes_value(true))
                .arg(clap::Arg::new("load_error_model_from_temporary_store").long("load_error_model_from_temporary_store").help("if provided, will fetch a Json from temporary store in web module to update error model").takes_value(true))
                .arg(clap::Arg::new("load_error_model_from_file").long("load_error_model_from_file").help("if provided, will fetch a Json from file to update error model").takes_value(true))
            )
            .subcommand(clap::Command::new("decoder_comparison_benchmark").about("benchmark fault tolerant algorithm")
                .arg(clap::Arg::new("Ls").help("[L1,L2,L3,...,Ln]").takes_value(true).required(true))
                .arg(clap::Arg::new("Ts").help("[T1,T2,T3,...,Tn], must have exactly the same length as `Ls`").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm]").takes_value(true).required(true))
                .arg(clap::Arg::new("max_N").short('m').long("max_N").help("maximum total count, default to 100000000").takes_value(true))
                .arg(clap::Arg::new("min_error_cases").short('e').long("min_error_cases").help("minimum error cases, default to 10000").takes_value(true))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 will use number of CPUs - 1").takes_value(true))
                .arg(clap::Arg::new("validate_layer").short('v').long("validate_layer").help("validate correction on which layer (all/top/bottom/boundary/<layer>), default to `boundary`").takes_value(true))
                .arg(clap::Arg::new("mini_batch").short('b').long("mini_batch").help("mini batch, default to 1000").takes_value(true))
                .arg(clap::Arg::new("autotune").short('a').long("autotune").help("whether enable autotune, default to true").takes_value(true))
                .arg(clap::Arg::new("rotated_planar_code").short('r').long("rotated_planar_code").help("whether use rotated planar code, default to false").takes_value(true))
                .arg(clap::Arg::new("ignore_6_neighbors").short('i').long("ignore_6_neighbors").help("whether ignore 6 neighbors, so that only straight neighbors are kept, default to false").takes_value(true))
                .arg(clap::Arg::new("extra_measurement_error").short('x').long("extra_measurement_error").help("the pure measurement error would be p*x, default to 1").takes_value(true))
                .arg(clap::Arg::new("bypass_correction").long("bypass_correction").help("bypass correction procedure to test is logical error rate calculation behaving good"))
                .arg(clap::Arg::new("independent_px_pz").long("independent_px_pz").help("change the error model to (1-px-pz-pxpz)I + px X + pz Z + pxpz Y"))
                .arg(clap::Arg::new("only_count_logical_x").long("only_count_logical_x").help("only count X logical errors but not all logical error. Alert: only available when validate_layer != all"))
                .arg(clap::Arg::new("imperfect_initialization").long("imperfect_initialization").help("if imperfect initialization, then there is bottom boundary because errors happen on the bottom"))
                .arg(clap::Arg::new("substreams").short('s').long("substreams").help("Number of substreams for substream comparison algorithm, default to 32").takes_value(true))
            )
            .subcommand(clap::Command::new("offer_decoder_standard_planar_benchmark").about("benchmark offer decoder algorithm with standard planar code")
                .arg(clap::Arg::new("Ls").help("[L1,L2,L3,...,Ln]").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm]").takes_value(true).required(true))
                .arg(clap::Arg::new("max_N").short('m').long("max_N").help("maximum total count, default to 100000000").takes_value(true))
                .arg(clap::Arg::new("min_error_cases").short('e').long("min_error_cases").help("minimum error cases, default to 10000").takes_value(true))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 will use number of CPUs - 1").takes_value(true))
                .arg(clap::Arg::new("mini_batch").short('b').long("mini_batch").help("mini batch, default to 1000").takes_value(true))
                .arg(clap::Arg::new("only_count_logical_x").long("only_count_logical_x").help("only count X logical errors but not all logical error."))
                .arg(clap::Arg::new("max_resend").short('r').long("max_resend").help("maximum rounds to resend offer, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("max_cycles").short('c').long("max_cycles").help("maximum cycles to run, corresponding to clock cycle in real hardware, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("disable_probabilistic_accept").long("disable_probabilistic_accept").help("disable probabilistic accept, this will cause dead lock and degrade performance of d>5"))
                .arg(clap::Arg::new("repeat_experiment_each_error").long("repeat_experiment_each_error").help("repeat experiment for each error pattern, default to 1").takes_value(true))
            )
            .subcommand(clap::Command::new("offer_algorithm_standard_planar_benchmark").about("benchmark offer decoder algorithm with standard planar code")
                .arg(clap::Arg::new("Ls").help("[L1,L2,L3,...,Ln]").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm]").takes_value(true).required(true))
                .arg(clap::Arg::new("max_N").short('m').long("max_N").help("maximum total count, default to 100000000").takes_value(true))
                .arg(clap::Arg::new("min_error_cases").short('e').long("min_error_cases").help("minimum error cases, default to 10000").takes_value(true))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 will use number of CPUs - 1").takes_value(true))
                .arg(clap::Arg::new("mini_batch").short('b').long("mini_batch").help("mini batch, default to 1000").takes_value(true))
                .arg(clap::Arg::new("only_count_logical_x").long("only_count_logical_x").help("only count X logical errors but not all logical error."))
                .arg(clap::Arg::new("max_resend").short('r').long("max_resend").help("maximum rounds to resend offer, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("max_cycles").short('c').long("max_cycles").help("maximum cycles to run, corresponding to clock cycle in real hardware, default to usize::MAX").takes_value(true))
                .arg(clap::Arg::new("disable_probabilistic_accept").long("disable_probabilistic_accept").help("disable probabilistic accept, this will cause dead lock and degrade performance of d>5"))
                .arg(clap::Arg::new("repeat_experiment_each_error").long("repeat_experiment_each_error").help("repeat experiment for each error pattern, default to 1").takes_value(true))
            )
            .subcommand(clap::Command::new("union_find_decoder_standard_planar_benchmark").about("benchmark union find decoder with standard planar code")
                .arg(clap::Arg::new("Ls").help("[L1,L2,L3,...,Ln]").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm]").takes_value(true).required(true))
                .arg(clap::Arg::new("max_N").short('m').long("max_N").help("maximum total count, default to 100000000").takes_value(true))
                .arg(clap::Arg::new("min_error_cases").short('e').long("min_error_cases").help("minimum error cases, default to 10000").takes_value(true))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 will use number of CPUs - 1").takes_value(true))
                .arg(clap::Arg::new("mini_batch").short('b').long("mini_batch").help("mini batch, default to 1000").takes_value(true))
                .arg(clap::Arg::new("only_count_logical_x").long("only_count_logical_x").help("only count X logical errors but not all logical error."))
                .arg(clap::Arg::new("no_y_error").long("no_y_error").help("set probability of y errors to 0"))
                .arg(clap::Arg::new("towards_mwpm").long("towards_mwpm").help("use advanced methods toward MWPM decoder"))
                .arg(clap::Arg::new("max_half_weight").long("max_half_weight").help("maximum weight will be 2 * max_half_weight").takes_value(true))
                .arg(clap::Arg::new("bias_eta").long("bias_eta").help("bias_eta = pz / (px + py) and px = py, px + py + pz = p. default to 1/2, which is px = pz = py").takes_value(true))
            )
            .subcommand(clap::Command::new("distributed_union_find_decoder_standard_planar_benchmark").about("benchmark distributed union find decoder with standard planar code")
                .arg(clap::Arg::new("Ls").help("[L1,L2,L3,...,Ln]").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm]").takes_value(true).required(true))
                .arg(clap::Arg::new("max_N").short('m').long("max_N").help("maximum total count, default to 100000000").takes_value(true))
                .arg(clap::Arg::new("min_error_cases").short('e').long("min_error_cases").help("minimum error cases, default to 10000").takes_value(true))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 will use number of CPUs - 1").takes_value(true))
                .arg(clap::Arg::new("mini_batch").short('b').long("mini_batch").help("mini batch, default to 1000").takes_value(true))
                .arg(clap::Arg::new("only_count_logical_x").long("only_count_logical_x").help("only count X logical errors but not all logical error."))
                .arg(clap::Arg::new("output_cycle_distribution").long("output_cycle_distribution").help("output cycle distribution to a json file"))
                .arg(clap::Arg::new("fast_channel_interval").long("fast_channel_interval").help("add fast channels at distance (fast_channel_interval ^ k), default to 0 (no fast channel)").takes_value(true))
                .arg(clap::Arg::new("no_y_error").long("no_y_error").help("set probability of y errors to 0"))
            )
        )
        .subcommand(clap::Command::new("fpga_generator")
            .about("fpga_generator")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(clap::Command::new("perfect_measurement_distributed_union_find").about("DUF decoder under perfect measurement condition")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
            )
            .subcommand(clap::Command::new("fault_tolerant_distributed_union_find").about("DUF decoder under imperfect measurement condition")
                .arg(clap::Arg::new("d").help("code distance").takes_value(true).required(true))
                .arg(clap::Arg::new("measurement_rounds").help("measurement rounds").takes_value(true).required(true))
                .arg(clap::Arg::new("p").help("physical error rate").takes_value(true))
                .arg(clap::Arg::new("autotune").short('a').long("autotune").help("if set, enable topological code autotune structure"))
                .arg(clap::Arg::new("fast_channel_interval").short('f').long("fast_channel_interval").help("fast channel interval, default to 1").takes_value(true))
            )
        )
        .subcommand(clap::Command::new("server").about("HTTP server for decoding information")
            .arg(clap::Arg::new("port").short('p').long("port").help("listening on <addr>:<port>, default to 8066").takes_value(true))
            .arg(clap::Arg::new("addr").short('a').long("addr").help("listening on <addr>:<port>, default to \"127.0.0.1\"").takes_value(true))
            .arg(clap::Arg::new("root_url").short('r').long("root_url").help("root url").takes_value(true))
        )
}
