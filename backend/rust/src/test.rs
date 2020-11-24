use super::clap;
use super::util;
use super::ndarray;

pub fn run_matched_test(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("save_load", Some(_)) => {
            save_load()
        }
        _ => unreachable!()
    }
}

fn save_load() {
    let N = 16;
    let L = 5;
    let p = 1e-1;
    // generate some random data
    let data = ndarray::Array::from_shape_fn((N, L, L), |_| false);
    for i in 0..N {
        println!("i = {}", i);
    }
    
    let (head_r, data_r) =  util::load("tmp/save_load.bin");
    // println!("{:?}", head_r);
    // println!("{:?}", data_r);
}
