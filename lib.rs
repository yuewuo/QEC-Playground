#[cfg(feature="python_interfaces")]
use pyo3::prelude::*;
mod util;
mod test;
mod tool;
mod types;
//mod web;
mod blossom_v;
mod ftqec;
mod offer_decoder;
mod reproducible_rand;
mod offer_mwpm;
mod union_find_decoder;
mod distributed_uf_decoder;
mod fpga_generator;
mod fast_benchmark;
mod simulator;
mod code_builder;
#[macro_use] mod util_macros;
mod model_graph;
mod complete_model_graph;
mod error_model;
mod decoder_mwpm;
mod decoder_tailored_mwpm;
mod decoder_union_find;
mod tailored_model_graph;
mod tailored_complete_model_graph;
mod error_model_builder;
mod union_find;
mod erasure_graph;

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
#[macro_use] extern crate derivative;
extern crate derive_more;
extern crate lazy_static;
extern crate either;
extern crate rug;
extern crate shlex;
extern crate cfg_if;
#[cfg(feature="python_interfaces")]
extern crate pyo3;
extern crate platform_dirs;
extern crate serde_hashkey;
extern crate float_cmp;
extern crate priority_queue;
extern crate float_ord;
extern crate parking_lot;

#[cfg(feature="python_interfaces")]
#[pymodule]
fn qecp(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    //panic!("hi");
    simulator::register(py, m)?;
    types::register(py, m)?;
    Ok(())
}