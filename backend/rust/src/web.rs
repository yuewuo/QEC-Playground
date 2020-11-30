#![allow(non_snake_case)]

use super::util;
use super::serde_json;
use super::serde::Deserialize;
use super::types::*;
use super::actix_web::{web, App, HttpServer, HttpResponse, Error, error};
use super::qec;

pub async fn run_server(port: i32, addr: String, root_url: String) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .service(
                web::scope(root_url.as_str())
                    .route("/hello", web::get().to(|| { HttpResponse::Ok().body("hello world") }))
                    .route("/stupid_decoder", web::post().to(stupid_decoder)),
            )
        }).bind(format!("{}:{}", addr, port))?.run().await
}

#[derive(Deserialize)]
pub struct DecodeSingleForm {
    L: usize,
    x_error: serde_json::Value,
    z_error: serde_json::Value,
}

// `array` should be JSON matrix of [L][L] where each element is 0 or 1
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

/// Decode a single error pattern using stupid_correction
async fn stupid_decoder(form: web::Json<DecodeSingleForm>) -> Result<HttpResponse, Error> {
    let L = form.L;
    if L < 2 { return Err(error::ErrorBadRequest("L must be no less than 2")) }
    let x_error = ZxError::new(parse_L2_bit_array_from_json(L, &form.x_error).map_err(|e| error::ErrorBadRequest(e))?);
    let z_error = ZxError::new(parse_L2_bit_array_from_json(L, &form.z_error).map_err(|e| error::ErrorBadRequest(e))?);
    let measurement = util::generate_perfect_measurements(&x_error, &z_error);
    let (x_correction, z_correction) = qec::stupid_correction(&measurement);
    let x_corrected = x_error.do_correction(&x_correction);
    let z_corrected = z_error.do_correction(&z_correction);
    let corrected_measurement = util::generate_perfect_measurements(&x_corrected, &z_corrected);
    let x_validate = x_error.validate_x_correction(&x_correction).is_ok();
    let z_validate = z_error.validate_z_correction(&z_correction).is_ok();
    let ret = json!({
        "x_error": output_L2_bit_array_to_json(&x_error),
        "z_error": output_L2_bit_array_to_json(&z_error),
        "measurement": output_L2_bit_array_to_json(&measurement),
        "x_correction": output_L2_bit_array_to_json(&x_correction),
        "z_correction": output_L2_bit_array_to_json(&z_correction),
        "x_corrected": output_L2_bit_array_to_json(&x_corrected),
        "z_corrected": output_L2_bit_array_to_json(&z_corrected),
        "corrected_measurement": output_L2_bit_array_to_json(&corrected_measurement),        
        "x_validate": x_validate,
        "z_validate": z_validate,
    });
    Ok(HttpResponse::Ok().body(serde_json::to_string(&ret)?))
}
