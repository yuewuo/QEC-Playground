#![allow(non_snake_case)]

#[cfg(not(feature="noserver"))]
use super::util;
#[cfg(not(feature="noserver"))]
use super::serde_json;
#[cfg(not(feature="noserver"))]
use super::serde::Deserialize;
#[cfg(not(feature="noserver"))]
use super::types::*;
#[cfg(not(feature="noserver"))]
use super::actix_web::{web, App, HttpServer, HttpResponse, Error, error};
#[cfg(not(feature="noserver"))]
use super::qec;
#[cfg(not(feature="noserver"))]
use super::pyo3::prelude::*;
#[cfg(not(feature="noserver"))]
use super::pyo3::types::{IntoPyDict};

#[cfg(not(feature="noserver"))]
pub async fn run_server(port: i32, addr: String, root_url: String) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .service(
                web::scope(root_url.as_str())
                    .route("/hello", web::get().to(|| { HttpResponse::Ok().body("hello world") }))
                    .route("/naive_decoder", web::post().to(naive_decoder))
                    .route("/MWPM_decoder", web::post().to(maximum_max_weight_matching_decoder))
                    .route("/view_error_model", web::get().to(view_error_model))
            )
        }).bind(format!("{}:{}", addr, port))?.run().await
}


#[cfg(feature="noserver")]
pub async fn run_server(port: i32, addr: String, root_url: String) -> std::io::Result<()> {
    panic!("compiled with feature `noserver`, cannot run server at {}:{}, root {}", port, addr, root_url)
}

#[cfg(not(feature="noserver"))]
#[derive(Deserialize)]
pub struct DecodeSingleForm {
    L: usize,
    x_error: serde_json::Value,
    z_error: serde_json::Value,
}

// `array` should be JSON matrix of [L][L] where each element is 0 or 1
#[cfg(not(feature="noserver"))]
fn parse_L2_bit_array_from_json(L: usize, array: &serde_json::Value) -> Result<ndarray::Array2<bool>, String> {
    let mut ret_ro = ndarray::Array::from_elem((L, L), false);
    let mut ret = ret_ro.view_mut();
    let rows = array.as_array().ok_or("JSON must be array")?;
    if rows.len() != L { return Err(format!("JSON must have L={} rows", L)) }
    for i in 0..L {
        let row = rows[i].as_array().ok_or("JSON must be matrix")?;
        if row.len() != L { return Err(format!("JSON must have L={} columns", L)) }
        for j in 0..L {
            ret[[i, j]] = row[j].as_u64().ok_or("each element must be integer")? == 1;
        }
    }
    Ok(ret_ro)
}

#[cfg(not(feature="noserver"))]
fn output_L2_bit_array_to_json(array: &ndarray::Array2<bool>) -> serde_json::Value {
    let shape = array.shape();
    let mut matrix = Vec::<Vec<i32>>::new();
    for i in 0..shape[0] {
        let mut row = Vec::<i32>::new();
        for j in 0..shape[1] {
            row.push(if array[[i, j]] { 1 } else { 0 });
        }
        matrix.push(row);
    }
    json!(matrix)
}

/// Decode a single error pattern using naive_correction
#[cfg(not(feature="noserver"))]
async fn naive_decoder(form: web::Json<DecodeSingleForm>) -> Result<HttpResponse, Error> {
    let L = form.L;
    if L < 2 { return Err(error::ErrorBadRequest("L must be no less than 2")) }
    let x_error = ZxError::new(parse_L2_bit_array_from_json(L, &form.x_error).map_err(|e| error::ErrorBadRequest(e))?);
    let z_error = ZxError::new(parse_L2_bit_array_from_json(L, &form.z_error).map_err(|e| error::ErrorBadRequest(e))?);
    let measurement = util::generate_perfect_measurements(&x_error, &z_error);
    let (x_correction, z_correction) = qec::naive_correction(&measurement);
    let x_corrected = x_error.do_correction(&x_correction);
    let z_corrected = z_error.do_correction(&z_correction);
    let corrected_measurement = util::generate_perfect_measurements(&x_corrected, &z_corrected);
    let x_valid = x_error.validate_x_correction(&x_correction).is_ok();
    let z_valid = z_error.validate_z_correction(&z_correction).is_ok();
    let if_all_x_stabilizers_plus1 = z_corrected.if_all_x_stabilizers_plus1();  // x stabilizers only detect z errors
    let if_all_z_stabilizers_plus1 = x_corrected.if_all_z_stabilizers_plus1();
    let ret = json!({
        "x_error": output_L2_bit_array_to_json(&x_error),
        "z_error": output_L2_bit_array_to_json(&z_error),
        "measurement": output_L2_bit_array_to_json(&measurement),
        "x_correction": output_L2_bit_array_to_json(&x_correction),
        "z_correction": output_L2_bit_array_to_json(&z_correction),
        "x_corrected": output_L2_bit_array_to_json(&x_corrected),
        "z_corrected": output_L2_bit_array_to_json(&z_corrected),
        "corrected_measurement": output_L2_bit_array_to_json(&corrected_measurement),
        "x_valid": x_valid,
        "z_valid": z_valid,
        "if_all_x_stabilizers_plus1": if_all_x_stabilizers_plus1,
        "if_all_z_stabilizers_plus1": if_all_z_stabilizers_plus1,
    });
    Ok(HttpResponse::Ok().body(serde_json::to_string(&ret)?))
}

