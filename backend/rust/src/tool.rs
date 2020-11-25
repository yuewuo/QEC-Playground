use super::clap;
use super::util;
use super::ndarray;
use super::rand::prelude::*;
use super::serde_json;
use std::path::Path;

#[allow(non_snake_case)]
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
        _ => unreachable!()
    }
}

/**
default example:
    d = 3,5,7,11,15 (thus L = 2d-1 = 5,9,13,21,29)
    p = 3e-2,1e-2,3e-3,1e-3,3e-4,1e-4
`cargo run --release -- tool generate_random_errors [5,9,13,21,29] [3e-2,1e-2,3e-3,1e-3,3e-4,1e-4] 1`
**/
#[allow(non_snake_case)]
fn generate_random_errors(Ls: &Vec<usize>, ps: &Vec<f64>, N: usize, directory: &str) {
    for pp in ps {
        for pL in Ls {
            let p = *pp;
            let L = *pL;
            println!("p: {}, L: {} starting", p, L);
            let mut data_ro = ndarray::Array::from_shape_fn((N, L, L), |_| false);
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
