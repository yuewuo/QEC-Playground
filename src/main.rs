extern crate clap;
extern crate pbr;

use qecp::cli::*;
use qecp::web;
use crate::clap::Parser;


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    match Cli::parse().command {
        Commands::Test { command } => {
            command.run();
        }
        Commands::Tool { command } => {
            let output = command.run();
            match output {
                Some(to_print) => { println!("{}", to_print); }
                None => { }
            }
        }
        Commands::Server(server_parameters) => {
            let port = server_parameters.port;
            let addr = server_parameters.addr;
            let root_url = server_parameters.root_url;
            println!("QECP server booting...");
            println!("visit http://{}:{}{}<commands>", addr, port, root_url);
            println!("supported commands include `hello`, `naive_decoder`, etc. See `web.rs` for more commands");
            web::run_server(port, addr, root_url).await?;
        }
    }

    Ok(())

}