/// Decode a single error pattern using naive_correction
#[cfg(not(feature="noserver"))]
async fn maximum_max_weight_matching_decoder(form: web::Json<DecodeSingleForm>) -> Result<HttpResponse, Error> {
    let L = form.L;
    if L < 2 { return Err(error::ErrorBadRequest("L must be no less than 2")) }
    let x_error = ZxError::new(parse_L2_bit_array_from_json(L, &form.x_error).map_err(|e| error::ErrorBadRequest(e))?);
    let z_error = ZxError::new(parse_L2_bit_array_from_json(L, &form.z_error).map_err(|e| error::ErrorBadRequest(e))?);
    let measurement = util::generate_perfect_measurements(&x_error, &z_error);
    let (x_correction, z_correction) = Python::with_gil(|py| {
        (|py: Python| -> PyResult<(ZxCorrection, ZxCorrection)> {
            // prepare python library
            let networkx = py.import("networkx")?;
            let max_weight_matching = networkx.getattr("algorithms")?.getattr("matching")?.getattr("max_weight_matching")?;
            let maximum_max_weight_matching = |_node_num: usize, weighted_edges: Vec<(usize, usize, f64)>| -> std::collections::HashSet<(usize, usize)> {
                let G = networkx.call_method0("Graph").unwrap();
                let weighted_edges = weighted_edges.to_object(py);
                G.call_method1("add_weighted_edges_from", (weighted_edges,)).unwrap();
                let dict = vec![("maxcardinality", true)].into_py_dict(py);
                let matched: std::collections::HashSet<(usize, usize)> = max_weight_matching.call((G,), Some(dict)).unwrap().extract().unwrap();
                matched
            };
            let (x_correction, z_correction) = qec::maximum_max_weight_matching_correction(&measurement, maximum_max_weight_matching);
            Ok((x_correction, z_correction))
        })(py).map_err(|e| {
            e.print_and_set_sys_last_vars(py);
        })
    }).expect("python run failed");
    let x_corrected = x_error.do_correction(&x_correction);
    let z_corrected = z_error.do_correction(&z_correction);
    let corrected_measurement = util::generate_perfect_measurements(&x_corrected, &z_corrected);
    let x_valid = x_error.validate_x_correction(&x_correction).is_ok();
    let z_valid = z_error.validate_z_correction(&z_correction).is_ok();
    let if_all_x_stabilizers_plus1 = z_corrected.if_all_x_stabilizers_plus1();  // x stabilizers only detect z errors
    let if_all_z_stabilizers_plus1 = x_corrected.if_all_z_stabilizers_plus1();
    let ret = json!({
        "x_error": output_L2_bit_array_to_json(&x_error),
        "z_error": output_L2_bit_array_to_json(&z_error),
        "measurement": output_L2_bit_array_to_json(&measurement),
        "x_correction": output_L2_bit_array_to_json(&x_correction),
        "z_correction": output_L2_bit_array_to_json(&z_correction),
        "x_corrected": output_L2_bit_array_to_json(&x_corrected),
        "z_corrected": output_L2_bit_array_to_json(&z_corrected),
        "corrected_measurement": output_L2_bit_array_to_json(&corrected_measurement),
        "x_valid": x_valid,
        "z_valid": z_valid,
        "if_all_x_stabilizers_plus1": if_all_x_stabilizers_plus1,
        "if_all_z_stabilizers_plus1": if_all_z_stabilizers_plus1,
    });
    Ok(HttpResponse::Ok().body(serde_json::to_string(&ret)?))
}

fn default_probability() -> f64 {
    0.
}

#[derive(Deserialize)]
struct ViewErrorModelQuery {
    parameters: String,
    #[serde(default = "default_probability")]
    p: f64,
    #[serde(default = "default_probability")]
    pe: f64,
}

/// call `tool fault_tolerant_benchmark` with code distance 5x5x5
#[cfg(not(feature="noserver"))]
async fn view_error_model(info: web::Query<ViewErrorModelQuery>) -> Result<HttpResponse, Error> {
    let di = 5;
    let dj = di;
    let T = di;
    let mut tokens = vec![format!("rust_qecp"), format!("tool"), format!("fault_tolerant_benchmark")
        , format!("--debug_print_only"), format!("--debug_print_error_model")
        , format!("[{}]", di), format!("--djs"), format!("[{}]", dj)
        , format!("[{}]", T), format!("[{}]", info.p), format!("--pes"), format!("[{}]", info.pe)];
    tokens.append(&mut match super::shlex::split(&info.parameters) {
        Some(mut t) => t,
        None => {
            return Ok(HttpResponse::BadRequest().body(format!("building tokens from parameters failed")))
        }
    });
    // println!("full_command: {:?}", tokens);
    let matches = match super::create_clap_parser(clap::AppSettings::ColorNever).get_matches_from_safe(tokens) {
        Ok(matches) => matches,
        Err(error) => { return Ok(HttpResponse::BadRequest().body(error.message)) }
    };
    let output = match matches.subcommand() {
        ("tool", Some(matches)) => {
            super::tool::run_matched_tool(&matches).expect("fault_tolerant_benchmark always gives output")
        }
        _ => unreachable!()
    };
    Ok(HttpResponse::Ok().body(output))
}
