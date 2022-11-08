extern crate clap;
extern crate pbr;


use qecp::tool;
use qecp::test;
use qecp::cli;
use qecp::web;


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let matches = cli::create_clap_parser(clap::ColorChoice::Auto).get_matches();

    match matches.subcommand() {
        Some(("test", matches)) => {
            test::run_matched_test(&matches);
        }
        Some(("tool", matches)) => {
            let output = tool::run_matched_tool(&matches);
            match output {
                Some(to_print) => { println!("{}", to_print); }
                None => { }
            }
        }
        Some(("server", matches)) => {
            let port = matches.value_of("port").unwrap_or("8066").to_string().parse::<i32>().unwrap();
            let addr = matches.value_of("addr").unwrap_or("127.0.0.1").to_string();
            let root_url = matches.value_of("root_url").unwrap_or("/").to_string();
            println!("QECP server booting...");
            println!("visit http://{}:{}{}<commands>", addr, port, root_url);
            println!("supported commands include `hello`, `naive_decoder`, etc. See `web.rs` for more commands");
            web::run_server(port, addr, root_url).await?;
        }
        Some(("fpga_generator", _matches)) => {
            unimplemented!();
            // TODO: migrate fpga_generator to newer code
            // fpga_generator::run_matched_fpga_generator(&matches);
        }
        _ => unreachable!()
    }

    Ok(())

}
