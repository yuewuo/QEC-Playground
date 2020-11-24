use super::clap;
use super::util;

pub fn run_matched_test(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("save_load", Some(_)) => {
            save_load()
        }
        _ => unreachable!()
    }
}

fn save_load() {
    println!("save_load ....");
    util::load("tmp/save_load.bin");
}
