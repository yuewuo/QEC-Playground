#![cfg_attr(
    feature="python_binding",
    feature(cfg_eval)
)]

#[cfg(feature="python_binding")]
use pyo3::prelude::*;
pub mod util;
pub mod test;
pub mod tool;
pub mod types;
pub mod web;
pub mod cli;
pub mod blossom_v;
pub mod reproducible_rand;
// pub mod distributed_uf_decoder;  TODO: migrate back
// pub mod fpga_generator;  TODO: migrate back
// pub mod fast_benchmark;  TODO: migrate back
pub mod simulator;
pub mod code_builder;
#[macro_use] pub mod util_macros;
pub mod model_graph;
pub mod complete_model_graph;
pub mod error_model;
pub mod decoder_mwpm;
pub mod decoder_tailored_mwpm;
pub mod decoder_union_find;
pub mod tailored_model_graph;
pub mod tailored_complete_model_graph;
pub mod error_model_builder;
pub mod union_find;
pub mod erasure_graph;
pub mod decoder_fusion;

extern crate clap;
#[macro_use] extern crate serde_json;
extern crate ndarray;
extern crate rand;
extern crate actix_web;
extern crate actix_cors;
extern crate serde;
extern crate libc;
extern crate num_cpus;
extern crate petgraph;
extern crate pbr;
extern crate rand_core;
extern crate derivative;
extern crate derive_more;
extern crate lazy_static;
extern crate either;
extern crate shlex;
extern crate cfg_if;
#[cfg(feature="python_binding")]
extern crate pyo3;
extern crate platform_dirs;
extern crate serde_hashkey;
extern crate float_cmp;
extern crate priority_queue;
extern crate float_ord;
extern crate parking_lot;
extern crate fusion_blossom;

#[cfg(feature="python_binding")]
#[pymodule]
fn qecp(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    //panic!("hi");
    simulator::register(py, m)?;
    types::register(py, m)?;
    code_builder::register(py, m)?;
    error_model::register(py, m)?;
    Ok(())
}
