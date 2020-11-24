use super::clap;

pub fn run_matched_test(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("save_load", Some(_)) => {
            save_load()
        }
        _ => unreachable!()
    }
}

fn save_load() {
    println!("save_load ....")
}
