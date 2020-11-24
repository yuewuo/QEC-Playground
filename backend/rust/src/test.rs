use super::clap;
use super::util;
use super::ndarray;
use super::rand::prelude::*;
use super::serde_json;

pub fn run_matched_test(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("save_load", Some(_)) => {
            save_load()
        }
        _ => unreachable!()
    }
}

#[allow(non_snake_case)]
fn save_load() {
    let N = 16;
    let L = 5;
    let p = 1e-1;
    // generate some random data
    let mut data_ro = ndarray::Array::from_shape_fn((N, L, L), |_| false);
    let mut data = data_ro.view_mut();
    let mut rng = thread_rng();
    let mut error_cnt = 0;
    for i in 0..N {
        for j in 0..L {
            for k in 0..L {
                let is_error = rng.gen::<f64>() < p;
                if is_error {
                    error_cnt += 1;
                }
                data[[i, j, k]] = is_error;
            }
        }
    }
    let error_rate = error_cnt as f64 / ((N*L*L) as f64);
    println!("error/total: {}/{} = {}%", error_cnt, N*L*L, 100. * error_rate);
    // prepare the head
    let head = serde_json::json!({
        "p": p,
        "error_cnt": error_cnt,
        "error_rate": error_rate,
    });
    // save to file
    util::save("TEST_save_load.bin", &head, &data_ro).unwrap();
    // load from the same file
    let (head_r, data_r) = util::load("TEST_save_load.bin");
    // check whether the file contains the same information
    // println!("{:?}", head_r);
    // println!("{:?}", data_r);
}
