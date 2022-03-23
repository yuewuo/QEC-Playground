mod util;
mod test;
mod tool;
mod types;
mod web;
mod blossom_v;
mod ftqec;
mod offer_decoder;
mod reproducible_rand;
mod offer_mwpm;
mod union_find_decoder;
mod distributed_uf_decoder;
mod fpga_generator;
mod fast_benchmark;
mod simulator;
mod code_builder;
#[macro_use] mod util_macros;

extern crate clap;
#[macro_use] extern crate serde_json;
extern crate ndarray;
extern crate rand;
extern crate actix_web;
extern crate actix_cors;
extern crate serde;
extern crate libc;
extern crate num_cpus;
extern crate petgraph;
extern crate pbr;
extern crate rand_core;
#[macro_use] extern crate derivative;
extern crate union_find;
extern crate derive_more;
extern crate lazy_static;
extern crate either;
extern crate rug;
extern crate shlex;
extern crate cfg_if;
#[cfg(feature="python_interfaces")]
extern crate pyo3;
extern crate platform_dirs;

fn create_clap_parser<'a>(color_choice: clap::ColorChoice) -> clap::Command<'a> {
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
                .arg(clap::Arg::new("pes").long("pes").help("[pe1,pe2,pe3,...,pem] erasure error rate, default to all 0").takes_value(true))
                .arg(clap::Arg::new("bias_eta").long("bias_eta").help("bias_eta = pz / (px + py) and px = py, px + py + pz = p. default to 1/2, which means px = pz = py").takes_value(true).default_value("0.5"))
                .arg(clap::Arg::new("max_repeats").short('m').long("max_repeats").help("maximum total repeats (previously known as `max_N`); 0 for infinity").takes_value(true).default_value("100000000"))
                .arg(clap::Arg::new("min_error_cases").short('e').long("min_error_cases").help("minimum error cases; 0 for infinity").takes_value(true).default_value("10000"))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 means using number of CPUs - 1, by default single thread").takes_value(true).default_value("1"))
                .arg(clap::Arg::new("code_type").short('c').long("code_type").help("code type, see code_builder.rs for more information").possible_values(["StandardPlanarCode", "RotatedPlanarCode", "StandardXZZXCode", "RotatedXZZXCode", "StandardTailoredCode", "RotatedTailoredCode"]).default_value("StandardPlanarCode").takes_value(true))
                .arg(clap::Arg::new("debug_print").long("debug_print").help("only print requested information without running the benchmark").takes_value(true).possible_values(tool::BenchmarkDebugPrint::possible_values()))
            )
            .subcommand(clap::Command::new("fault_tolerant_benchmark").about("benchmark fault tolerant algorithm")
                .arg(clap::Arg::new("Ls").help("[L1,L2,L3,...,Ln] will be code distance of i and j dimension if djs is not provided").takes_value(true).required(true))
                .arg(clap::Arg::new("djs").long("djs").help("[dj1,dj2,dj3,...,djn], will be [L1,L2,L3,...,Ln] if not provided").takes_value(true))
                .arg(clap::Arg::new("Ts").help("[T1,T2,T3,...,Tn], must have exactly the same length as `Ls`").takes_value(true).required(true))
                .arg(clap::Arg::new("ps").help("[p1,p2,p3,...,pm] p = px + py + pz").takes_value(true).required(true))
                .arg(clap::Arg::new("pes").long("pes").help("[pe1,pe2,pe3,...,pem] erasure error rate, default to 0").takes_value(true))
                .arg(clap::Arg::new("max_N").short('m').long("max_N").help("maximum total count, default to 100000000; 0 for infinity").takes_value(true))
                .arg(clap::Arg::new("min_error_cases").short('e').long("min_error_cases").help("minimum error cases, default to 10000; 0 for infinity").takes_value(true))
                .arg(clap::Arg::new("parallel").short('p').long("parallel").help("how many parallel threads to use. 0 will use number of CPUs - 1").takes_value(true))
                .arg(clap::Arg::new("validate_layer").short('v').long("validate_layer").help("validate correction on which layer (all/top/bottom/boundary/<layer>), default to `boundary`").takes_value(true))
                .arg(clap::Arg::new("mini_sync_time").long("mini_sync_time").help("minimum sync time, default to 0.5s").takes_value(true))
                .arg(clap::Arg::new("no_autotune").short('n').long("no_autotune").help("disable autotune, so that all edges are equally weighted"))
                .arg(clap::Arg::new("rotated_planar_code").short('r').long("rotated_planar_code").help("use rotated planar code instead of standard planar code"))
                .arg(clap::Arg::new("ignore_6_neighbors").short('i').long("ignore_6_neighbors").help("ignore 6 neighbors, so that only straight neighbors are kept"))
                .arg(clap::Arg::new("extra_measurement_error").short('x').long("extra_measurement_error").help("the pure measurement error would be p*x, default to 1").takes_value(true))
                .arg(clap::Arg::new("bypass_correction").long("bypass_correction").help("bypass correction procedure to test is logical error rate calculation behaving good; it's also useful when benchmarking the speed of pure simulation"))
                .arg(clap::Arg::new("independent_px_pz").long("independent_px_pz").help("change the error model to (1-px-pz-pxpz)I + px X + pz Z + pxpz Y"))
                .arg(clap::Arg::new("only_count_logical_x").long("only_count_logical_x").help("only count X logical errors but not all logical error. Alert: only available when validate_layer != all"))
                .arg(clap::Arg::new("only_count_logical_z").long("only_count_logical_z").help("only count Z logical errors but not all logical error. Alert: only available when validate_layer != all"))
                .arg(clap::Arg::new("imperfect_initialization").long("imperfect_initialization").help("if imperfect initialization, then there is bottom boundary because errors happen on the bottom"))
                .arg(clap::Arg::new("shallow_error_on_bottom").long("shallow_error_on_bottom").help("add error to data qubit at t=6, so that no measurement error happens at bottom layer. this option can be used with T=0 for single perfect measurement round with only data qubit errors"))
                .arg(clap::Arg::new("no_y_error").long("no_y_error").help("set probability of y errors to 0"))
                .arg(clap::Arg::new("use_xzzx_code").long("use_xzzx_code").help("use XZZX surface code"))
                .arg(clap::Arg::new("use_rotated_tailored_code").long("use_rotated_tailored_code").help("use rotated tailored surface code with high bias to Z errors (X stabilizers and Y stabilizers)"))
                .arg(clap::Arg::new("bias_eta").long("bias_eta").help("bias_eta = pz / (px + py) and px = py, px + py + pz = p. default to 1/2, which is px = pz = py").takes_value(true))
                .arg(clap::Arg::new("decoder").long("decoder").help("supported decoders: MWPM=MinimumWeightPerfectMatching, UF=UnionFind").takes_value(true))
                .arg(clap::Arg::new("max_half_weight").long("max_half_weight").help("[UnionFind decoder only] maximum weight will be 2 * max_half_weight").takes_value(true))
                // 2022.1.25: change default behavior to use combined probability, this will improve threshold and makes more sense
                .arg(clap::Arg::new("disable_combined_probability").long("disable_combined_probability").help("disable combined probability of matching pairs instead of choosing the maximum probability"))
                // 2022.1.25: change default behavior to use ln((1-p)/p) instead of -ln(p), this will improve threshold (although very slightly) but makes more sense
                .arg(clap::Arg::new("disable_autotune_minus_no_error").long("disable_autotune_minus_no_error").help("use ln(1/p) as weight instead of the default ln((1-p)/p)"))
                .arg(clap::Arg::new("error_model").long("error_model").help("possible error models see types::ErrorModel").takes_value(true))
                .arg(clap::Arg::new("error_model_configuration").long("error_model_configuration").help("a json object describing the error model details").takes_value(true))
                .arg(clap::Arg::new("no_stop_if_next_model_is_not_prepared").short('s').long("no_stop_if_next_model_is_not_prepared").help("in rough experiment (e.g. estimate the threshold by testing multiple (di,dj,T) configurations) you can use this option to avoid wasting CPU time, as it will not stop current experiment if the model of next experiment is not prepared. Note that you should keep #threads + 1 <= #CPU because the additional thread is for computing the next model"))
                .arg(clap::Arg::new("log_runtime_statistics").long("log_runtime_statistics").help("log the runtime statistical information, given the path of the log file").takes_value(true))
                .arg(clap::Arg::new("detailed_runtime_statistics").long("detailed_runtime_statistics").help("log the detailed runtime statistics if available, leading to much larger log file"))
                .arg(clap::Arg::new("log_error_pattern_into_statistics_when_has_logical_error").long("log_error_pattern_into_statistics_when_has_logical_error").help("log the error pattern in the log file, which makes the log file much larger"))
                .arg(clap::Arg::new("time_budget").long("time_budget").help("for each configuration, give a maximum time to run (in second)").takes_value(true))
                .arg(clap::Arg::new("use_fast_benchmark").long("use_fast_benchmark").help("use fast benchmark estimation instead of Monte Carlo method"))
                .arg(clap::Arg::new("fbench_disable_additional_error").long("fbench_disable_additional_error").help("do not add additional error when running fast benchmark"))
                .arg(clap::Arg::new("fbench_use_fake_decoder").long("fbench_use_fake_decoder").help("fake decoder succeed only if mini_sync_time"))
                .arg(clap::Arg::new("fbench_use_simple_sum").long("fbench_use_simple_sum").help("by default to false").takes_value(true))
                .arg(clap::Arg::new("fbench_assignment_sampling_amount").long("fbench_assignment_sampling_amount").help("by default to 1").takes_value(true))
                .arg(clap::Arg::new("fbench_weighted_path_sampling").long("fbench_weighted_path_sampling").help("used weighted sampling"))
                .arg(clap::Arg::new("fbench_weighted_assignment_sampling").long("fbench_weighted_assignment_sampling").help("use weighted sampling in error assignment"))
                .arg(clap::Arg::new("fbench_target_dev").long("fbench_target_dev").help("if the deviation of fbench logical error rate is smaller than a number (by default 0 which is never achieved) while keeping for at least 100 rounds, it will exit normally").takes_value(true))
                .arg(clap::Arg::new("rug_precision").long("rug_precision").help("default to 128, the number of bits in a float number used for fast benchmark").takes_value(true))
                .arg(clap::Arg::new("disable_optimize_correction_pattern").long("disable_optimize_correction_pattern").help("disable this optimization"))
                // debugging print utilities
                .arg(clap::Arg::new("debug_print_only").long("debug_print_only").help("only print requested information without running the benchmark"))
                .arg(clap::Arg::new("debug_print_direct_connections").long("debug_print_direct_connections").help("print direct connections, or model graph in our paper https://www.yecl.org/publications/wu2022qec.pdf"))
                .arg(clap::Arg::new("debug_print_exhausted_connections").long("debug_print_exhausted_connections").help("print exhausted connections, or complete model graph in our paper https://www.yecl.org/publications/wu2022qec.pdf"))
                .arg(clap::Arg::new("debug_print_error_model").long("debug_print_error_model").help("print error model, without building the exhausted graph"))
                .arg(clap::Arg::new("debug_print_with_all_possible_error_rates").long("debug_print_with_all_possible_error_rates").help("with all possible positions of error rate, used for externally generating error model"))
                .arg(clap::Arg::new("disable_reduced_graph").long("disable_reduced_graph").help("disable feature: remove edge between two vertices if both of them have smaller weight matching to boundary than matching each other"))  // adding features from Fowler's paper
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let matches = create_clap_parser(clap::ColorChoice::Auto).get_matches();

    match matches.subcommand() {
        Some(("test", matches)) => {
            test::run_matched_test(&matches);
        }
        Some(("tool", matches)) => {
            let output = tool::run_matched_tool(&matches);
            match output {
                Some(to_print) => { print!("{}", to_print); }
                None => { }
            }
        }
        Some(("server", matches)) => {
            let port = matches.value_of("port").unwrap_or("8066").to_string().parse::<i32>().unwrap();
            let addr = matches.value_of("addr").unwrap_or("127.0.0.1").to_string();
            let root_url = matches.value_of("root_url").unwrap_or("/").to_string();
            println!("QECP server booting...");
            println!("visit http://{}:{}{}<commands>", addr, port, root_url);
            println!("supported commands include `hello`, `naive_decoder`, etc. See `web.rs` for more commands");
            web::run_server(port, addr, root_url).await?;
        }
        Some(("fpga_generator", matches)) => {
            fpga_generator::run_matched_fpga_generator(&matches);
        }
        _ => unreachable!()
    }

    Ok(())

}
