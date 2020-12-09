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
            error_rate_MWPM_with_weight(&Ls, &ps, max_N, min_error_cases, &weights);
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
    if qec_decoder == "naive_decoder" || qec_decoder == "maximum_max_weight_matching_decoder" {
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
                    } else {  // maximum_max_weight_matching_decoder
                        let maximum_max_weight_matching = |weighted_edges: Vec<(usize, usize, f64)>| -> std::collections::HashSet<(usize, usize)> {
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
fn error_rate_MWPM_with_weight(Ls: &Vec<usize>, ps: &Vec<f64>, max_N: usize, min_error_cases: usize, weights_filename: &str) {
    // println!("format: <p> <L> <total_rounds> <qec_failed> <error_rate>");
    let maximum_max_weight_matching = |weighted_edges: Vec<(usize, usize, f64)>| -> std::collections::HashSet<(usize, usize)> {
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
    let weights_of = |i1: usize, j1: usize, i2: usize, j2: usize| weights[[i1, j1, i2, j2]];
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
                let (x_correction, _z_correction) = qec::maximum_max_weight_matching_correction_weighted(&measurement, maximum_max_weight_matching, weights_of);
                if x_error_ro.validate_x_correction(&x_correction).is_err() {
                    qec_failed += 1;
                }
            }
            let error_rate = qec_failed as f64 / total_rounds as f64;
            println!("{} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate);
        }
    }
}
