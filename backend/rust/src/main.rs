mod util;
mod test;
mod tool;
mod types;
mod qec;
mod web;

#[macro_use] extern crate clap;
#[macro_use] extern crate serde_json;
extern crate ndarray;
extern crate rand;
extern crate actix_web;
extern crate actix_cors;
extern crate serde;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let matches = clap_app!(QECPlayground =>
        (version: "1.0")
        (author: "Yue Wu yue.wu@yale.edu")
        (setting: clap::AppSettings::VersionlessSubcommands)
        (about: "Quantum Error Correction Playground for BIM'20 course")
        (setting: clap::AppSettings::SubcommandRequiredElseHelp)
        (@subcommand test => (about: "testing features")
            (setting: clap::AppSettings::SubcommandRequiredElseHelp)
            (@subcommand save_load => (about: "testing save and load functionality"))
            (@subcommand perfect_measurement => (about: "print a perfect measurement"))
            (@subcommand validate_correction => (about: "validate x and z correction"))
            (@subcommand stupid_correction => (about: "a stupid error correction algorithm"))
            (@subcommand debug_tests => (about: "test for debug"))
            (@subcommand all => (about: "run all tests"))
        )
        (@subcommand tool => (about: "tools")
            (setting: clap::AppSettings::SubcommandRequiredElseHelp)
            (@subcommand generate_random_errors => (about: "generate random errors")
                (@arg Ls: +required "[L1,L2,L3,...,Ln]")
                (@arg ps: +required "[p1,p2,p3,...,pm]")
                (@arg N: +required "how many valid samples for each (d,p) config")
                (@arg directory: -d +takes_value "directory to output files, default to ./")
            )
            (@subcommand decoder_benchmark => (about: "test decoder")
                (@arg Ls: +required "[L1,L2,L3,...,Ln]")
                (@arg ps: +required "[p1,p2,p3,...,pm]")
                (@arg directory: -d +takes_value "directory to output files, default to ./")
                (@arg qec_decoder: -q +takes_value "available decoders, e.g. `stupid_decoder`")
            )
        )
        (@subcommand server => (about: "HTTP server for decoding information")
            (@arg port: -p --port +takes_value "listening on <addr>:<port>, default to 8066")
            (@arg addr: -a --addr +takes_value "listening on <addr>:<port>, default to \"127.0.0.1\"")
            (@arg root_url: -r --root_url +takes_value "root url")
        )
    ).get_matches();

    match matches.subcommand() {
        ("test", Some(matches)) => {
            test::run_matched_test(&matches);
        }
        ("tool", Some(matches)) => {
            tool::run_matched_tool(&matches);
        }
        ("server", Some(matches)) => {
            let port = matches.value_of("port").unwrap_or("8066").to_string().parse::<i32>().unwrap();
            let addr = matches.value_of("addr").unwrap_or("127.0.0.1").to_string();
            let root_url = matches.value_of("root_url").unwrap_or("/").to_string();
            println!("QECP server booting...");
            println!("visit http://{}:{}{}<commands>", addr, port, root_url);
            println!("supported commands include `hello`, `stupid_decoder`, etc. See `web.rs` for more commands");
            web::run_server(port, addr, root_url).await?;
        }
        _ => unreachable!()
    }

    Ok(())

}
