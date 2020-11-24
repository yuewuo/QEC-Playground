mod util;
mod test;

#[macro_use] extern crate clap;

fn main() {
    println!("Hello, world!");

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
    ).get_matches();

    match matches.subcommand() {
        ("test", Some(matches)) => {
            test::run_matched_test(&matches);
        }
        _ => unreachable!()
    }

}
