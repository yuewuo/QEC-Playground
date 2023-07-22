#![allow(non_snake_case)]

use super::util::{local_get_temporary_store, local_put_temporary_store, TEMPORARY_STORE};
use crate::actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use crate::serde::Deserialize;
use clap::FromArgMatches;

pub const TEMPORARY_STORE_SIZE_LIMIT: usize = 10_000_000; // 10MB, only applicable to web service

pub async fn run_server(port: i32, addr: String, root_url: String) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(web::JsonConfig::default().limit(1024 * 1024 * 50)))
            .wrap(actix_cors::Cors::permissive())
            .service(
                web::scope(root_url.as_str().trim_end_matches('/')) // must remove trailing slashes from scope, see https://actix.rs/actix-web/actix_web/struct.Scope.html
                    .service(web::resource("hello").route(web::get().to(get_hello)))
                    .service(web::resource("version").route(web::get().to(get_version)))
                    .service(web::resource("view_noise_model").route(web::get().to(view_noise_model)))
                    .service(web::resource("new_temporary_store").route(web::post().to(new_temporary_store)))
                    .service(web::resource("get_temporary_store/{resource_id}").route(web::get().to(get_temporary_store))),
            )
    })
    .bind(format!("{}:{}", addr, port))?
    .run()
    .await
}

async fn get_hello() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("hello world"))
}

async fn get_version() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body(env!("CARGO_PKG_VERSION")))
}

fn default_probability() -> f64 {
    0.
}

fn default_parameters() -> String {
    "".into()
}

fn default_resource_id() -> usize {
    0
}

#[derive(Deserialize)]
struct ViewNoiseModelQuery {
    #[serde(default = "default_parameters")]
    parameters: String,
    #[serde(default = "default_probability")]
    p: f64,
    #[serde(default = "default_probability")]
    pe: f64,
    #[serde(default = "default_resource_id")]
    noise_model_temporary_id: usize,
}

/// call `tool benchmark` with code distance 5x5x5
async fn view_noise_model(info: web::Query<ViewNoiseModelQuery>) -> Result<HttpResponse, Error> {
    let di = 5;
    let dj = di;
    let T = di;
    let mut tokens = vec![
        format!("qecp"),
        format!("tool"),
        format!("benchmark"),
        format!("--debug_print"),
        format!("full-noise-model"),
        format!("[{}]", di),
        format!("--djs"),
        format!("[{}]", dj),
        format!("[{}]", T),
        format!("[{}]", info.p),
        format!("--pes"),
        format!("[{}]", info.pe),
    ];
    let temporary_store = TEMPORARY_STORE.read().unwrap(); // must acquire a reader lock, so that tool.rs is definitely; will slow down requests a little bit, but safety worth it
    if info.noise_model_temporary_id > 0 {
        match local_get_temporary_store(info.noise_model_temporary_id) {
            Some(_) => {}
            None => {
                return Ok(HttpResponse::NotFound().body(format!(
                    "noise_model_temporary_id={} not found, might be expired",
                    info.noise_model_temporary_id
                )))
            }
        }
        tokens.push(format!("--load_noise_model_from_temporary_store"));
        tokens.push(format!("{}", info.noise_model_temporary_id));
    }
    tokens.append(&mut match crate::shlex::split(&info.parameters) {
        Some(mut t) => t,
        None => return Ok(HttpResponse::BadRequest().body(format!("building tokens from parameters failed"))),
    });
    // println!("full_command: {:?}", tokens);
    use crate::clap::CommandFactory;
    use crate::cli::*;
    let cli = match Cli::command().color(clap::ColorChoice::Never).try_get_matches_from(tokens) {
        Ok(matches) => match Cli::from_arg_matches(&matches) {
            Ok(cli) => cli,
            Err(error) => return Ok(HttpResponse::BadRequest().body(format!("{:?}", error))),
        },
        Err(error) => return Ok(HttpResponse::BadRequest().body(format!("{:?}", error))),
    };
    let output = match cli.command {
        Commands::Tool { command } => command.run().expect("benchmark always gives output"),
        _ => unreachable!(), // forbid the web to access other commands
    };
    drop(temporary_store); // force the lifetime of locked temporary store to be more than `tool::run_matched_tool`
    Ok(HttpResponse::Ok().body(output))
}

#[derive(Deserialize)]
struct NewTemporaryStore {
    value: String,
}

async fn new_temporary_store(form: web::Json<NewTemporaryStore>) -> Result<HttpResponse, Error> {
    if form.value.len() > TEMPORARY_STORE_SIZE_LIMIT {
        return Ok(HttpResponse::BadRequest().body(format!(
            "upload size {} > limit {}",
            form.value.len(),
            TEMPORARY_STORE_SIZE_LIMIT
        )));
    }
    match local_put_temporary_store(form.value.clone()) {
        Some(insert_key) => {
            // println!("[web] inserted a temporary store with key: {}, length: {}", insert_key, form.value.len());
            Ok(HttpResponse::Ok().body(format!("{}", insert_key)))
        }
        None => Ok(HttpResponse::InternalServerError().body(format!("temporary store not available"))),
    }
}

async fn get_temporary_store(req: HttpRequest) -> Result<HttpResponse, Error> {
    let resource_id = match req.match_info().query("resource_id").parse::<usize>() {
        Ok(resource_id) => resource_id,
        Err(_) => return Ok(HttpResponse::BadRequest().body(format!("invalid resource id"))),
    };
    match local_get_temporary_store(resource_id) {
        Some(value) => Ok(HttpResponse::Ok().body(value.clone())),
        None => Ok(HttpResponse::NotFound().body(format!(
            "noise_model_temporary_id={} not found, might be expired",
            resource_id
        ))),
    }
}
