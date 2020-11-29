#![allow(non_snake_case)]

use super::clap;
use super::util;
use super::rand::prelude::*;
use super::serde_json;
use super::serde_json::{Value, Map};
use super::types::*;

pub fn run_matched_test(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("save_load", Some(_)) => {
            save_load()
        }
        ("perfect_measurement", Some(_)) => {
            perfect_measurement()
        }
        ("debug_tests", Some(_)) => {
            debug_tests()
        }
        _ => unreachable!()
    }
}

fn save_load() {
    let N = 16;
    let L = 5;
    let p = 1e-1;
    // generate some random data
    let mut data_ro = BatchZxError::new_N_L(N, L);
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
    assert_eq!(data_ro.N(), N);
    assert_eq!(data_ro.L(), L);
    let error_rate = error_cnt as f64 / ((N*L*L) as f64);
    println!("error/total: {}/{} = {}%", error_cnt, N*L*L, 100. * error_rate);
    // prepare the head
    let head = serde_json::json!({
        "p": p,
        "error_cnt": error_cnt,
        "error_rate": error_rate,
    });
    // save to file
    util::save("TEST_save_load.bin", &head, &data_ro).expect("save failed");
    // load from the same file
    let (head_r, data_r) = util::load("TEST_save_load.bin").expect("load failed");
    // check whether the file contains the same information
    let mut head_N_L: Map<String, Value> = serde_json::from_value(head.clone()).expect("head JSON error");
    head_N_L.insert("N".to_string(), json!(N));
    head_N_L.insert("L".to_string(), json!(L));
    let head_N_L: Value = serde_json::to_value(&head_N_L).expect("head JSON serialization error");
    assert_eq!(head_N_L, head_r);
    assert_eq!(data_ro, data_r);
}

fn perfect_measurement() {
    let mut z_error_ro = ZxError::new_L(5);
    let mut z_error = z_error_ro.view_mut();
    z_error[[1, 0]] = true;
    z_error[[3, 2]] = true;
    z_error[[3, 3]] = true;
    println!("z_error_ro:");
    z_error_ro.print();
    let zx_measurement= util::generate_perfect_measurements(&z_error_ro, &z_error_ro);
    println!("zx_measurement:");
    zx_measurement.print();
    // test rotation of measurement
    assert_eq!(zx_measurement.rotate_x2z().rotate_z2x(), zx_measurement);
}

fn debug_tests() {
    let mut z_error_ro = ZxError::new_L(5);
    let mut z_error = z_error_ro.view_mut();
    z_error[[1, 0]] = true;
    z_error[[3, 2]] = true;
    z_error[[3, 3]] = true;
    println!("z_error_ro:");
    z_error_ro.print();
    let rotated_clockwise = z_error_ro.rotate_x2z();
    println!("rotated_clockwise:");
    rotated_clockwise.print();
    let rotated_back = rotated_clockwise.rotate_z2x();
    assert_eq!(z_error_ro, rotated_back);
}
