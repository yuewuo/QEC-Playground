#![allow(non_snake_case)]

use super::clap;
use super::util;
use super::rand::prelude::*;
use super::serde_json;
use std::path::Path;
use super::types::*;
use super::ndarray::{Axis};
use super::qec;
use super::pyo3::prelude::*;
use super::pyo3::types::{IntoPyDict};
use std::io::BufRead;
use super::blossom_v;
use super::num_cpus;
use std::sync::{Arc, Mutex};
use super::ftqec;
use super::pbr::ProgressBar;
use super::types::QubitType;

pub fn run_matched_tool(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("generate_random_errors", Some(matches)) => {
            let Ls = value_t!(matches, "Ls", String).expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps = value_t!(matches, "ps", String).expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let N = value_t!(matches, "N", usize).expect("N should be integer");
            let directory = value_t!(matches, "directory", String).unwrap_or("./".to_string());
            generate_random_errors(&Ls, &ps, N, &directory);
        }
        ("decoder_benchmark", Some(matches)) => {
            let Ls = value_t!(matches, "Ls", String).expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps = value_t!(matches, "ps", String).expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let directory = value_t!(matches, "directory", String).unwrap_or("./".to_string());
            let qec_decoder = value_t!(matches, "qec_decoder", String).unwrap_or("naive_decoder".to_string());
            decoder_benchmark(&Ls, &ps, &directory, &qec_decoder);
        }
        ("automatic_benchmark", Some(matches)) => {
            let Ls = value_t!(matches, "Ls", String).expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps = value_t!(matches, "ps", String).expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N = value_t!(matches, "max_N", usize).unwrap_or(100000000);  // default to 1e8
            let min_error_cases = value_t!(matches, "min_error_cases", usize).unwrap_or(1000);  // default to 1e3
            let qec_decoder = value_t!(matches, "qec_decoder", String).unwrap_or("naive_decoder".to_string());
            automatic_benchmark(&Ls, &ps, max_N, min_error_cases, &qec_decoder);
        }
        ("error_rate_MWPM_with_weight", Some(matches)) => {
            let Ls = value_t!(matches, "Ls", String).expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let ps = value_t!(matches, "ps", String).expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N = value_t!(matches, "max_N", usize).unwrap_or(100000000);  // default to 1e8
            let min_error_cases = value_t!(matches, "min_error_cases", usize).unwrap_or(1000);  // default to 1e3
            let weights = value_t!(matches, "weights", String).unwrap_or("default_weights".to_string());
            let parallel = value_t!(matches, "parallel", usize).unwrap_or(1);  // default to 1
            error_rate_MWPM_with_weight(&Ls, &ps, max_N, min_error_cases, &weights, parallel);
        }
        ("fault_tolerant_benchmark", Some(matches)) => {
            let Ls = value_t!(matches, "Ls", String).expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let Ts = value_t!(matches, "Ts", String).expect("required");
            let Ts: Vec<usize> = serde_json::from_str(&Ts).expect("Ts should be [T1,T2,T3,...,Tn]");
            assert!(Ts.len() == Ls.len(), "Ts and Ls should be paired");
            let ps = value_t!(matches, "ps", String).expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N = value_t!(matches, "max_N", usize).unwrap_or(100000000);  // default to 1e8
            let min_error_cases = value_t!(matches, "min_error_cases", usize).unwrap_or(10000);  // default to 1e3
            let parallel = value_t!(matches, "parallel", usize).unwrap_or(1);  // default to 1
            let validate_layer = value_t!(matches, "validate_layer", String).unwrap_or("boundary".to_string());
            let mini_batch = value_t!(matches, "mini_batch", usize).unwrap_or(1);  // default to 1
            let autotune = ! matches.is_present("no_autotune");  // default autotune is enabled
            let rotated_planar_code = matches.is_present("rotated_planar_code");  // default use standard planar code
            let ignore_6_neighbors = matches.is_present("ignore_6_neighbors");  // default use 12 neighbors version
            let extra_measurement_error = value_t!(matches, "extra_measurement_error", f64).unwrap_or(1.);  // default to 1.
            let bypass_correction = matches.is_present("bypass_correction");
            let independent_px_pz = matches.is_present("independent_px_pz");
            let only_count_logical_x = matches.is_present("only_count_logical_x");
            let imperfect_initialization = matches.is_present("imperfect_initialization");
            let shallow_error_on_bottom = matches.is_present("shallow_error_on_bottom");
            fault_tolerant_benchmark(&Ls, &Ts, &ps, max_N, min_error_cases, parallel, validate_layer, mini_batch, autotune, rotated_planar_code
                , ignore_6_neighbors, extra_measurement_error, bypass_correction, independent_px_pz, only_count_logical_x, !imperfect_initialization
                , shallow_error_on_bottom);
        }
        ("decoder_comparison_benchmark", Some(matches)) => {
            let Ls = value_t!(matches, "Ls", String).expect("required");
            let Ls: Vec<usize> = serde_json::from_str(&Ls).expect("Ls should be [L1,L2,L3,...,Ln]");
            let Ts = value_t!(matches, "Ts", String).expect("required");
            let Ts: Vec<usize> = serde_json::from_str(&Ts).expect("Ts should be [T1,T2,T3,...,Tn]");
            assert!(Ts.len() == Ls.len(), "Ts and Ls should be paired");
            let ps = value_t!(matches, "ps", String).expect("required");
            let ps: Vec<f64> = serde_json::from_str(&ps).expect("ps should be [p1,p2,p3,...,pm]");
            let max_N = value_t!(matches, "max_N", usize).unwrap_or(100000000);  // default to 1e8
            let min_error_cases = value_t!(matches, "min_error_cases", usize).unwrap_or(10000);  // default to 1e3
            let parallel = value_t!(matches, "parallel", usize).unwrap_or(1);  // default to 1
            let validate_layer = value_t!(matches, "validate_layer", String).unwrap_or("boundary".to_string());
            let mini_batch = value_t!(matches, "mini_batch", usize).unwrap_or(1);  // default to 1
            let autotune = ! matches.is_present("no_autotune");  // default autotune is enabled
            let rotated_planar_code = matches.is_present("rotated_planar_code");  // default use standard planar code
            let ignore_6_neighbors = matches.is_present("ignore_6_neighbors");  // default use 12 neighbors version
            let extra_measurement_error = value_t!(matches, "extra_measurement_error", f64).unwrap_or(1.);  // default to 1.
            let bypass_correction = matches.is_present("bypass_correction");
            let independent_px_pz = matches.is_present("independent_px_pz");
            let only_count_logical_x = matches.is_present("only_count_logical_x");
            let imperfect_initialization = matches.is_present("imperfect_initialization");
            let substreams = value_t!(matches, "substreams", usize).unwrap_or(32);  // default to 32.
            decoder_comparison_benchmark(&Ls, &Ts, &ps, max_N, min_error_cases, parallel, validate_layer, mini_batch, autotune, rotated_planar_code
                , ignore_6_neighbors, extra_measurement_error, bypass_correction, independent_px_pz, only_count_logical_x, !imperfect_initialization, substreams);
        }
        _ => unreachable!()
    }
}

