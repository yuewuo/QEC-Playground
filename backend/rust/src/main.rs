mod util;
mod test;
mod tool;

#[macro_use] extern crate clap;
#[macro_use] extern crate serde_json;
use serde_json::json;
#[macro_use] extern crate ndarray;
extern crate rand;

fn main() {

    let matches = clap_app!(QECPlayground =>
        (version: "1.0")
        (author: "Yue Wu yue.wu@yale.edu")
        (setting: clap::AppSettings::VersionlessSubcommands)
        (about: "Quantum Error Correction Playground for BIM'20 course")
        (setting: clap::AppSettings::SubcommandRequiredElseHelp)
        (@subcommand test => (about: "testing features")
            (setting: clap::AppSettings::SubcommandRequiredElseHelp)
            (@subcommand save_load => (about: "testing save and load functionality"))
        )
        (@subcommand tool => (about: "tools")
            (setting: clap::AppSettings::SubcommandRequiredElseHelp)
            (@subcommand generate_random_errors => (about: "generate random errors")
                (@arg Ls: +required "[L1,L2,L3,...,Ln]")
                (@arg ps: +required "[p1,p2,p3,...,pm]")
                (@arg N: +required "how many valid samples for each (d,p) config")
                (@arg directory: -d +takes_value "directory to output files, default to ./")
            )
        )
    ).get_matches();

    match matches.subcommand() {
        ("test", Some(matches)) => {
            test::run_matched_test(&matches);
        }
        ("tool", Some(matches)) => {
            tool::run_matched_tool(&matches);
        }
        _ => unreachable!()
    }

}
