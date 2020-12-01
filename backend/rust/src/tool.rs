#![allow(non_snake_case)]

use super::clap;
use super::util;
use super::rand::prelude::*;
use super::serde_json;
use std::path::Path;
use super::types::*;
use super::ndarray::{Axis};
use super::qec;

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
            let qec_decoder = value_t!(matches, "qec_decoder", String).unwrap_or("stupid_decoder".to_string());
            decoder_benchmark(&Ls, &ps, &directory, &qec_decoder);
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
`cargo run --release -- tool decoder_benchmark [3,5,7,9,11,15,25] [3e-2,1e-2,3e-3,1e-3,3e-4,1e-4] -d ./tmp/random_errors -q stupid_decoder`
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
                let (x_correction, _z_correction) = qec::stupid_correction(&measurement);
                if x_error.validate_x_correction(&x_correction).is_err() {
                    qec_failed += 1;
                }
            }
            let error_rate = qec_failed as f64 / total_rounds as f64;
            println!("{} {} {} {} {}", p, L, total_rounds, qec_failed, error_rate);
        }
    }
    if qec_decoder == "stupid_decoder" {

    }
}