/**
default example:
    d = L = 3,5,7,9,11,15,25
    p = 3e-2,1e-2,3e-3,1e-3,3e-4,1e-4
`cargo run --release -- tool generate_random_errors [3,5,7,9,11,15,25] [3e-2,1e-2,3e-3,1e-3,3e-4,1e-4] 1000 -d ./tmp/random_errors`
**/
fn generate_random_errors(Ls: &Vec<usize>, ps: &Vec<f64>, N: usize, directory: &str) {
    for p in ps {
        for L in Ls {
            let p = *p;
            let L = *L;
            println!("p: {}, L: {} starting", p, L);
            let mut data_ro = BatchZxError::new_N_L(N, L);
            let mut data = data_ro.view_mut();
            let mut rng = thread_rng();
            let mut total_rounds = 0;
            let mut i = 0;
            let mut error_cnt = 0;
            while i < N {
                let mut has_error = false;
                for j in 0..L {
                    for k in 0..L {
                        let is_error = rng.gen::<f64>() < p;
                        if is_error {
                            error_cnt += 1;
                            has_error = true;
                        }
                        data[[i, j, k]] = is_error;
                    }
                }
                total_rounds += 1;  // record the total round
                if has_error {  // only save data when error occurs
                    i += 1;
                }
            }
            println!("    N/total_rounds = {}/{} = {}", N, total_rounds, N as f64 / total_rounds as f64);
            // prepare the head
            let error_rate = error_cnt as f64 / ((total_rounds*L*L) as f64);
            let head = serde_json::json!({
                "p": p,
                "error_cnt": error_cnt,
                "error_rate": error_rate,
                "total_rounds": total_rounds,
            });
            // save to file
            let filename = format!("errors_{}_{}.bin", p, L);
            let path = Path::new(directory).join(filename);
            util::save(path.to_str().expect("path string"), &head, &data_ro).expect("save failed");
        }
    }
}

/**
default example:
    d = L = 3,5,7,9,11,15,25
    p = 3e-2,1e-2,3e-3,1e-3,3e-4,1e-4
`cargo run --release -- tool decoder_benchmark [3,5,7,9,11,15,25] [3e-2,1e-2,3e-3,1e-3,3e-4,1e-4] -d ./tmp/random_errors -q naive_decoder`
**/
fn decoder_benchmark(Ls: &Vec<usize>, ps: &Vec<f64>, directory: &str, qec_decoder: &str) {
    println!("format: <p> <L> <total_rounds> <qec_failed> <error_rate>");
    for p in ps {
        for L in Ls {
            let p = *p;
            let L = *L;
            // load from file
            let filename = format!("errors_{}_{}.bin", p, L);
            let path = Path::new(directory).join(filename);
            let (head, data) = util::load(path.to_str().expect("path string")).expect("load failed");
            let total_rounds = head.get("total_rounds").expect("total_rounds").as_u64().expect("u64") as usize;
            let N = head.get("N").expect("N").as_u64().expect("u64") as usize;
            let no_error = ZxError::new_L(L);
            let mut qec_failed = 0;
            for i in 0..N {
                let x_error = ZxError::new(data.index_axis(Axis(0), i).to_owned());
                let measurement = util::generate_perfect_measurements(&x_error, &no_error);
                let (x_correction, _z_correction) = qec::naive_correction(&measurement);
                if x_error.validate_x_correction(&x_correction).is_err() {
                    qec_failed += 1;
                }
            }
            let error_rate = qec_failed as f64 / total_rounds as f64;
            println!("{} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate);
        }
    }
    if qec_decoder == "naive_decoder" {

    }
}

/**
default example:
`cargo run --release -- tool automatic_benchmark [3] [3e-2,1e-2,3e-3] -q naive_decoder`
**/
fn automatic_benchmark(Ls: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, qec_decoder: &str) {
    println!("format: <p> <L> <total_rounds> <qec_failed> <error_rate>");
    if qec_decoder == "naive_decoder" || qec_decoder == "maximum_max_weight_matching_decoder" || qec_decoder == "blossom_V" {
        for L in Ls {
            for p in ps {
                let p = *p;
                let L = *L;
                let no_error = ZxError::new_L(L);
                let mut x_error_ro = ZxError::new_L(L);
                let mut rng = thread_rng();
                let mut total_rounds = 0;
                let mut qec_failed = 0;
                while total_rounds < max_N && qec_failed < min_error_cases {
                    let mut x_error = x_error_ro.view_mut();
                    let mut has_error = false;
                    for i in 0..L {
                        for j in 0..L {
                            let is_error = rng.gen::<f64>() < p;
                            x_error[[i, j]] = is_error;
                            if is_error {
                                has_error = true;
                            }
                        }
                    }
                    total_rounds += 1;  // record the total round
                    if !has_error {
                        continue
                    }
                    let measurement = util::generate_perfect_measurements(&x_error_ro, &no_error);
                    let (x_correction, _z_correction) = if qec_decoder == "naive_decoder" {
                        qec::naive_correction(&measurement)
                    } else {  // "maximum_max_weight_matching_decoder" or "blossom_V"
                        if qec_decoder == "maximum_max_weight_matching_decoder" {
                            let maximum_max_weight_matching = |_node_num: usize, weighted_edges: Vec<(usize, usize, f64)>| 
                                -> std::collections::HashSet<(usize, usize)> {
                                    Python::with_gil(|py| {
                                        (|py: Python| -> PyResult<std::collections::HashSet<(usize, usize)>> {
                                            let networkx = py.import("networkx")?;
                                            let max_weight_matching = networkx.getattr("algorithms")?.getattr("matching")?.getattr("max_weight_matching")?;
                                            let G = networkx.call_method0("Graph")?;
                                            let weighted_edges = weighted_edges.to_object(py);
                                            G.call_method1("add_weighted_edges_from", (weighted_edges,))?;
                                            let dict = vec![("maxcardinality", true)].into_py_dict(py);
                                            let matched: std::collections::HashSet<(usize, usize)> = max_weight_matching.call((G,), Some(dict))?.extract()?;
                                            Ok(matched)
                                        })(py).map_err(|e| {
                                            e.print_and_set_sys_last_vars(py);
                                        })
                                    }).expect("python run failed")
                                };
                            qec::maximum_max_weight_matching_correction(&measurement, maximum_max_weight_matching)
                        } else {
                            qec::maximum_max_weight_matching_correction(&measurement, blossom_v::maximum_weight_perfect_matching_compatible)
                        }
                    };
                    if x_error_ro.validate_x_correction(&x_correction).is_err() {
                        qec_failed += 1;
                    }
                }
                let error_rate = qec_failed as f64 / total_rounds as f64;
                println!("{} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate);
            }
        }
    } else {
        println!("[error] unknown decoder");
    }
}

/**
default example:
`cargo run --release -- tool error_rate_MWPM_with_weight [5] [1e-2] -w default_weights.txt`
(run `python ../python/MWPM_weighted.py` to generate `default_weights.txt`)
**/
fn error_rate_MWPM_with_weight(Ls: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, weights_filename: &str, parallel: usize) {
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    // println!("format: <p> <L> <total_rounds> <qec_failed> <error_rate>");
    // read weights from file
    let file = std::fs::File::open(weights_filename).expect("file open failed");
    let mut lines = std::io::BufReader::new(file).lines();
    let weight_L = lines.next().expect("next").expect("should have L").parse::<usize>().expect("L usize");
    let mut weight_covered = ndarray::Array::from_elem((weight_L+1, weight_L+1, weight_L+1, weight_L+1), false);
    let mut weight_covered_mut = weight_covered.view_mut();
    let mut weights = ndarray::Array::from_elem((weight_L+1, weight_L+1, weight_L+1, weight_L+1), 0f64);
    let mut weights_mut = weights.view_mut();
    for line in lines {
        if let Ok(line) = line {
            let mut elements = line.split_ascii_whitespace();
            let i1 = elements.next().expect("next").parse::<usize>().expect("usize");
            let j1 = elements.next().expect("next").parse::<usize>().expect("usize");
            let i2 = elements.next().expect("next").parse::<usize>().expect("usize");
            let j2 = elements.next().expect("next").parse::<usize>().expect("usize");
            let weight = elements.next().expect("next").parse::<f64>().expect("f64");
            weight_covered_mut[[i1, j1, i2, j2]] = true;
            weights_mut[[i1, j1, i2, j2]] = weight;
        }
    }
    for i1 in 0..weight_L+1 {
        for j1 in 0..weight_L+1 {
            for i2 in 0..weight_L+1 {
                for j2 in 0..weight_L+1 {
                    let warning_weight_not_exist = true;  // whether warn if weight is missing in the weights file
                    if warning_weight_not_exist && !weight_covered[[i1, j1, i2, j2]] {
                        println!("[warning] weight of (i1, j1, i2, j2) = ({}, {}, {}, {}) missing, weights file might be wrong", i1, j1, i2, j2);
                    }
                }
            }
        }
    }
    for L in Ls {
        for p in ps {
            let total_rounds = Arc::new(Mutex::new(0));
            let qec_failed = Arc::new(Mutex::new(0));
            let mut handlers = Vec::new();
            let mut pb = ProgressBar::on(std::io::stderr(), max_N as u64);
            pb.set(0);
            for _i in 0..parallel {
                let p = *p;
                let L = *L;
                assert_eq!(weight_L, L);
                let total_rounds = Arc::clone(&total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let weights = weights.clone();
                handlers.push(std::thread::spawn(move || {
                    // println!("thread {}", _i);
                    let weights_of = |i1: usize, j1: usize, i2: usize, j2: usize| weights[[i1, j1, i2, j2]];
                    let no_error = ZxError::new_L(L);
                    let mut x_error_ro = ZxError::new_L(L);
                    let mut rng = thread_rng();
                    let mut current_total_rounds = {
                        *total_rounds.lock().unwrap()
                    };
                    let mut current_qec_failed = {
                        *qec_failed.lock().unwrap()
                    };
                    while current_total_rounds < max_N && current_qec_failed < min_error_cases {
                        let mini_batch = 1000;
                        let mut mini_qec_failed = 0;
                        for _j in 0..mini_batch {  // run at least `mini_batch` times before sync with outside
                            let mut x_error = x_error_ro.view_mut();
                            let mut has_error = false;
                            for i in 0..L {
                                for j in 0..L {
                                    let is_error = rng.gen::<f64>() < p;
                                    x_error[[i, j]] = is_error;
                                    if is_error {
                                        has_error = true;
                                    }
                                }
                            }
                            if !has_error {
                                continue
                            }
                            let measurement = util::generate_perfect_measurements(&x_error_ro, &no_error);
                            let (x_correction, _z_correction) = qec::maximum_max_weight_matching_correction_weighted(&measurement,
                                blossom_v::maximum_weight_perfect_matching_compatible, weights_of);
                            if x_error_ro.validate_x_correction(&x_correction).is_err() {
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
                let progress = total_rounds / max_N;
                pb.set(progress as u64);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
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
`cargo run --release -- tool fault_tolerant_benchmark [5] [5] [1e-3]`
it supports progress bar (in stderr), so you can run this in backend by redirect stdout to a file. This will not contain information of dynamic progress
**/
fn fault_tolerant_benchmark(Ls: &Vec<usize>, Ts: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, parallel: usize
        , validate_layer: String, mini_batch: usize, autotune: bool, rotated_planar_code: bool, ignore_6_neighbors: bool, extra_measurement_error: f64
        , bypass_correction: bool, independent_px_pz: bool, only_count_logical_x: bool, perfect_initialization: bool, shallow_error_on_bottom: bool) {
    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    println!("format: <p> <L> <T> <total_rounds> <qec_failed> <error_rate>");
    // println!("Perfect {} Rotated {} " ,perfect_initialization, rotated_planar_code);

    for (L_idx, L) in Ls.iter().enumerate() {
        let MeasurementRounds = Ts[L_idx];
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
            // build general models
            let mut model = if rotated_planar_code {
                ftqec::PlanarCodeModel::new_rotated_planar_code(MeasurementRounds, L)
            } else {
                ftqec::PlanarCodeModel::new_standard_planar_code(MeasurementRounds, L)
            };
            if !perfect_initialization {
                model.set_depolarizing_error(p);
            } else {
                // if we use the `set_depolarizing_error` model, then old judgement doesn't work
                // in order to verify that the modification is good, here we mimic the behavior of old model
                // that is, we do not generate error on the added bottom layer, so that there is no bottom boundary
                model.set_depolarizing_error_with_perfect_initialization(p);
            }
            if shallow_error_on_bottom {
                model.iterate_snapshot_mut(|t, _i, _j, node| {
                    if t == 6 && node.qubit_type == QubitType::Data {
                        node.error_rate_x = p;
                        node.error_rate_z = p;
                        node.error_rate_y = p;
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
            });
            model.build_graph();
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
            let model_error = model.clone();  // avoid copying decoding structure a lot of times
            model.optimize_correction_pattern();
            if !bypass_correction {
                if autotune {
                    model.build_exhausted_path_autotune();
                } else {
                    model.build_exhausted_path_equally_weighted();
                }
            }
            let model_decoder = Arc::new(model);  // only for decode, so that you're confident I'm not cheating by using information of original errors
            for _i in 0..parallel {
                let total_rounds = Arc::clone(&total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let mut model_error = model_error.clone();  // only for generating error and validating correction
                let model_decoder = Arc::clone(&model_decoder);  // only for decode, so that you're confident I'm not cheating by using information of original errors
                let validate_layer: isize = match validate_layer.as_str() {
                    "boundary" => -2,
                    "all" => -1,
                    "bottom" => 0,
                    "top" => MeasurementRounds as isize,
                    _ => validate_layer.parse::<isize>().expect("integer"),
                };
                let mini_batch = mini_batch;
                handlers.push(std::thread::spawn(move || {
                    // println!("thread {}", _i);
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
                            let error_count = model_error.generate_random_errors(|| rng.gen::<f64>());
                            if error_count == 0 {
                                continue
                            }
                            model_error.propagate_error();
                            let measurement = model_error.generate_measurement();
                            // use `model_decoder` for decoding, so that it is blind to the real error information
                            let correction = if !bypass_correction {
                                model_decoder.decode_MWPM(&measurement)
                            } else {
                                model_decoder.generate_default_correction()
                            };
                            if validate_layer == -2 {
                                let validation_ret = model_error.validate_correction_on_boundary(&correction);
                                if validation_ret.is_err() {
                                    if only_count_logical_x {
                                        match validation_ret {
                                            Err(ftqec::ValidationFailedReason::XLogicalError(_, _, _)) => { mini_qec_failed += 1; },
                                            Err(ftqec::ValidationFailedReason::BothXandZLogicalError(_, _, _, _, _)) => { mini_qec_failed += 1; },
                                            _ => {},
                                        }
                                    } else {
                                        mini_qec_failed += 1;
                                    }
                                }
                            } else if validate_layer == -1 {
                                // model_error.validate_correction_on_boundary(&correction);
                                if model_error.validate_correction_on_all_layers(&correction).is_err() {
                                    mini_qec_failed += 1;
                                }
                            } else {
                                let validation_ret = model_error.validate_correction_on_t_layer(&correction, validate_layer as usize);
                                if validation_ret.is_err() {
                                    if only_count_logical_x {  // only if contains logical X error will it count
                                        match validation_ret {
                                            Err(ftqec::ValidationFailedReason::XLogicalError(_, _, _)) => { mini_qec_failed += 1; },
                                            Err(ftqec::ValidationFailedReason::BothXandZLogicalError(_, _, _, _, _)) => { mini_qec_failed += 1; },
                                            _ => { },
                                        }
                                    } else {
                                        mini_qec_failed += 1;
                                    }
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
                pb.message(format!("{} {} {} {} {} {} ", p, L, MeasurementRounds, total_rounds, qec_failed, error_rate).as_str());
                let progress = total_rounds / mini_batch;
                pb.set(progress as u64);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            pb.finish();
            for handler in handlers {
                handler.join().unwrap();
            }
            let total_rounds = *total_rounds.lock().unwrap();
            let qec_failed = *qec_failed.lock().unwrap();
            let error_rate = qec_failed as f64 / total_rounds as f64;
            println!("{} {} {} {} {} {}", p, L, MeasurementRounds, total_rounds, qec_failed, error_rate);
        }
    }
}


fn decoder_comparison_benchmark(Ls: &Vec<usize>, Ts: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, parallel: usize
    , validate_layer: String, mini_batch: usize, autotune: bool, rotated_planar_code: bool, ignore_6_neighbors: bool, extra_measurement_error: f64 , bypass_correction: bool, independent_px_pz: bool, only_count_logical_x: bool, perfect_initialization: bool, substreams: usize) {

    let mut parallel = parallel;
    if parallel == 0 {
        parallel = num_cpus::get() - 1;
    }
    println!("format: <p> <L> <T> <total_rounds> <qec_failed_MWPM> <qec_failed_approx> <error_rate_MWPM> <error_rate_approx>");
    // println!("FT BM {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", Ls, Ts, ps, max_N, min_error_cases, parallel, validate_layer, mini_batch, autotune, rotated_planar_code, ignore_6_neighbors, extra_measurement_error);

    for (L_idx, L) in Ls.iter().enumerate() {
        let MeasurementRounds = Ts[L_idx];
        for p in ps {
            let p = *p;
            assert!(3. * p < 0.5, "why should errors (X, Z, Y) happening more than half of a time?");
            let L = *L;
            let total_rounds = Arc::new(Mutex::new(0));
            let qec_failed = Arc::new(Mutex::new((0,0)));
            let mut handlers = Vec::new();
            let mini_batch_count = 1 + max_N / mini_batch;
            let mut pb = ProgressBar::on(std::io::stderr(), mini_batch_count as u64);
            pb.set(0);
            // build general models
            let mut model = if rotated_planar_code {
                ftqec::PlanarCodeModel::new_rotated_planar_code(MeasurementRounds, L)
            } else {
                ftqec::PlanarCodeModel::new_standard_planar_code(MeasurementRounds, L)
            };
            if !perfect_initialization {
                model.set_depolarizing_error(p);
            } else {
                // if we use the `set_depolarizing_error` model, then old judgement doesn't work
                // in order to verify that the modification is good, here we mimic the behavior of old model
                // that is, we do not generate error on the added bottom layer, so that there is no bottom boundary
                model.set_depolarizing_error_with_perfect_initialization(p);
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
            });
            model.build_graph();
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
            let model_error = model.clone();  // avoid copying decoding structure a lot of times
            model.optimize_correction_pattern();
            if !bypass_correction {
                if autotune {
                    model.build_exhausted_path_autotune();
                } else {
                    model.build_exhausted_path_equally_weighted();
                }
            }
            let model_decoder_MWPM = Arc::new(model.clone());
            let model_decoder_approx = Arc::new(model.clone());  // only for decode, so that you're confident I'm not cheating by using information of original errors
            // println!("Parallel {:?}", parallel);
            for _i in 0..parallel {
                let total_rounds = Arc::clone(&total_rounds);
                // println!("Total rounds {:?}", total_rounds);
                let qec_failed = Arc::clone(&qec_failed);
                let mut model_error = model_error.clone();  // only for generating error and validating correction
                let model_decoder_MWPM = Arc::clone(&model_decoder_MWPM);
                let model_decoder_approx = Arc::clone(&model_decoder_approx);  // only for decode, so that you're confident I'm not cheating by using information of original errors
                let validate_layer: isize = match validate_layer.as_str() {
                    "boundary" => -2,
                    "all" => -1,
                    "bottom" => 0,
                    "top" => MeasurementRounds as isize,
                    _ => validate_layer.parse::<isize>().expect("integer"),
                };
                if validate_layer != -2 {
                    println!("Check evaluation metric");
                }
                let mini_batch = mini_batch;
                handlers.push(std::thread::spawn(move || {
                    // println!("thread {}", _i);
                    let mut rng = thread_rng();
                    let mut current_total_rounds = {
                        *total_rounds.lock().unwrap()
                    };
                    let mut current_qec_failed = {
                        *qec_failed.lock().unwrap()
                    };
                    while current_total_rounds < max_N && current_qec_failed.0 < min_error_cases {
                        // println!("current_total_rounds {:?}", current_total_rounds);
                        let mut mini_qec_failed = (0,0);
                        for _j in 0..mini_batch {  // run at least `mini_batch` times before sync with outside
                            let error_count = model_error.generate_random_errors(|| rng.gen::<f64>());
                            if error_count == 0 {
                                continue
                            }
                            // let original = model_error.clone();
                            // println!{"error_count {:?}", error_count};
                            model_error.propagate_error();
                            // let propagate = model_error.clone();
                            let measurement = model_error.generate_measurement();
                            // println!{"Measurement {:?}", measurement};
                            // use `model_decoder` for decoding, so that it is blind to the real error information
                            let correction_MWPM = model_decoder_MWPM.decode_MWPM(&measurement);
                            // let correction_MWPM = model_decoder_MWPM.decode_MWPM_approx(&measurement, substreams, false);
                            // println!("correction : {:?}", correction_MWPM);
                            let correction_approx = model_decoder_approx.decode_MWPM_approx(&measurement, substreams, false);
                            // println!("correction approx: {:?}", correction_approx);
                            // We need a new model to test approx corrections
                            let model_error_approx = model_error.clone();
                            if validate_layer == -2 {
                                let validation_ret = model_error.validate_correction_on_boundary(&correction_MWPM);
                                if validation_ret.is_err() {
                                    if only_count_logical_x {
                                        match validation_ret {
                                            Err(ftqec::ValidationFailedReason::XLogicalError(_, _, _)) => { mini_qec_failed.0 += 1; },
                                            Err(ftqec::ValidationFailedReason::BothXandZLogicalError(_, _, _, _, _)) => { mini_qec_failed.0 += 1; },
                                            _ => {},
                                        }
                                    } else {
                                        mini_qec_failed.0 += 1;
                                    }
                                }
                                let validation_ret = model_error_approx.validate_correction_on_boundary(&correction_approx);
                                if validation_ret.is_err() {
                                    if only_count_logical_x {
                                        match validation_ret {
                                            Err(ftqec::ValidationFailedReason::XLogicalError(_, _, _)) => { mini_qec_failed.1 += 1; },
                                            Err(ftqec::ValidationFailedReason::BothXandZLogicalError(_, _, _, _, _)) => { mini_qec_failed.1 += 1; },
                                            _ => {},
                                        }
                                    } else {
                                        mini_qec_failed.1 += 1;
                                    }
                                }
                            } else if validate_layer == -1 {
                                if model_error.validate_correction_on_all_layers(&correction_MWPM).is_err() {
                                    // println!("MWPM failed");
                                    mini_qec_failed.0 += 1;
                                }
                                if model_error_approx.validate_correction_on_all_layers(&correction_approx).is_err() {
                                    // println!("MWPM approx failed");
                                    mini_qec_failed.1 += 1;
                                }
                            } else {
                                println!("When boundary checking layer by layer checking not allowed");
                                // if model_error.validate_correction_on_t_layer(&correction_MWPM, validate_layer as usize).is_err() {
                                //     // println!("MWPM failed at {}",validate_layer);
                                //     mini_qec_failed.0 += 1;
                                // }
                                // if model_error_approx.validate_correction_on_t_layer(&correction_approx, validate_layer as usize).is_err() {
                                //     // println!("MWPM approx failed at {}", validate_layer);
                                //     mini_qec_failed.1 += 1;
                                // }
                            }
                            // if mini_qec_failed.0 != mini_qec_failed.1 {
                            //     println!("Original errors");
                            //     original.print_errors();
                            //     println!("Popagated Errors");
                            //     propagate.print_errors();
                            //     println!("{:?}", measurement);
                            //     println!("{:?}", correction_MWPM);
                            //     println!("{:?}", correction_approx);
                            //     return;
                            // }
                        }
                        // sync data from outside
                        current_total_rounds = {
                            let mut total_rounds = total_rounds.lock().unwrap();
                            // println!("total_rounds {:?}", *total_rounds);
                            *total_rounds += mini_batch;
                            *total_rounds
                        };
                        // println!("current_total_rounds {:?}", current_total_rounds);
                        current_qec_failed = {
                            let mut qec_failed = qec_failed.lock().unwrap();
                            *qec_failed = (qec_failed.0 + mini_qec_failed.0, qec_failed.1 + mini_qec_failed.1);
                            *qec_failed
                        };
                    }
                }));
            }
            loop {
                let total_rounds = *total_rounds.lock().unwrap();
                // println!("total_rounds {:?}", total_rounds);
                if total_rounds >= max_N { break }
                let qec_failed = *qec_failed.lock().unwrap();
                if qec_failed.0 >= min_error_cases { break }
                let error_rate = (qec_failed.0 as f64 / total_rounds as f64, qec_failed.1 as f64 / total_rounds as f64);
                pb.message(format!("{} {} {} {} {} {} {} {} ", p, L, MeasurementRounds, total_rounds, qec_failed.0, qec_failed.1,error_rate.0, error_rate.1).as_str());
                let progress = total_rounds / mini_batch;
                pb.set(progress as u64);
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            pb.finish();
            for handler in handlers {
                handler.join().unwrap();
            }
            let total_rounds = *total_rounds.lock().unwrap();
            let qec_failed = *qec_failed.lock().unwrap();
            let error_rate = (qec_failed.0 as f64 / total_rounds as f64, qec_failed.1 as f64 / total_rounds as f64);
            println!("{} {} {} {} {} {} {} {}", p, L, MeasurementRounds, total_rounds, qec_failed.0, qec_failed.1,error_rate.0, error_rate.1);
        }
    }
}
